use futures_util::{FutureExt, SinkExt, StreamExt};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{
    main_server::MainServer,
    serial::write_serial,
    tracker::{TrackerConfig, TrackerData, TrackerInfo},
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum WebsocketServerMessage {
    TrackerInfo { index: usize, info: TrackerInfo },
    TrackerData { index: usize, data: TrackerData },
    Error { error: String },
}
// Receieved from client
#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum WebsocketClientMessage {
    Wifi { ssid: String, password: String },
    FactoryReset,
    RemoveTracker { index: usize },
    UpdateTrackerConfig { index: usize, config: TrackerConfig },
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
                            main.new_errors.push(error.to_string());
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
            .chain(main.new_errors.iter().map(|error| {
                // Errors emmited
                WebsocketServerMessage::Error {
                    error: error.clone(),
                }
            }))
            .chain(main.tracker_info_updated_indexs.iter().map(|index| {
                WebsocketServerMessage::TrackerInfo {
                    index: *index,
                    info: main.trackers[*index].info.clone(),
                }
            }))
            .chain(main.trackers.iter().enumerate().map(|(index, tracker)| {
                // Data from trackers
                WebsocketServerMessage::TrackerData {
                    index,
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
        for (index, tracker) in main.trackers.iter().enumerate() {
            let string = serde_json::to_string(&WebsocketServerMessage::TrackerInfo {
                index,
                info: tracker.info.clone(),
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
        WebsocketClientMessage::RemoveTracker { index } => {
            main.trackers[index].info.removed = true;
            main.tracker_info_updated(index);
        }
        WebsocketClientMessage::UpdateTrackerConfig { index, config } => {
            main.update_tracker_config(index, config)?;
        }
    }

    Ok(())
}
