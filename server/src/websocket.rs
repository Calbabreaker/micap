use futures_util::{FutureExt, SinkExt, StreamExt};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    main_server::{MainServer, UpdateEvent},
    serial::write_serial,
    tracker::{TrackerConfig, TrackerData, TrackerInfo},
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum WebsocketServerMessage {
    TrackerInfo {
        id: String,
        // Remove tracker if info is None
        info: Option<TrackerInfo>,
    },
    TrackerData {
        id: String,
        data: TrackerData,
    },
    Error {
        error: String,
    },
}
// Receieved from client
#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
    RemoveTracker { id: String },
    UpdateTrackerConfig { id: String, config: TrackerConfig },
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

        self.ws_streams.retain_mut(|ws| {
            // Use retain_mut to remove ws_streams when they are closed (returning false removes it)
            match ws.next().now_or_never() {
                Some(Some(Ok(message))) => {
                    if let Ok(text) = message.to_text() {
                        if let Err(error) = handle_websocket_message(text, main) {
                            log::error!("Failed to handle websocket message: {error}");
                            main.updates
                                .push(UpdateEvent::TrackerRemove(error.to_string()));
                        }
                    }

                    true
                }
                Some(Some(Err(e))) => {
                    log::error!("Websocket error: {e:?}");
                    false
                }
                Some(None) => {
                    log::info!("Websocket disconnected");
                    false
                }
                None => true,
            }
        });

        // Get list of messages to send
        let new_messages = std::iter::empty()
            .chain(main.updates.iter().map(|update| match update {
                UpdateEvent::TrackerInfoUpdate(id) => WebsocketServerMessage::TrackerInfo {
                    id: id.clone(),
                    info: Some(main.trackers[id].info.clone()),
                },
                UpdateEvent::TrackerRemove(id) => WebsocketServerMessage::TrackerInfo {
                    id: id.clone(),
                    info: None,
                },
                UpdateEvent::NewError(error) => WebsocketServerMessage::Error {
                    error: error.clone(),
                },
            }))
            .chain(main.trackers.iter().map(|(id, tracker)| {
                // Data from trackers
                WebsocketServerMessage::TrackerData {
                    id: id.clone(),
                    data: tracker.data.clone(),
                }
            }))
            .map(|message| serde_json::to_string(&message).unwrap())
            .collect::<Vec<_>>();

        for ws_stream in self.ws_streams.iter_mut() {
            for message in new_messages.iter() {
                ws_stream.feed(Message::text(message)).await.ok();
            }

            if !new_messages.is_empty() {
                ws_stream.flush().await?;
            }
        }

        Ok(())
    }

    async fn on_connect(
        &mut self,
        mut ws_stream: WebSocketStream<TcpStream>,
        main: &mut MainServer,
    ) -> anyhow::Result<()> {
        for (id, tracker) in main.trackers.iter() {
            let string = serde_json::to_string(&WebsocketServerMessage::TrackerInfo {
                id: id.clone(),
                info: Some(tracker.info.clone()),
            })?;
            ws_stream.feed(Message::text(string)).await?;
        }

        ws_stream.flush().await?;
        self.ws_streams.push(ws_stream);
        Ok(())
    }
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

            write_serial(format!("Wifi\0{ssid}\0{password}\n").as_bytes())?;
        }
        WebsocketClientMessage::FactoryReset => {
            write_serial(b"FactoryReset\n")?;
        }
        WebsocketClientMessage::RemoveTracker { id } => {
            main.remove_tracker(&id);
            main.save_config()?;
        }
        WebsocketClientMessage::UpdateTrackerConfig { id, config } => {
            if let Some(tracker) = main.tracker_info_update(&id) {
                tracker.info.config = config;
                main.save_config()?;
            }
        }
    }

    Ok(())
}
