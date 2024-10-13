mod config;
mod looper;
mod main_server;
mod osc;
mod record;
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
        Ok(config_update) => {
            if let Err(err) = main.apply_config(config_update, &mut modules).await {
                log::warn!("Failed to apply part of config: {err}");
            }
        }
        Err(err) => log::warn!("Failed to load config: {err}"),
    }

    let mut looper = Looper::default();

    loop {
        looper.start_loop();

        let result = main.update(&mut modules).await;
        main.updates = Default::default();

        if let Err(err) = result {
            log::error!("{err:?}");
            main.updates.error = Some(err.to_string().into());
        }

        looper.loop_end_wait().await;
    }
}
