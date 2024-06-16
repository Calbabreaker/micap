use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{tracker::*, udp_server::UdpServer};

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
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    message_channels: MessageChannelManager,
}

impl MainServer {
    pub fn new_message_channel(&mut self) -> UnboundedReceiver<ServerMessage> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.message_channels.channels.push(tx);
        rx
    }

    pub fn load_config(&mut self) {
        let tracker_configs = HashMap::<String, TrackerConfig>::new();
        for (id, config) in tracker_configs {
            self.register_tracker(id, config);
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        for tracker in &mut self.trackers {
            tracker.tick(delta);
            self.message_channels
                .send_to_all(ServerMessage::TrackerData {
                    index: tracker.info.index,
                    data: tracker.data.clone(),
                });
        }
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

    pub fn update_tracker_status(&mut self, index: usize, status: TrackerStatus) {
        let info = &mut self.trackers[index].info;
        info.status = status;
        self.message_channels
            .send_to_all(ServerMessage::TrackerInfo { info: info.clone() });
    }

    pub fn update_tracker_data(
        &mut self,
        index: usize,
        acceleration: glam::Vec3A,
        orientation: glam::Quat,
    ) {
        let data = &mut self.trackers[index].data;
        data.orientation = orientation;
        data.acceleration = acceleration;
    }

    pub fn notify_error(&mut self, error: &str) {
        self.message_channels.send_to_all(ServerMessage::Error {
            error: error.to_string(),
        });
    }
}

const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 50);

pub async fn start_server(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    let mut last_loop_time = Instant::now();

    let mut udp_server = UdpServer::new().await?;

    loop {
        let delta = last_loop_time.elapsed();
        last_loop_time = Instant::now();

        // Tick all the servers
        {
            let mut main = main.write().await;
            main.tick(delta);
            udp_server.tick(&mut main).await?;
        }

        let post_delta = last_loop_time.elapsed();
        if let Some(sleep_duration) = TARGET_LOOP_DELTA.checked_sub(post_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {post_delta:?} which is longer than target {TARGET_LOOP_DELTA:?}"
            );
        }
    }
}
