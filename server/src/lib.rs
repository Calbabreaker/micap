#![allow(clippy::needless_return)]

mod config;
mod looper;
mod main_server;
mod osc;
mod serial;
mod skeleton;
pub mod tracker;
pub mod udp;
pub mod websocket;

#[cfg(test)]
mod test;

use crate::{
    config::GlobalConfig,
    looper::Looper,
    main_server::{MainServer, ServerModules},
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

    match GlobalConfig::load() {
        Ok(config_update) => main.apply_config(config_update, &mut modules).await?,
        Err(err) => log::warn!("Failed to load config: {err}"),
    }

    let mut looper = Looper::default();

    loop {
        looper.start_loop();

        let result = main.update(&mut modules).await;
        main.updates = Default::default();

        if let Err(err) = result {
            log::error!("{err:?}");
            main.updates.error = Some(Box::from(err.to_string()));
        }

        looper.end_loop_and_wait().await;
    }
}
