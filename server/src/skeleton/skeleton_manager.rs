use std::collections::HashMap;

use serde::Serialize;
use ts_rs::TS;

use crate::{
    main_server::{GlobalConfig, TrackerRef},
    skeleton::{Bone, BoneLocation},
};

#[derive(Default, Serialize, TS)]
pub struct SkeletonManager {
    pub bones: HashMap<BoneLocation, Bone>,
    #[serde(skip)]
    trackers: HashMap<BoneLocation, TrackerRef>,
}

impl SkeletonManager {
    pub fn update(&mut self) {}

    pub fn apply_config(&mut self, config: &GlobalConfig, trackers: &HashMap<String, TrackerRef>) {
        // Sets self.trackers based on bone location
        self.trackers.clear();
        for (id, config) in &config.trackers {
            if let Some(location) = config.location {
                self.trackers.insert(location, trackers[id].clone());
            }
        }

        for joints in self.bones.values_mut() {
            joints.set_tail_offset(&config.skeleton.offsets);
        }
    }
}
