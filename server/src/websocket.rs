use futures_util::{
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use std::{net::Ipv4Addr, sync::Arc};
use tokio::sync::RwLock;
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

use crate::{serial::write_serial, ServerState};

pub const WEBSOCKET_PORT: u16 = 8298;

pub async fn start_server(state: Arc<RwLock<ServerState>>) {
    let websocket = warp::ws()
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, state| ws.on_upgrade(|ws| on_connect(ws, state)));

    warp::serve(websocket)
        .run((Ipv4Addr::LOCALHOST, WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket, state: Arc<RwLock<ServerState>>) {
    WebsocketConnection::new(ws, state).handle().await
}

struct WebsocketConnection {
    state: Arc<RwLock<ServerState>>,
    ws_tx: SplitSink<WebSocket, Message>,
    ws_rx: SplitStream<WebSocket>,
}

impl WebsocketConnection {
    pub fn new(ws: WebSocket, state: Arc<RwLock<ServerState>>) -> Self {
        log::info!("Websocket client connected");
        let (ws_tx, ws_rx) = ws.split();
        Self {
            state,
            ws_tx,
            ws_rx,
        }
    }

    async fn handle(&mut self) {
        while let Some(ws_result) = self.ws_rx.next().await {
            let msg = match ws_result {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("Websocket error: {e}");
                    break;
                }
            };

            if let Ok(string) = msg.to_str() {
                log::info!("Got from websocket: {string}");
                if let Err(error) = self.handle_websocket_message(string) {
                    log::error!("{error}");
                    self.send_websocket_message(WebsocketMessage::Error { error })
                        .await;
                }
            }
        }
    }

    fn handle_websocket_message(&self, string: &str) -> Result<(), String> {
        let message = serde_json::from_str(string).map_err(|e| e.to_string())?;

        match message {
            WebsocketMessage::Wifi { ssid, password } => {
                if ssid.len() > 32 || password.len() > 64 {
                    return Err("SSID or password too long".to_string());
                }

                write_serial(format!("WIFI\0{ssid}\0{password}").as_bytes())?;
            }
            WebsocketMessage::FactoryReset => {
                write_serial("FACTORY-RESET".as_bytes())?;
            }
            WebsocketMessage::Error { error } => log::error!("Error from client: {error}"),
            WebsocketMessage::RequestTrackerData { id } => todo!(),
        }

        Ok(())
    }
    async fn send_websocket_message(&mut self, message: WebsocketMessage) {
        if let Ok(string) = serde_json::to_string(&message) {
            self.ws_tx.send(Message::text(string)).await.ok();
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WebsocketMessage {
    Wifi { ssid: String, password: String },
    Error { error: String },
    FactoryReset,
    RequestTrackerData { id: u8 },
}
