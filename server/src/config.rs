use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    osc::{vmc_connector::VmcConfig, vrchat_connector::VrChatConfig},
    skeleton::SkeletonConfig,
    tracker::TrackerConfig,
};

#[derive(Debug, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct GlobalConfig {
    pub trackers: HashMap<Arc<str>, TrackerConfig>,
    pub vmc: VmcConfig,
    pub vrchat: VrChatConfig,
    pub skeleton: SkeletonConfig,
}

#[derive(Default, Debug, Serialize, Deserialize, TS)]
pub struct GlobalConfigUpdate {
    // Note: every field as optional to allow for specific config updates
    pub trackers: Option<HashMap<Arc<str>, TrackerConfig>>,
    pub vmc: Option<VmcConfig>,
    pub vrchat: Option<VrChatConfig>,
    pub skeleton: Option<SkeletonConfig>,
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

    pub fn into_update(self) -> GlobalConfigUpdate {
        GlobalConfigUpdate {
            trackers: Some(self.trackers),
            vmc: Some(self.vmc),
            vrchat: Some(self.vrchat),
            skeleton: Some(self.skeleton),
        }
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
