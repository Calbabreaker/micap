use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::PathBuf,
    time::Instant,
};

use anyhow::Context;
use glam::Vec3Swizzles;

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
            vmc_connector: VmcConnector::new()
                .await
                .context("Failed to connect to VMC")?,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    trackers: HashMap<String, TrackerConfig>,
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
    pub fn load_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::trace!("Loading from {path:?}");
        let text = std::fs::read_to_string(path).context("Failed to load config")?;
        let config: Config = serde_json::from_str(&text)?;

        for (id, config) in config.trackers {
            self.add_tracker(id, Tracker::new(config));
        }

        Ok(())
    }

    pub fn save_config(&mut self) -> anyhow::Result<()> {
        let trackers = self
            .tracker_id_to_index
            .iter()
            .filter_map(|(id, index)| {
                let tracker = &self.trackers[*index];
                if tracker.info.removed {
                    None
                } else {
                    Some((id.clone(), tracker.info.config.clone()))
                }
            })
            .collect::<HashMap<String, TrackerConfig>>();

        let config = Config { trackers };

        let path = get_config_dir()?.join("config.json");
        log::trace!("Saving to {path:?}");
        let text = serde_json::to_string_pretty(&config)?;
        std::fs::write(path, text).context("Failed to save config")?;

        Ok(())
    }

    pub async fn update(&mut self, modules: &mut SubModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        // for tracker in &mut self.trackers {
        //     // tracker.tick();
        // }

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

        let index = self.add_tracker(id, tracker);
        self.save_config().ok();
        index
    }

    fn add_tracker(&mut self, id: String, tracker: Tracker) -> usize {
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
        let acceleration = acceleration.xzy();
        let tracker = &mut self.trackers[index];
        tracker.data.orientation = orientation;
        tracker.data.acceleration = acceleration;

        if tracker.info.status == TrackerStatus::Ok && acceleration.length() > 3. {
            let delta = tracker.time_data_received.elapsed().as_secs_f32();
            tracker.data.velocity += tracker.data.acceleration * delta;
            tracker.data.position += tracker.data.velocity * delta;
        }

        tracker.time_data_received = Instant::now();
    }

    pub fn update_tracker_config(
        &mut self,
        index: usize,
        config: TrackerConfig,
    ) -> anyhow::Result<()> {
        self.trackers[index].info.config = config;
        self.tracker_info_updated(index);
        self.save_config()
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
