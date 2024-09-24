use futures_util::{FutureExt, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use ts_rs::TS;

use crate::{
    main_server::{GlobalConfig, GlobalConfigUpdate, MainServer},
    serial::SerialPortManager,
    skeleton::{Bone, BoneLocation},
    tracker::TrackerRef,
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Serialize, TS)]
#[serde(tag = "type")]
pub enum WebsocketServerMessage<'a> {
    TrackerUpdate {
        trackers: &'a HashMap<Arc<str>, TrackerRef>,
    },
    InitialState {
        config: &'a GlobalConfig,
        #[ts(optional)]
        port_name: Option<Box<str>>,
        default_config: GlobalConfig,
    },
    SkeletonUpdate {
        bones: &'a HashMap<BoneLocation, Bone>,
    },
    SerialLog {
        log: &'a str,
    },
    SerialPortChanged {
        #[ts(optional)]
        port_name: Option<Box<str>>,
    },
    Error {
        error: &'a str,
    },
}

// Receieved from client
#[derive(Deserialize, TS)]
#[serde(tag = "type")]
pub enum WebsocketClientMessage {
    SerialSend { data: String },
    RemoveTracker { id: Box<String> },
    UpdateConfig { config: GlobalConfigUpdate },
}

pub struct WebsocketServer {
    listener: TcpListener,
    ws_stream: Option<WebSocketStream<TcpStream>>,
    serial_manager: SerialPortManager,
    time_last_send_messages: Instant,
}

impl WebsocketServer {
    pub async fn new() -> anyhow::Result<Self> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT));
        log::info!("Started websocket server on {address}");
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            time_last_send_messages: Instant::now(),
            listener,
            ws_stream: None,
            serial_manager: SerialPortManager::default(),
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        if let Some(ws_stream) = self.ws_stream.as_mut() {
            // Get new data from websocket
            match ws_stream.next().now_or_never() {
                Some(Some(Ok(message))) => {
                    if let Ok(text) = message.to_text() {
                        self.handle_ws_message(text, main)?;
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
        } else {
            self.try_receive_ws_connection(main).await?;
        }

        // Limit ws sending to 60 times per second
        if self.time_last_send_messages.elapsed() > Duration::from_millis(1000 / 60) {
            self.send_ws_messages(main).await?;
            self.time_last_send_messages = Instant::now();
        }

        Ok(())
    }

    async fn try_receive_ws_connection(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        match self.listener.accept().now_or_never() {
            Some(Ok((stream, peer_addr))) => {
                let mut ws_stream = tokio_tungstenite::accept_async(stream).await?;

                let message = WebsocketServerMessage::InitialState {
                    config: &main.config,
                    port_name: self.serial_manager.port_name(),
                    default_config: GlobalConfig::default(),
                };
                feed_ws_message(&mut ws_stream, message).await?;

                ws_stream.flush().await?;
                self.ws_stream = Some(ws_stream);
                log::info!("Websocket client connected at {peer_addr}");
                Ok(())
            }
            Some(Err(e)) => Err(e)?,
            None => Ok(()),
        }
    }

    async fn send_ws_messages(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        let ws_stream = match self.ws_stream.as_mut() {
            Some(s) => s,
            None => return Ok(()),
        };

        // Send the serial stuff
        if self.serial_manager.check_port().await {
            let message = WebsocketServerMessage::SerialPortChanged {
                port_name: self.serial_manager.port_name(),
            };
            feed_ws_message(ws_stream, message).await?;
        }

        if let Some(log) = self.serial_manager.read_line() {
            feed_ws_message(ws_stream, WebsocketServerMessage::SerialLog { log }).await?;
        }

        if let Some(error) = main.updates.error.as_ref() {
            feed_ws_message(ws_stream, WebsocketServerMessage::Error { error }).await?;
        }

        let message = WebsocketServerMessage::TrackerUpdate {
            trackers: &main.trackers,
        };
        feed_ws_message(ws_stream, message).await?;

        let message = WebsocketServerMessage::SkeletonUpdate {
            bones: &main.skeleton_manager.bones,
        };
        feed_ws_message(ws_stream, message).await?;

        ws_stream.flush().await?;
        Ok(())
    }

    fn handle_ws_message(&mut self, message: &str, main: &mut MainServer) -> anyhow::Result<()> {
        if message.is_empty() {
            return Ok(());
        }

        match serde_json::from_str(message)? {
            WebsocketClientMessage::SerialSend { data } => {
                log::info!("Writing {data:?} to port");
                self.serial_manager.write(data.as_bytes())?;
            }
            WebsocketClientMessage::RemoveTracker { id } => {
                if let Some(tracker) = main.trackers.get(id.as_str()) {
                    tracker.lock().unwrap().internal.to_be_removed = true;
                }
            }
            WebsocketClientMessage::UpdateConfig { config } => {
                main.updates.config = Some(config);
            }
        }

        Ok(())
    }
}

async fn feed_ws_message<'a>(
    ws_stream: &mut WebSocketStream<TcpStream>,
    ws_message: WebsocketServerMessage<'a>,
) -> anyhow::Result<()> {
    if let Ok(text) = serde_json::to_string(&ws_message) {
        ws_stream.feed(Message::Text(text)).await?;
    }

    Ok(())
}
