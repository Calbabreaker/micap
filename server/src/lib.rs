mod main_server;
mod serial;
mod tracker;
pub mod udp;
mod vmc;
mod websocket;

pub use websocket::WEBSOCKET_PORT;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::main_server::MainServer;

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Warn)
        .filter_module("micap", log::LevelFilter::Info)
        .parse_env("RUST_LOG")
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
