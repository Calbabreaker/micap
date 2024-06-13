use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{
    math::{Quaternion, Vector3},
    tracker::*,
};

#[derive(Clone)]
pub enum ServerMessage {
    TrackerInfoUpdate(TrackerInfo),
    TrackerDataUpdate((usize, TrackerData)),
}

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
                to_remove = Some(i)
            }
        }

        if let Some(to_remove) = to_remove {
            self.channels.swap_remove(to_remove);
        }
    }

    pub fn add(&mut self, tx: UnboundedSender<ServerMessage>) {
        self.channels.push(tx);
    }
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

    pub fn tick(&mut self, delta: Duration) {
        for tracker in &mut self.trackers {
            tracker.tick(delta);
            self.message_channels
                .send_to_all(ServerMessage::TrackerDataUpdate((
                    tracker.info.index,
                    tracker.data.clone(),
                )));
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
            .send_to_all(ServerMessage::TrackerInfoUpdate(tracker.info.clone()));
        self.trackers.push(tracker);
        index
    }

    pub fn update_tracker_status(&mut self, index: usize, status: TrackerStatus) {
        let info = &mut self.trackers[index].info;
        info.status = status;
        self.message_channels
            .send_to_all(ServerMessage::TrackerInfoUpdate(info.clone()));
    }

    pub fn update_tracker_data(
        &mut self,
        index: usize,
        acceleration: Vector3,
        orientation: Quaternion,
    ) {
        let data = &mut self.trackers[index].data;
        data.orientation = orientation;
        data.acceleration = acceleration;
    }
}

const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 50);

pub async fn start_main_server_loop(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    let mut last_loop_time = Instant::now();
    loop {
        let delta = last_loop_time.elapsed();
        last_loop_time = Instant::now();
        main.write().await.tick(delta);

        let post_delta = last_loop_time.elapsed();
        if let Some(sleep_duration) = TARGET_LOOP_DELTA.checked_sub(post_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {post_delta:?} which is longer than target {TARGET_LOOP_DELTA:?}"
            )
        }
    }
}
