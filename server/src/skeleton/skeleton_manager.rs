use std::collections::HashMap;

use futures_util::FutureExt;
use serde::Serialize;
use ts_rs::TS;

use crate::{
    main_server::{GlobalConfig, TrackerRef},
    skeleton::{Bone, BoneLocation},
};

#[derive(Serialize, TS)]
pub struct SkeletonManager {
    pub bones: HashMap<BoneLocation, Bone>,
    #[serde(skip)]
    trackers: HashMap<BoneLocation, TrackerRef>,
}

impl Default for SkeletonManager {
    fn default() -> Self {
        let bones = BoneLocation::SELF_TO_PARENT
            .iter()
            .map(|(location, parent)| (*location, Bone::new(*parent)))
            .collect();

        Self {
            bones,
            trackers: HashMap::new(),
        }
    }
}

impl SkeletonManager {
    pub fn update(&mut self) {
        use BoneLocation::*;

        self.set_bone_orientation(
            UpperChest,
            self.get_tracker_orientation(&[UpperChest, Chest, Waist, Hip]),
        );

        self.set_bone_orientation(
            Chest,
            self.get_tracker_orientation(&[Chest, UpperChest, Waist, Hip]),
        );

        self.set_bone_orientation(
            Waist,
            self.get_tracker_orientation(&[Waist, Chest, UpperChest, Hip]),
        );

        self.set_bone_orientation(
            Hip,
            self.get_tracker_orientation(&[Hip, Waist, Chest, UpperChest]),
        );
    }

    pub fn apply_config(&mut self, config: &GlobalConfig, trackers: &HashMap<String, TrackerRef>) {
        // Sets self.trackers based on bone location
        self.trackers.clear();
        for (id, config) in &config.trackers {
            if let Some(location) = config.location {
                self.trackers.insert(location, trackers[id].clone());
            }
        }

        for (location, joints) in &mut self.bones {
            joints.set_tail_offset(*location, &config.skeleton.offsets);
        }
    }

    fn set_bone_orientation(&mut self, location: BoneLocation, orientation: Option<glam::Quat>) {
        if let Some(orientation) = orientation {
            self.bones.get_mut(&location).unwrap().orientation = orientation;
        }
    }

    /// Gets the first avaliable tracker's orientation based on provided locations
    fn get_tracker_orientation(&self, locations: &[BoneLocation]) -> Option<glam::Quat> {
        for location in locations {
            if let Some(tracker) = self.trackers.get(location) {
                let tracker = tracker.read().now_or_never()?;
                if tracker.data_was_updated {
                    return Some(tracker.data.orientation);
                }
            }
        }

        None
    }
}
