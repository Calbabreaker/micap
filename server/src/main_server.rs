use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    osc::vmc_connector::{VmcConfig, VmcConnector},
    skeleton::{SkeletonConfig, SkeletonManager},
    tracker::*,
    udp::server::UdpServer,
    websocket::WebsocketServer,
};

pub struct ServerModules {
    pub udp_server: UdpServer,
    pub vmc_connector: VmcConnector,
    pub websocket_server: WebsocketServer,
}

impl ServerModules {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            websocket_server: WebsocketServer::new()
                .await
                .context("Failed to start websocket server")?,
            udp_server: UdpServer::new()
                .await
                .context("Failed to start UDP server")?,
            vmc_connector: VmcConnector::new().await?,
        })
    }
}

#[derive(Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct GlobalConfig {
    pub trackers: HashMap<String, TrackerConfig>,
    pub vmc: VmcConfig,
    pub skeleton: SkeletonConfig,
}

#[derive(Serialize, Deserialize, TS)]
pub struct GlobalConfigUpdate {
    // Note: every field as optional to allow for specific config updates
    #[ts(optional)]
    pub trackers: Option<HashMap<String, TrackerConfig>>,
    #[ts(optional)]
    pub vmc: Option<VmcConfig>,
    #[ts(optional)]
    pub skeleton: Option<SkeletonConfig>,
}

#[derive(Default)]
pub struct ServerUpdates {
    pub error: Option<String>,
    pub config: Option<GlobalConfigUpdate>,
}

#[derive(Default)]
pub struct MainServer {
    // Maps a tracker id to a tracker
    pub trackers: HashMap<String, TrackerRef>,
    pub skeleton_manager: SkeletonManager,
    pub config: GlobalConfig,
    pub updates: ServerUpdates,
}

impl MainServer {
    pub fn load_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Loading from {path:?}");
        let text = std::fs::read_to_string(path)?;
        let config = serde_json::from_str::<GlobalConfigUpdate>(&text)?;

        if let Some(tracker_config) = config.trackers.as_ref() {
            for id in tracker_config.keys() {
                self.trackers.insert(id.clone(), TrackerRef::default());
            }
        }

        self.updates.config = Some(config);

        Ok(())
    }

    pub fn save_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Saving to {path:?}");
        let text = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, text)?;

        Ok(())
    }

    pub async fn update(&mut self, modules: &mut ServerModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        if let Some(config_update) = self.updates.config.take() {
            self.apply_config(config_update, modules).await?;
        }

        self.skeleton_manager.update();
        modules.vmc_connector.update(self).await?;

        if let Some(removed_id) = self.upkeep_trackers().await {
            // Remove the tracker when is set to remove
            self.trackers.remove(&removed_id);
            if self.config.trackers.remove(&removed_id).is_some() {
                self.save_config()?;
            }
        }

        Ok(())
    }

    // Returns a tracker id if that tracker should be removed
    async fn upkeep_trackers(&mut self) -> Option<String> {
        for (id, tracker) in &self.trackers {
            let tracker = tracker.lock().unwrap();
            if tracker.internal.to_be_removed {
                return Some(id.clone());
            }
        }

        None
    }

    async fn apply_config(
        &mut self,
        config: GlobalConfigUpdate,
        modules: &mut ServerModules,
    ) -> anyhow::Result<()> {
        if let Some(config) = config.trackers {
            self.skeleton_manager
                .apply_tracker_config(&config, &self.trackers);
            self.config.trackers = config;
        }

        if let Some(config) = config.skeleton {
            self.skeleton_manager.apply_skeleton_config(&config);
            self.config.skeleton = config;
        }

        if let Some(config) = config.vmc {
            modules.vmc_connector.apply_config(&config).await?;
            self.config.vmc = config;
        }

        self.save_config()?;
        Ok(())
    }

    pub fn add_tracker(&mut self, id: String) {
        if !self.trackers.contains_key(&id) {
            let tracker = TrackerRef::default();
            // Note: we only set the config once the user does
            self.trackers.insert(id.clone(), tracker.clone());
        }
    }
}

pub fn get_config_dir() -> anyhow::Result<PathBuf> {
    let config_folder = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get a config directory"))?
        .join("micap");

    if !config_folder.is_dir() {
        std::fs::create_dir_all(&config_folder)?;
    }
    Ok(config_folder)
}
