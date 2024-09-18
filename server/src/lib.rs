mod main_server;
mod osc;
mod serial;
mod skeleton;
pub mod tracker;
pub mod udp;
pub mod websocket;

use std::time::{Duration, Instant};

use crate::main_server::{MainServer, ServerEvent, ServerModules};

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Warn)
        .filter_module("micap", log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
}

const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 60);

pub async fn start_server() -> anyhow::Result<()> {
    // Seperate out  main and modules to prevent multiple borrow
    let mut main = MainServer::default();
    let mut modules = ServerModules::new().await?;

    if let Err(error) = main.load_config() {
        log::warn!("Failed to load config: {error:?}");
    }

    loop {
        let update_start_time = Instant::now();

        let result = main.update(&mut modules).await;
        main.events.clear();

        if let Err(err) = result {
            log::error!("{err:?}");
            main.events.push(ServerEvent::Error {
                error: err.root_cause().to_string(),
            });
        }

        let post_delta = update_start_time.elapsed();
        if let Some(sleep_duration) = TARGET_LOOP_DELTA.checked_sub(post_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {post_delta:?} which is longer than target {TARGET_LOOP_DELTA:?}"
            );
        }
    }
}
