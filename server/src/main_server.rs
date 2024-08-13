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

pub enum UpdateEvent {
    TrackerInfoUpdate(String),
    TrackerRemove(String),
    NewError(String),
}

#[derive(Default)]
pub struct MainServer {
    // Maps a tracker id to a tracker
    pub trackers: HashMap<String, Tracker>,
    /// Contains list of update event emited in the middle of a frame
    pub updates: Vec<UpdateEvent>,
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
            .trackers
            .iter()
            .map(|(id, tracker)| (id.clone(), tracker.info.config.clone()))
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
        self.updates.clear();

        Ok(())
    }

    pub fn add_tracker(&mut self, id: String, tracker: Tracker) {
        self.trackers.insert(id.clone(), tracker);
        self.updates.push(UpdateEvent::TrackerInfoUpdate(id));
    }

    pub fn tracker_info_update(&mut self, id: &String) -> Option<&mut Tracker> {
        let tracker = self.trackers.get_mut(id)?;
        self.updates
            .push(UpdateEvent::TrackerInfoUpdate(id.clone()));
        Some(tracker)
    }

    pub fn remove_tracker(&mut self, id: &String) {
        if self.trackers.remove(id).is_some() {
            self.updates.push(UpdateEvent::TrackerRemove(id.clone()));
        }
    }

    pub fn update_tracker_data(
        &mut self,
        id: &String,
        acceleration: glam::Vec3A,
        orientation: glam::Quat,
    ) {
        if let Some(tracker) = self.trackers.get_mut(id) {
            let acceleration = acceleration.xzy();
            tracker.data.orientation = orientation;
            tracker.data.acceleration = acceleration;

            if tracker.info.status == TrackerStatus::Ok && acceleration.length() > 3. {
                let delta = tracker.time_data_received.elapsed().as_secs_f32();
                tracker.data.velocity += tracker.data.acceleration * delta;
                tracker.data.position += tracker.data.velocity * delta;
            }

            tracker.time_data_received = Instant::now();
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
