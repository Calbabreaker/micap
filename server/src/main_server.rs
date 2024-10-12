use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::{GlobalConfig, GlobalConfigUpdate},
    osc::{vmc_connector::VmcConnector, vrchat_connector::VrChatConnector},
    skeleton::SkeletonManager,
    tracker::*,
    udp::server::{UdpServer, UDP_PORT},
    websocket::{WebsocketServer, WEBSOCKET_PORT},
};

pub struct ServerModules {
    pub udp_server: UdpServer,
    pub vmc_connector: VmcConnector,
    pub vrchat_connector: VrChatConnector,
    pub websocket_server: WebsocketServer,
}

impl ServerModules {
    pub async fn new() -> anyhow::Result<Self> {
        fn get_context(server: &str, port: u16) -> String {
            format!("Failed to start {server} server!\nNote: Port {port} needs to be open")
        }

        Ok(Self {
            websocket_server: WebsocketServer::new()
                .await
                .with_context(|| get_context("Websocket", WEBSOCKET_PORT))?,
            udp_server: UdpServer::new()
                .await
                .with_context(|| get_context("UDP", UDP_PORT))?,
            vmc_connector: VmcConnector::new().await?,
            vrchat_connector: VrChatConnector::new().await?,
        })
    }
}

#[derive(Default)]
pub struct ServerUpdates {
    pub error: Option<Box<str>>,
    pub config: Option<GlobalConfigUpdate>,
}

#[derive(Default)]
pub struct MainServer {
    // Maps a tracker id to a tracker
    pub trackers: HashMap<Arc<str>, TrackerRef>,
    pub skeleton_manager: SkeletonManager,
    pub config: GlobalConfig,
    pub updates: ServerUpdates,
}

impl MainServer {
    pub async fn update(&mut self, modules: &mut ServerModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        if let Some(config_update) = self.updates.config.take() {
            self.apply_config(config_update, modules).await?;
            self.config.save()?;
        }

        self.skeleton_manager.update();
        modules.vmc_connector.update(self).await?;

        if let Some(removed_id) = self.upkeep_trackers().await {
            // Remove the tracker when is set to remove
            self.trackers.remove(&removed_id);

            if self.config.trackers.remove(&removed_id).is_some() {
                self.updates.config = Some(GlobalConfigUpdate {
                    // We're not changing anything so set to none
                    trackers: Some(HashMap::new()),
                    ..Default::default()
                });
            }
        }

        Ok(())
    }

    // Returns a tracker id if that tracker should be removed
    async fn upkeep_trackers(&mut self) -> Option<Arc<str>> {
        for (id, tracker) in &self.trackers {
            let mut tracker = tracker.lock().unwrap();
            tracker.internal.was_updated = false;
            if tracker.internal.to_be_removed {
                return Some(id.clone());
            }
        }

        None
    }

    pub async fn apply_config(
        &mut self,
        config: GlobalConfigUpdate,
        modules: &mut ServerModules,
    ) -> anyhow::Result<()> {
        // Check what update was set and apply specifically to each module
        if let Some(config) = config.skeleton {
            self.skeleton_manager.apply_skeleton_config(&config);
            self.config.skeleton = config;
        }

        if let Some(config) = config.vmc {
            modules.vmc_connector.apply_config(&config).await?;
            self.config.vmc = config;
        }

        if let Some(config) = config.vrchat {
            modules.vrchat_connector.apply_config(&config).await?;
            self.config.vrchat = config;
        }

        if let Some(mut tracker_configs) = config.trackers {
            // Set all the tracker configs provided
            for (id, config_update) in tracker_configs.drain() {
                // Insert tracker if not exist (usually in first load)
                if !self.trackers.contains_key(&id) {
                    self.trackers.insert(id.clone(), TrackerRef::default());
                }

                self.config.trackers.insert(id, config_update);
            }

            self.skeleton_manager
                .apply_tracker_config(&self.config.trackers, &self.trackers);
        }

        modules.websocket_server.send_config(&self.config).await?;
        Ok(())
    }

    pub fn add_tracker(&mut self, id: &Arc<str>) -> Option<TrackerRef> {
        if !self.trackers.contains_key(id) {
            let tracker = TrackerRef::default();
            // Note: we only set the config once the user does
            self.trackers.insert(id.clone(), tracker.clone());
        }

        self.trackers.get(id).cloned()
    }
}
