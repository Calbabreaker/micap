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

#[derive(PartialEq, Serialize, TS)]
#[serde(tag = "type")]
pub enum ServerEvent {
    Error { error: String },
    ConfigUpdate,
}

#[derive(Default, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct GlobalConfig {
    pub trackers: HashMap<String, TrackerConfig>,
    pub vmc: VmcConfig,
    pub skeleton: SkeletonConfig,
}

pub type TrackerRef = std::sync::Arc<std::sync::Mutex<Tracker>>;

#[derive(Default)]
pub struct MainServer {
    // Maps a tracker id to a tracker
    pub trackers: HashMap<String, TrackerRef>,
    /// Contains list of update event emited in the middle of a loop
    /// Gets cleared at the end of the loop
    pub events: Vec<ServerEvent>,
    pub skeleton_manager: SkeletonManager,
    pub config: GlobalConfig,
}

impl MainServer {
    pub fn load_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Loading from {path:?}");
        let text = std::fs::read_to_string(path)?;
        let config = serde_json::from_str::<GlobalConfig>(&text)?;

        for id in config.trackers.keys() {
            self.trackers.insert(id.clone(), TrackerRef::default());
        }

        self.config = config;
        self.events.push(ServerEvent::ConfigUpdate);
        Ok(())
    }

    pub fn save_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Saving to {path:?}");
        let text = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, text)?;
        self.events.push(ServerEvent::ConfigUpdate);

        Ok(())
    }

    pub async fn update(&mut self, modules: &mut ServerModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        if self.events.contains(&ServerEvent::ConfigUpdate) {
            self.skeleton_manager
                .apply_config(&self.config, &self.trackers);
            modules.vmc_connector.apply_config(&self.config).await?;
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
