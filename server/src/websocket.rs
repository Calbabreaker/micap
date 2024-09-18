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
    main_server::{GlobalConfig, MainServer, ServerEvent},
    serial::SerialPortManager,
    tracker::{TrackerData, TrackerInfo},
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Clone, Serialize, TS)]
#[serde(tag = "type")]
pub enum WebsocketServerMessage<'a> {
    TrackerUpdate {
        id: &'a String,
        #[ts(optional)]
        info: Option<&'a TrackerInfo>,
        #[ts(optional)]
        data: Option<&'a TrackerData>,
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
    SerialPortChanged {
        #[ts(optional)]
        port_name: Option<String>,
    },
    // Passes through the server events
    #[serde(untagged)]
    ServerEvent(&'a ServerEvent),
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
    serial_manager: SerialPortManager,
}

impl WebsocketServer {
    pub async fn new() -> anyhow::Result<Self> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT));
        log::info!("Started websocket server on {address}");
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            listener,
            ws_stream: None,
            serial_manager: SerialPortManager::default(),
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        self.feed_ws_messages(main).await;

        if let Some(ws_stream) = self.ws_stream.as_mut() {
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

                // Get the tracker infos from main
                let tracker_infos = futures_util::stream::iter(&main.trackers)
                    .then(|(id, tracker)| async { (id.clone(), tracker.read().await.info.clone()) })
                    .collect()
                    .await;

                let message = WebsocketServerMessage::InitialState {
                    tracker_infos,
                    config: &main.config,
                    port_name: self.serial_manager.port_name(),
                };

                feed_ws_message(&mut ws_stream, message).await;
                ws_stream.flush().await?;
                self.ws_stream = Some(ws_stream);
                log::info!("Websocket client connected at {peer_addr}");
            }
            Some(Err(e)) => Err(e)?,
            None => (),
        }

        Ok(())
    }

    async fn feed_ws_messages(&mut self, main: &mut MainServer) {
        let ws_stream = match self.ws_stream.as_mut() {
            Some(s) => s,
            None => return,
        };

        // Send the serial stuff
        if self.serial_manager.check_port().await {
            let message = WebsocketServerMessage::SerialPortChanged {
                port_name: self.serial_manager.port_name(),
            };
            feed_ws_message(ws_stream, message).await;
        }

        if let Some(log) = self.serial_manager.read_line() {
            feed_ws_message(ws_stream, WebsocketServerMessage::SerialLog { log }).await;
        }

        // Send events
        for event in &main.events {
            feed_ws_message(ws_stream, WebsocketServerMessage::ServerEvent(event)).await;
        }

        for (id, tracker) in &main.trackers {
            let tracker = tracker.read().await;
            if tracker.info_was_updated || tracker.data_was_updated {
                let message = WebsocketServerMessage::TrackerUpdate {
                    id,
                    info: match tracker.info_was_updated {
                        true => Some(&tracker.info),
                        false => None,
                    },
                    data: match tracker.data_was_updated {
                        true => Some(&tracker.data),
                        false => None,
                    },
                };
                feed_ws_message(ws_stream, message).await;
            }
        }

        ws_stream.flush().await.ok();
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
                self.serial_manager.write(data.as_bytes())?;
            }
            WebsocketClientMessage::RemoveTracker { id } => {
                if let Some(tracker) = main.trackers.get(&id) {
                    tracker.write().now_or_never().unwrap().to_be_removed = true;
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
