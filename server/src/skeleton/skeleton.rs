use std::collections::HashMap;

use crate::{
    main_server::{MainServer, TrackerRef, UpdateEvent},
    skeleton::bone::{Bone, BoneLocation},
};

pub struct Skeleton {
    bones: HashMap<BoneLocation, Bone>,
    trackers: HashMap<BoneLocation, TrackerRef>,
}

impl Skeleton {
    pub fn new() -> Self {
        // Bones will be offseted based on body config
        // Hip is considered the root bone as well
        (BoneLocation::Hip, BoneLocation::Waist);
        (BoneLocation::Neck, BoneLocation::Chest);
        Self {
            bones: HashMap::new(),
            trackers: HashMap::new(),
        }
    }

    pub fn update(&mut self, main: &mut MainServer) {
        if main.updates.contains(&UpdateEvent::ConfigUpdate) {
            self.assign_trackers(main);
        }
    }

    pub fn assign_trackers(&mut self, main: &mut MainServer) {
        self.trackers.clear();
        for (id, config) in &main.config.trackers {
            if let Some(location) = config.location {
                self.trackers.insert(location, main.trackers[id].clone());
            }
        }
    }
}
