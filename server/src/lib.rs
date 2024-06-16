mod main_server;
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
    let main = Arc::new(RwLock::new(MainServer::default()));

    tokio::try_join!(
        flatten(tokio::spawn(websocket::start_server(main.clone()))),
        flatten(tokio::spawn(main_server::start_server(main)))
    )?;

    Ok(())
}

async fn flatten(handle: tokio::task::JoinHandle<anyhow::Result<()>>) -> anyhow::Result<()> {
    handle.await?
}
