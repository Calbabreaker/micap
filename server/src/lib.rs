mod serial;
mod udp_protocol;

use std::sync::Arc;

use futures_util::{lock::Mutex, stream::SplitSink, StreamExt};
use warp::{
    filters::ws::{Message, WebSocket},
    Filter,
};

pub const WEBSOCKET_PORT: u16 = 8298;

#[derive(Default)]
struct ServerState {
    websocket_tx: Option<SplitSink<WebSocket, Message>>,
}

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub async fn start_server() {
    let state = Arc::new(Mutex::new(ServerState::default()));

    tokio::spawn(udp_protocol::bind_udp_socket(state.clone()));

    let websocket = warp::ws()
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, state| ws.on_upgrade(|socket| on_connect(socket, state)));

    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket, state: Arc<Mutex<ServerState>>) {
    let (ws_tx, mut ws_rx) = ws.split();
    {
        state.lock().await.websocket_tx = Some(ws_tx);
    }

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
