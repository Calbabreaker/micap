use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{tracker::*, udp::server::UdpServer, vmc::connector::VmcConnector};

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    TrackerInfo { info: TrackerInfo },
    TrackerData { index: usize, data: TrackerData },
    Error { error: String },
}

/// Stuff like the websocket server needs to run on a completely seperate task but need to receive
/// tracker data when it is ready. So we use mspc channels
#[derive(Default)]
pub struct MessageChannelManager {
    channels: Vec<UnboundedSender<ServerMessage>>,
}

impl MessageChannelManager {
    fn send_to_all(&mut self, message: ServerMessage) {
        let mut to_remove = None;

        for (i, channel) in self.channels.iter().enumerate() {
            // The channel got closed so remove it
            if channel.send(message.clone()).is_err() {
                to_remove = Some(i);
            }
        }

        if let Some(to_remove) = to_remove {
            self.channels.swap_remove(to_remove);
        }
    }

    pub fn new_channel(&mut self) -> UnboundedReceiver<ServerMessage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.channels.push(tx);
        rx
    }
}

pub struct SubModules {
    udp_server: UdpServer,
    vmc_connector: VmcConnector,
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    pub message_channels: MessageChannelManager,
}

impl MainServer {
    pub fn load_config(&mut self) {
        let tracker_configs = HashMap::<String, TrackerConfig>::new();
        for (id, config) in tracker_configs {
            self.register_tracker(id, config);
        }
    }

    pub async fn update(&mut self, modules: &mut SubModules) -> anyhow::Result<()> {
        modules.udp_server.update(self).await?;

        for tracker in &mut self.trackers {
            tracker.tick();
            self.message_channels
                .send_to_all(ServerMessage::TrackerData {
                    index: tracker.info.index,
                    data: tracker.data.clone(),
                });
        }

        modules.vmc_connector.update(self).await?;

        Ok(())
    }

    // Register a tracker to get its index and use that to access it later since using strings with
    // hashmaps is a bit slow
    pub fn register_tracker(&mut self, id: String, config: TrackerConfig) -> usize {
        if let Some(index) = self.tracker_id_to_index.get(&id) {
            return *index;
        }

        let index = self.trackers.len();
        let tracker = Tracker::new(id.clone(), index, config);
        self.tracker_id_to_index.insert(id, index);
        self.message_channels
            .send_to_all(ServerMessage::TrackerInfo {
                info: tracker.info.clone(),
            });
        self.trackers.push(tracker);
        index
    }

    pub fn tracker_info_updated(&mut self, index: usize) {
        self.message_channels
            .send_to_all(ServerMessage::TrackerInfo {
                info: self.trackers[index].clone().info,
            });
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

    pub fn notify_error(&mut self, error: &str) {
        self.message_channels.send_to_all(ServerMessage::Error {
            error: error.to_string(),
        });
    }
}

const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 50);

pub async fn start_server(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    let mut modules = SubModules {
        udp_server: UdpServer::new()
            .await
            .context("Failed to start UDP server")?,
        vmc_connector: VmcConnector::new().await?,
    };

    loop {
        let tick_start_time = Instant::now();

        // Tick all the servers
        {
            let mut main = main.write().await;
            main.update(&mut modules).await?;
        }

        let post_delta = tick_start_time.elapsed();
        if let Some(sleep_duration) = TARGET_LOOP_DELTA.checked_sub(post_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {post_delta:?} which is longer than target {TARGET_LOOP_DELTA:?}"
            );
        }
    }
}
