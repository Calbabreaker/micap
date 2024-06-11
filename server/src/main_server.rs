use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;

use crate::tracker::*;

#[derive(Clone)]
pub enum ServerMessage {
    TrackerInfoUpdate(TrackerInfo),
    TrackerDataUpdate(TrackerData),
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    message_rxs: Vec<UnboundedSender<ServerMessage>>,
}

impl MainServer {
    pub fn add_message_rx(&mut self, rx: UnboundedSender<ServerMessage>) {
        self.message_rxs.push(rx)
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
        self.trackers[index].info.status = status;
        self.send_message_to_all(ServerMessage::TrackerInfoUpdate(
            self.trackers[index].info.clone(),
        ));
    }

    fn send_message_to_all(&self, message: ServerMessage) {
        for rx in &self.message_rxs {
            rx.send(message.clone());
        }
    }
}
