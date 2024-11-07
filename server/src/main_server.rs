use std::{collections::HashMap, sync::Arc};

use anyhow::Context;

use crate::{
    config::GlobalConfig,
    osc::{vmc_connector::VmcConnector, vrchat_connector::VrChatConnector},
    record::MotionRecorder,
    skeleton::{BoneLocation, SkeletonManager},
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
            format!("Failed to start {server} server!\nNote: Port {port} needs to be open, check if another instance is already runnning")
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
    pub config: Option<GlobalConfig>,
}

#[derive(Default)]
pub struct MainServer {
    // Maps a tracker id to a tracker
    pub trackers: HashMap<Arc<str>, TrackerRef>,
    pub skeleton_manager: SkeletonManager,
    pub motion_recorder: MotionRecorder,
    pub config: GlobalConfig,
    pub updates: ServerUpdates,
}

impl MainServer {
    pub async fn update(&mut self, modules: &mut ServerModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        if let Some(config) = self.updates.config.take() {
            self.config = config;
            self.apply_config(modules).await?;
            self.config.save()?;
        }

        self.skeleton_manager.update();
        modules.vmc_connector.update(self).await?;

        self.motion_recorder.update(&self.skeleton_manager);

        if let Some(removed_id) = self.upkeep_trackers().await {
            // Remove the tracker when is set to remove
            self.trackers.remove(&removed_id);

            if self.config.trackers.remove(&removed_id).is_some() {
                self.apply_config(modules).await?;
            }
        }

        Ok(())
    }

    // Returns a tracker id if that tracker should be removed
    async fn upkeep_trackers(&mut self) -> Option<Arc<str>> {
        for (id, tracker) in &self.trackers {
            let mut tracker = tracker.lock().unwrap();
            tracker.internal.was_updated = false;

            if tracker.info().to_be_removed {
                return Some(id.clone());
            }
        }

        None
    }

    pub async fn apply_config(&mut self, modules: &mut ServerModules) -> anyhow::Result<()> {
        self.config.skeleton.update_height();

        let config = &self.config;

        // Set all the tracker configs provided
        for (id, tracker_config) in config.trackers.iter() {
            // Insert tracker if not exist (usually in first load)
            if !self.trackers.contains_key(id) {
                self.trackers.insert(id.clone(), TrackerRef::default());
            }

            if let Some(location) = tracker_config.location {
                self.trackers[id].lock().unwrap().set_mount_offset(location);
            }
        }

        self.skeleton_manager
            .apply_tracker_config(&config.trackers, &self.trackers);
        self.skeleton_manager
            .apply_skeleton_config(&config.skeleton);
        modules.vrchat_connector.apply_config(&config.vrchat).await;
        modules.vmc_connector.apply_config(&config.vmc).await;
        modules.websocket_server.send_config(config).await?;
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
