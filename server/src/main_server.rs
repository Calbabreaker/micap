use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::PathBuf,
    time::Instant,
};

use anyhow::Context;

use crate::{
    serial::SerialPortManager,
    tracker::*,
    udp::server::UdpServer,
    vmc::connector::{VmcConfig, VmcConnector},
    websocket::WebsocketServer,
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

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub trackers: HashMap<String, TrackerConfig>,
    pub vmc: VmcConfig,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum UpdateEvent {
    TrackerInfo { id: String },
    Error { error: String },
    SerialPort { port_name: Option<String> },
    ConfigUpdate,
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
    pub serial_manager: SerialPortManager,
    pub config: GlobalConfig,
}

impl MainServer {
    pub fn load_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Loading from {path:?}");
        let text = std::fs::read_to_string(path)?;
        let config = serde_json::from_str::<GlobalConfig>(&text)?;

        for id in config.trackers.keys() {
            self.trackers.insert(id.clone(), Tracker::default());
            self.tracker_info_update(id);
        }

        self.config = config;
        Ok(())
    }

    pub fn save_config(&mut self) -> anyhow::Result<()> {
        let path = get_config_dir()?.join("config.json");
        log::info!("Saving to {path:?}");
        let text = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, text)?;

        Ok(())
    }

    pub async fn update(&mut self, modules: &mut SubModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;
        modules.websocket_server.update(self).await?;

        modules.vmc_connector.update(self).await?;

        Ok(())
    }

    pub fn add_tracker(&mut self, id: String, config: TrackerConfig) {
        if !self.trackers.contains_key(&id) {
            self.trackers.insert(id.clone(), Tracker::default());
            self.tracker_info_update(&id);
            self.config.trackers.insert(id, config);
            self.updates.push(UpdateEvent::ConfigUpdate);

            if let Err(err) = self.save_config() {
                log::error!("Failed to save tracker: {err:?}");
            }
        }
    }

    pub fn tracker_info_update(&mut self, id: &String) -> Option<&mut Tracker> {
        let tracker = self.trackers.get_mut(id)?;
        self.updates
            .push(UpdateEvent::TrackerInfo { id: id.clone() });
        Some(tracker)
    }

    pub fn remove_tracker(&mut self, id: &String) {
        if self.trackers.remove(id).is_some() {
            self.config.trackers.remove(id);
            self.updates.push(UpdateEvent::ConfigUpdate);
        }
    }

    pub fn update_tracker_data(
        &mut self,
        id: &String,
        acceleration: glam::Vec3A,
        orientation: glam::Quat,
    ) {
        if let Some(tracker) = self.trackers.get_mut(id) {
            tracker.data.orientation = orientation;
            tracker.data.acceleration = acceleration;

            if tracker.info.status == TrackerStatus::Ok {
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
