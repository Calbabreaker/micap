#![feature(concat_bytes)]

mod math;
mod serial;
mod udp_packet;
mod udp_server;

use std::sync::Arc;

use futures_util::{lock::Mutex, stream::SplitSink, FutureExt, StreamExt, TryFutureExt};
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Default)]
struct ServerState {}

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub async fn start_server() {
    let state = Arc::new(Mutex::new(ServerState::default()));

    tokio::spawn(udp_server::start_server());

    let websocket = warp::ws().map(|ws: warp::ws::Ws| ws.on_upgrade(on_connect));

    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket) {
    let (ws_tx, mut ws_rx) = ws.split();

    log::info!("Websocket client connected");
    while let Some(message) = ws_rx.next().await.and_then(|result| {
        result
            .inspect_err(|e| log::error!("websocket error: {e}"))
            .ok()
    }) {
        if let Ok(message) = message.to_str() {
            let mut args = message.split(":");
            log::info!("{:?}", message);

            if args.next() == Some("SERIAL") {
                if let Some(command) = args.next() {
                    serial::write_serial(command.as_bytes())
                        .inspect_err(|e| log::error!("Failed to write to serial port: {e}"))
                        .ok();
                }
            }
        }
    }
}
