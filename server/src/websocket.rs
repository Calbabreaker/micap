use anyhow::Context;
use futures_util::{FutureExt, SinkExt, StreamExt};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    main_server::{GlobalConfig, MainServer, UpdateEvent},
    tracker::{Tracker, TrackerData, TrackerInfo, TrackerStatus},
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum WebsocketServerMessage<'a> {
    TrackerInfo {
        id: &'a String,
        info: &'a TrackerInfo,
    },
    TrackerData {
        id: &'a String,
        data: &'a TrackerData,
    },
    InitialState {
        trackers: &'a HashMap<String, Tracker>,
        config: &'a GlobalConfig,
        port_name: Option<String>,
    },
    SerialLog {
        log: &'a str,
    },
    // Passes throught the server events
    #[serde(untagged)]
    UpdateEvent(&'a UpdateEvent),
}

// Receieved from client
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    SerialSend { data: String },
    RemoveTracker { id: String },
    UpdateConfig { config: GlobalConfig },
}

pub struct WebsocketServer {
    listener: TcpListener,
    ws_stream: Option<WebSocketStream<TcpStream>>,
}

impl WebsocketServer {
    pub async fn new() -> anyhow::Result<Self> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT));
        log::info!("Started websocket server on {address}");
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            listener,
            ws_stream: None,
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            for message in get_messages_to_send(main) {
                ws_stream.feed(message.clone()).await.ok();
            }

            ws_stream.flush().await.ok();

            // Get new data from websocket
            match ws_stream.next().now_or_never() {
                Some(Some(Ok(message))) => {
                    if let Ok(text) = message.to_text() {
                        self.handle_websocket_message(text, main)
                            .context("Failed to handle websocket message")?;
                    }
                }
                Some(None) | Some(Some(Err(_))) => {
                    // Socket was closed so remove the ws stream
                    log::info!("Websocket disconnected");
                    self.ws_stream.take();
                }
                // Data has not been processed yet
                None => (),
            }
        }

        match self.listener.accept().now_or_never() {
            Some(Ok((stream, peer_addr))) => {
                if self.ws_stream.is_some() {
                    anyhow::bail!("Websocket already connected, not connecting");
                }

                let mut ws_stream = tokio_tungstenite::accept_async(stream).await?;
                let string = serde_json::to_string(&WebsocketServerMessage::InitialState {
                    trackers: &main.trackers,
                    config: &main.config,
                    port_name: main.serial_manager.port_name(),
                })?;

                ws_stream.send(Message::text(string)).await?;
                self.ws_stream = Some(ws_stream);
                log::info!("Websocket client connected at {peer_addr}");
            }
            Some(Err(e)) => Err(e)?,
            None => (),
        }

        Ok(())
    }

    fn handle_websocket_message(
        &mut self,
        message: &str,
        main: &mut MainServer,
    ) -> anyhow::Result<()> {
        if message.is_empty() {
            return Ok(());
        }

        match serde_json::from_str(message)? {
            WebsocketClientMessage::SerialSend { data } => {
                log::info!("Writing {data:?} to port");
                main.serial_manager.write(data.as_bytes())?;
            }
            WebsocketClientMessage::RemoveTracker { id } => {
                main.remove_tracker(&id);
                main.save_config()?;
            }
            WebsocketClientMessage::UpdateConfig { config } => {
                main.config = config;
                main.save_config()?;
            }
        }

        Ok(())
    }
}

// Get list of messages to send
fn get_messages_to_send(main: &mut MainServer) -> impl Iterator<Item = Message> + use<'_> {
    std::iter::empty()
        // Add the server events
        .chain(main.updates.iter().map(|update| {
            Some(match update {
                UpdateEvent::TrackerInfoUpdate { id } => {
                    // Convert to websocket server message to include the tracker info
                    WebsocketServerMessage::TrackerInfo {
                        id,
                        info: &main.trackers[id].info,
                    }
                }
                event => WebsocketServerMessage::UpdateEvent(event),
            })
        }))
        // Add the tracker data
        .chain(main.trackers.iter().map(|(id, tracker)| {
            // Only send if client requested listen and is Ok
            if tracker.info.status != TrackerStatus::Ok {
                return None;
            }

            // Data from trackers
            Some(WebsocketServerMessage::TrackerData {
                id,
                data: &tracker.data,
            })
        }))
        .chain(std::iter::once_with(|| {
            Some(WebsocketServerMessage::SerialLog {
                log: main.serial_manager.read_line()?,
            })
        }))
        .filter_map(|m| Some(Message::Text(serde_json::to_string(&m).ok()?)))
}
