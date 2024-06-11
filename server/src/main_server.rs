use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;

use crate::tracker::*;

#[derive(Clone)]
pub enum ServerMessage {
    TrackerInfoUpdate(TrackerInfo),
    TrackerDataUpdate(TrackerData),
}

struct ServerMessageChannel {
    tx: UnboundedSender<ServerMessage>,
    // Id to keep track of where channel is in array to remove
    id: u32,
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    message_channels: Vec<ServerMessageChannel>,
    next_message_tx_id: u32,
}

impl MainServer {
    pub fn add_message_channel(&mut self, tx: UnboundedSender<ServerMessage>) -> u32 {
        let id = self.next_message_tx_id;
        self.message_channels.push(ServerMessageChannel { tx, id });
        self.next_message_tx_id += 1;
        id
    }

    pub fn remove_message_channel(&mut self, id: u32) {
        if let Some(index) = self
            .message_channels
            .iter()
            .position(|channel| channel.id == id)
        {
            self.message_channels.swap_remove(index);
        }
    }

    pub fn load_config(&mut self) {
        let tracker_configs = HashMap::<String, TrackerConfig>::new();
        for (id, config) in &tracker_configs {
            self.register_tracker(id);
        }
    }

    // Register a tracker to get its index and use that to access it later since using strings with
    // hashmaps is a bit slow
    pub fn register_tracker(&mut self, id: &String) -> usize {
        if let Some(index) = self.tracker_id_to_index.get(id) {
            return *index;
        }

        let index = self.trackers.len();
        self.trackers.push(Tracker::new(id.clone(), index));
        self.tracker_id_to_index.insert(id.clone(), index);
        self.send_message_to_all(ServerMessage::TrackerInfoUpdate(
            self.trackers[index].info.clone(),
        ));
        index
    }

    pub fn update_tracker_status(&mut self, index: usize, status: TrackerStatus) {
        let info = &mut self.trackers[index].info;
        // Only allow changing status to TimedOut if tracker is Ok and vice-versa
        if (status == TrackerStatus::TimedOut && info.status != TrackerStatus::Ok)
            || (info.status == TrackerStatus::TimedOut && status != TrackerStatus::Ok)
        {
            return;
        }

        info.status = status;
        self.send_message_to_all(ServerMessage::TrackerInfoUpdate(
            self.trackers[index].info.clone(),
        ));
    }

    fn send_message_to_all(&mut self, message: ServerMessage) {
        for channel in &self.message_channels {
            channel.tx.send(message.clone()).unwrap();
        }
    }
}
