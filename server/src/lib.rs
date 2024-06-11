#![feature(concat_bytes)]

mod main_server;
mod math;
mod serial;
mod tracker;
mod udp_packet;
mod udp_server;
mod websocket;

pub use udp_server::UDP_PORT;
pub use websocket::WEBSOCKET_PORT;

use futures_util::TryFutureExt;
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

pub async fn start_server() {
    let state = Arc::new(RwLock::new(MainServer::default()));

    tokio::spawn(
        udp_server::start_server(state.clone())
            .inspect_err(|e| log::error!("UDP server error: {e}")),
    );

    websocket::start_server(state.clone()).await;
}
