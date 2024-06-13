use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;

use crate::tracker::*;

#[derive(Clone)]
pub enum ServerMessage {
    TrackerInfoUpdate(TrackerInfo),
    TrackerDataUpdate((usize, TrackerData)),
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    message_channels: Vec<UnboundedSender<ServerMessage>>,
}

impl MainServer {
    pub fn add_message_channel(&mut self, tx: UnboundedSender<ServerMessage>) {
        self.message_channels.push(tx);
    }

    pub fn load_config(&mut self) {
        let tracker_configs = HashMap::<String, TrackerConfig>::new();
        for (id, config) in tracker_configs {
            self.register_tracker(id, config);
        }
    }

    pub fn tick() {}

    // Register a tracker to get its index and use that to access it later since using strings with
    // hashmaps is a bit slow
    pub fn register_tracker(&mut self, id: String, config: TrackerConfig) -> usize {
        if let Some(index) = self.tracker_id_to_index.get(&id) {
            return *index;
        }

        let index = self.trackers.len();
        self.trackers.push(Tracker::new(id.clone(), index, config));
        self.tracker_id_to_index.insert(id, index);
        self.send_message_to_channels(ServerMessage::TrackerInfoUpdate(
            self.trackers[index].info.clone(),
        ));
        index
    }

    pub fn update_tracker_status(&mut self, index: usize, status: TrackerStatus) {
        self.trackers[index].info.status = status;
        self.send_message_to_channels(ServerMessage::TrackerInfoUpdate(
            self.trackers[index].info.clone(),
        ));
    }

    fn send_message_to_channels(&mut self, message: ServerMessage) {
        let mut to_remove = None;

        for (i, channel) in self.message_channels.iter().enumerate() {
            // The channel got closed so remove it
            if channel.send(message.clone()).is_err() {
                to_remove = Some(i)
            }
        }

        if let Some(to_remove) = to_remove {
            self.message_channels.swap_remove(to_remove);
        }
    }
}
