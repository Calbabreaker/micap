mod main_server;
mod serial;
mod tracker;
pub mod udp;
mod vmc;
mod websocket;

pub use websocket::WEBSOCKET_PORT;

use std::time::{Duration, Instant};

use crate::main_server::{MainServer, SubModules};

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Warn)
        .filter_module("micap", log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
}

const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 50);

pub async fn start_server() -> anyhow::Result<()> {
    // Seperate out  main and modules to prevent multiple borrow
    let mut main = MainServer::default();
    let mut modules = SubModules::new().await?;

    loop {
        let tick_start_time = Instant::now();

        main.update(&mut modules).await?;

        let post_delta = tick_start_time.elapsed();
        if let Some(sleep_duration) = TARGET_LOOP_DELTA.checked_sub(post_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {post_delta:?} which is longer than target {TARGET_LOOP_DELTA:?}"
            );
        }
    }
}
