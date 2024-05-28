#![feature(concat_bytes)]

mod math;
mod serial;
mod udp_packet;
mod udp_server;

use std::{net::SocketAddr, sync::Arc};

use futures_util::{lock::Mutex, StreamExt, TryFutureExt};
use warp::{filters::ws::WebSocket, Filter};

pub const WEBSOCKET_PORT: u16 = 8298;

pub struct Device {
    pub address: SocketAddr,
}

#[derive(Default)]
pub struct ServerState {
    devices: Vec<Device>,
}

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .filter_module("mycap", log::LevelFilter::Trace)
        .init();
}

pub async fn start_server() {
    let state = Arc::new(Mutex::new(ServerState::default()));

    tokio::spawn(udp_server::start_server().inspect_err(|e| log::error!("UDP server error: {e}")));

    let websocket = warp::ws()
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, tx| ws.on_upgrade(on_connect));

    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket) {
    let (ws_tx, mut ws_rx) = ws.split();

    log::info!("Websocket client connected");
    while let Some(msg) = ws_rx.next().await.and_then(|result| {
        result
            .inspect_err(|e| log::error!("websocket error: {e}"))
            .ok()
    }) {
        if let Ok(msg) = msg.to_str() {
            log::info!("Got from websocket: {msg}");
            if let Ok(message) = serde_json::from_str(msg)
                .inspect_err(|e| log::error!("Error decoding websocket  message: {e}"))
            {
                handle_websocket_message(message);
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WebsocketMessage {
    Serial { command: String },
}

fn handle_websocket_message(message: WebsocketMessage) {
    match message {
        WebsocketMessage::Serial { command } => {
            serial::write_serial(command.as_bytes())
                .inspect_err(|e| log::error!("Failed to write to serial port: {e}"))
                .ok();
        }
    }
}
