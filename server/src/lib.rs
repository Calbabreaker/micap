mod looper;
mod main_server;
mod osc;
mod serial;
mod skeleton;
pub mod tracker;
pub mod udp;
pub mod websocket;

use crate::{
    looper::Looper,
    main_server::{MainServer, ServerEvent, ServerModules},
};

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Warn)
        .filter_module("micap", log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
}

pub async fn start_server() -> anyhow::Result<()> {
    // Seperate out  main and modules to prevent multiple borrow
    let mut main = MainServer::default();
    let mut modules = ServerModules::new().await?;

    if let Err(error) = main.load_config() {
        log::warn!("Failed to load config: {error:?}");
    }

    let mut looper = Looper::default();

    loop {
        looper.start_loop();

        let result = main.update(&mut modules).await;
        main.events.clear();

        if let Err(err) = result {
            log::error!("{err:?}");
            main.events.push(ServerEvent::Error {
                error: err.root_cause().to_string(),
            });
        }

        looper.end_loop_and_wait().await;
    }
}
