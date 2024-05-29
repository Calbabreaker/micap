use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

use crate::ServerState;

pub const WEBSOCKET_PORT: u16 = 8298;

pub async fn start_server(state: Arc<RwLock<ServerState>>) {
    let websocket = warp::ws()
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, state| ws.on_upgrade(|ws| on_connect(ws, state)));

    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
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
            ws_rx,
            ws_tx,
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

            if let Ok(msg) = msg.to_str() {
                log::info!("Got from websocket: {msg}");
                if let Ok(message) = serde_json::from_str(msg)
                    .inspect_err(|e| log::error!("Error decoding websocket message: {e}"))
                {
                    self.handle_websocket_message(message);
                }
            }
        }
    }

    fn handle_websocket_message(&self, message: WebsocketMessage) {
        match message {
            WebsocketMessage::Wifi { ssid, password } => {
                let command = format!("WIFI\0{ssid}\0{password}");
                crate::serial::write_serial(command.as_bytes())
                    .inspect_err(|e| log::error!("Failed to write to serial port: {e}"))
                    .ok();
            }
            WebsocketMessage::Error { message } => log::error!("Error from client: {message}"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WebsocketMessage {
    Wifi { ssid: String, password: String },
    Error { message: String },
}
