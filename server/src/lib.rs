mod main_server;
mod math;
mod serial;
mod tracker;
mod udp_packet;
mod udp_server;
mod websocket;

pub use udp_server::UDP_PORT;
pub use websocket::WEBSOCKET_PORT;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::main_server::MainServer;

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Warn)
        .filter_module("mycap", log::LevelFilter::Trace)
        .init();
}

pub async fn start_server() -> anyhow::Result<()> {
    let state = Arc::new(RwLock::new(MainServer::default()));

    let mut join_set = tokio::task::JoinSet::new();

    join_set.spawn(udp_server::start_server(state.clone()));
    join_set.spawn(websocket::start_server(state.clone()));

    while let Some(result) = join_set.join_next().await {
        result??;
    }

    Ok(())
}
