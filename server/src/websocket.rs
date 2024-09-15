use anyhow::Context;
use futures_util::{FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use ts_rs::TS;

use crate::{
    main_server::{GlobalConfig, MainServer, UpdateEvent},
    tracker::{TrackerData, TrackerInfo},
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Clone, Serialize, TS)]
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
        tracker_infos: HashMap<String, TrackerInfo>,
        config: &'a GlobalConfig,
        #[ts(optional)]
        port_name: Option<String>,
    },
    SerialLog {
        log: &'a str,
    },
    // Passes through the server events
    #[serde(untagged)]
    UpdateEvent(&'a UpdateEvent),
}

// Receieved from client
#[derive(Deserialize, TS)]
#[serde(tag = "type")]
pub enum WebsocketClientMessage {
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
            feed_ws_messages(ws_stream, main).await;
            ws_stream.flush().await.ok();

            // Get new data from websocket
            match ws_stream.next().now_or_never() {
                Some(Some(Ok(message))) => {
                    if let Ok(text) = message.to_text() {
                        self.handle_websocket_message(text, main)
                            .await
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

                // Get the tracker infos from main
                let tracker_infos = futures_util::stream::iter(&main.trackers)
                    .then(|(id, tracker)| async { (id.clone(), tracker.read().await.info.clone()) })
                    .collect()
                    .await;

                let string = serde_json::to_string(&WebsocketServerMessage::InitialState {
                    tracker_infos,
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

    async fn handle_websocket_message(
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
                if let Some(tracker) = main.trackers.get(&id) {
                    tracker.write().await.to_be_removed = true;
                }
            }
            WebsocketClientMessage::UpdateConfig { config } => {
                main.config = config;
                main.save_config()?;
            }
        }

        Ok(())
    }
}

async fn feed_ws_message<'a>(
    ws_stream: &mut WebSocketStream<TcpStream>,
    ws_message: WebsocketServerMessage<'a>,
) {
    if let Ok(text) = serde_json::to_string(&ws_message) {
        ws_stream.feed(Message::Text(text)).await.ok();
    }
}

async fn feed_ws_messages(ws_stream: &mut WebSocketStream<TcpStream>, main: &mut MainServer) {
    // Send messages
    for event in &main.updates {
        feed_ws_message(ws_stream, WebsocketServerMessage::UpdateEvent(event)).await;
    }

    for (id, tracker) in &main.trackers {
        let tracker = tracker.read().await;
        if tracker.info_was_updated {
            feed_ws_message(
                ws_stream,
                WebsocketServerMessage::TrackerInfo {
                    id,
                    info: &tracker.info,
                },
            )
            .await;
        }

        if tracker.data_was_updated {
            feed_ws_message(
                ws_stream,
                WebsocketServerMessage::TrackerData {
                    id,
                    data: &tracker.data,
                },
            )
            .await;
        }
    }

    if let Some(log) = main.serial_manager.read_line() {
        feed_ws_message(ws_stream, WebsocketServerMessage::SerialLog { log }).await;
    }
}
