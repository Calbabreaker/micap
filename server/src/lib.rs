mod main_server;
mod serial;
mod tracker;
pub mod udp;
mod vmc;
mod websocket;

pub use websocket::WEBSOCKET_PORT;

use std::time::{Duration, Instant};

use crate::main_server::{MainServer, SubModules, UpdateEvent};

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

    if let Err(error) = main.load_config() {
        log::warn!("Failed to load config: {error:?}");
    }

    let mut last_serial_scan_time = Instant::now();

    loop {
        if last_serial_scan_time.elapsed() > Duration::from_secs(5) {
            main.serial_manager.scan_ports().ok();
            last_serial_scan_time = Instant::now();
        }

        let update_start_time = Instant::now();

        let result = main.update(&mut modules).await;
        main.updates.clear();

        if let Err(err) = result {
            log::error!("{err:?}");
            main.updates.push(UpdateEvent::Error {
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
