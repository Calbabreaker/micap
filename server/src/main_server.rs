use std::collections::HashMap;

use crate::math::{Quaternion, Vector3};

#[derive(Default, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct Tracker {
    pub id: String,
    pub index: usize,
    pub status: TrackerStatus,
    #[serde(skip)]
    pub orientation: Quaternion,
    #[serde(skip)]
    pub acceleration: Vector3,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {}

#[derive(Default)]
pub struct MainServer {
    trackers: Vec<Tracker>,
    tracker_id_to_index: HashMap<String, usize>,
    tracker_configs: HashMap<String, TrackerConfig>,
}

impl MainServer {
    pub fn load_config(&mut self) {
        for (id, config) in &self.tracker_configs.clone() {
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
        self.trackers.push(Tracker {
            id: id.clone(),
            index,
            ..Default::default()
        });
        self.tracker_id_to_index.insert(id.clone(), index);
        self.tracker_configs
            .insert(id.clone(), TrackerConfig::default());
        index
    }

    pub fn set_tracker_status(&mut self, index: usize, status: TrackerStatus) {
        self.trackers[index].status = status;
    }
}
