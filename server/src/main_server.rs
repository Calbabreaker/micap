use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    time::Instant,
};

use anyhow::Context;

use crate::{
    tracker::*, udp::server::UdpServer, vmc::connector::VmcConnector, websocket::WebsocketServer,
};

pub struct SubModules {
    udp_server: UdpServer,
    vmc_connector: VmcConnector,
    websocket_server: WebsocketServer,
}

impl SubModules {
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

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    /// Contains list of indexs of trackers who's info has been updated in the middle of a frame
    pub tracker_info_updated_indexs: Vec<usize>,
    /// Contains list of errors emited in the middle of a frame
    pub new_errors: Vec<String>,
    /// Set of address that should not be allowed to connect
    /// This is to allow for servers to ignore ignored trackers that are trying to connect
    pub address_blacklist: HashSet<SocketAddr>,
}

impl MainServer {
    pub fn load_config(&mut self) {
        let tracker_configs = HashMap::<String, TrackerConfig>::new();
        for (id, config) in tracker_configs {
            self.register_tracker(id, Tracker::new(config));
        }
    }

    pub async fn update(&mut self, modules: &mut SubModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        for tracker in &mut self.trackers {
            if tracker.info.removed {
                continue;
            }

            tracker.tick();
        }

        modules.vmc_connector.update(self).await?;
        self.tracker_info_updated_indexs.clear();
        self.new_errors.clear();

        Ok(())
    }

    /// Register a tracker to get its index and use that to access it later since using strings with hashmaps is a bit slow
    pub fn register_tracker(&mut self, id: String, tracker: Tracker) -> usize {
        if let Some(index) = self.tracker_id_to_index.get(&id) {
            return *index;
        }

        let index = self.trackers.len();
        self.tracker_id_to_index.insert(id, index);
        self.tracker_info_updated_indexs.push(index);
        self.trackers.push(tracker);
        index
    }

    pub fn tracker_info_updated(&mut self, index: usize) {
        self.tracker_info_updated_indexs.push(index);
    }

    pub fn update_tracker_data(
        &mut self,
        index: usize,
        acceleration: glam::Vec3A,
        orientation: glam::Quat,
    ) {
        let tracker = &mut self.trackers[index];
        tracker.data.orientation = orientation;
        tracker.data.acceleration = acceleration;
        tracker.time_data_received = Instant::now();
    }
}
