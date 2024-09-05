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
    },
}

// Receieved from client
#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
    RemoveTracker { id: String },
    UpdateConfig { config: GlobalConfig },
}

pub struct WebsocketServer {
    listener: TcpListener,
    ws_streams: Vec<WebSocketStream<TcpStream>>,
}

impl WebsocketServer {
    pub async fn new() -> anyhow::Result<Self> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT));
        log::info!("Started websocket server on {address}");
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            listener,
            ws_streams: Vec::new(),
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        match self.listener.accept().now_or_never() {
            Some(Ok((stream, peer_addr))) => {
                let ws_stream = tokio_tungstenite::accept_async(stream).await?;
                log::info!("Websocket client connected at {peer_addr}");
                self.on_connect(ws_stream, main).await?;
            }
            Some(Err(e)) => Err(e)?,
            None => (),
        }

        // Get list of messages to send
        let messages_to_send = std::iter::empty()
            .chain(main.updates.iter().filter_map(|update| {
                match update {
                    UpdateEvent::TrackerInfoUpdate { id } => {
                        // Convert to websocket server message to include the tracker info
                        serde_json::to_string(&WebsocketServerMessage::TrackerInfo {
                            id,
                            info: &main.trackers[id].info,
                        })
                    }
                    event => serde_json::to_string(&event),
                }
                .ok()
            }))
            .chain(main.trackers.iter().filter_map(|(id, tracker)| {
                if tracker.info.status != TrackerStatus::Ok {
                    return None;
                }

                // Data from trackers
                serde_json::to_string(&WebsocketServerMessage::TrackerData {
                    id,
                    data: &tracker.data,
                })
                .ok()
            }))
            .collect::<Vec<_>>();

        for i in (0..self.ws_streams.len()).rev() {
            if handle_websocket(main, &mut self.ws_streams[i], &messages_to_send).await? {
                self.ws_streams.remove(i);
            }
        }

        Ok(())
    }

    async fn on_connect(
        &mut self,
        mut ws_stream: WebSocketStream<TcpStream>,
        main: &mut MainServer,
    ) -> anyhow::Result<()> {
        let string = serde_json::to_string(&WebsocketServerMessage::InitialState {
            trackers: &main.trackers,
            config: &main.config,
        })?;

        ws_stream.send(Message::text(string)).await?;
        self.ws_streams.push(ws_stream);
        Ok(())
    }
}

// Return true if should remove the websocket
async fn handle_websocket(
    main: &mut MainServer,
    ws_stream: &mut WebSocketStream<TcpStream>,
    messages_to_send: &[String],
) -> anyhow::Result<bool> {
    match ws_stream.next().now_or_never() {
        Some(Some(Ok(message))) => {
            if let Ok(text) = message.to_text() {
                handle_websocket_message(text, main)
                    .context("Failed to handle websocket message")?;
            }
        }
        Some(Some(Err(err))) => {
            // Socket was closed
            log::error!("Websocket error: {err}");
            return Ok(true);
        }
        Some(None) => {
            // Socket was closed
            log::info!("Websocket disconnected");
            return Ok(true);
        }
        // Future has not ended
        None => (),
    }

    for message in messages_to_send.iter() {
        ws_stream.feed(Message::text(message)).await.ok();
    }

    if !messages_to_send.is_empty() {
        ws_stream.flush().await?;
    }

    Ok(false)
}

fn handle_websocket_message(message: &str, main: &mut MainServer) -> anyhow::Result<()> {
    if message.is_empty() {
        return Ok(());
    }

    match serde_json::from_str(message)? {
        WebsocketClientMessage::Wifi { ssid, password } => {
            if ssid.len() > 32 || password.len() > 64 {
                anyhow::bail!("SSID or password too long");
            }

            let data = format!("Wifi\0{ssid}\0{password}\n");
            main.serial_manager.write(data.as_bytes())?;
        }
        WebsocketClientMessage::FactoryReset => {
            main.serial_manager.write(b"FactoryReset\n")?;
        }
        WebsocketClientMessage::RemoveTracker { id } => {
            main.remove_tracker(&id);
            main.save_config()?;
        }
        WebsocketClientMessage::UpdateConfig { config } => {
            main.config = config;
        }
    }

    Ok(())
}
