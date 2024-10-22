use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    osc::{vmc_connector::VmcConfig, vrchat_connector::VrChatConfig},
    skeleton::SkeletonConfig,
    tracker::TrackerConfig,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct InterfaceConfig {
    pub hide_in_system_tray: bool,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct GlobalConfig {
    pub trackers: HashMap<Arc<str>, TrackerConfig>,
    pub vmc: VmcConfig,
    pub vrchat: VrChatConfig,
    pub skeleton: SkeletonConfig,
    pub interface: InterfaceConfig,
}

impl GlobalConfig {
    pub fn load() -> anyhow::Result<GlobalConfig> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Loading from {path:?}");
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Saving to {path:?}");
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}

pub fn get_config_dir() -> anyhow::Result<PathBuf> {
    let env_dir = std::env::var("MICAP_CONFIG_DIR").ok().map(PathBuf::from);
    let default_dir = dirs::config_dir().map(|p| p.join("micap"));

    let config_folder = env_dir
        .or(default_dir)
        .ok_or_else(|| anyhow::anyhow!("Failed to get a config directory"))?;

    if !config_folder.exists() {
        std::fs::create_dir_all(&config_folder)?;
    }
    Ok(config_folder)
}
