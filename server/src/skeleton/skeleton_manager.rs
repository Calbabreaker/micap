use std::{collections::HashMap, sync::Arc};

use crate::{
    skeleton::{Bone, BoneLocation, SkeletonConfig, BONE_LOCATION_TO_CHILDREN},
    tracker::{TrackerConfig, TrackerRef},
};

pub struct SkeletonManager {
    pub bones: HashMap<BoneLocation, Bone>,
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

        self.update_bone_position_recursive(Hip, glam::Vec3A::ZERO);
    }

    pub fn apply_tracker_config(
        &mut self,
        configs: &HashMap<Arc<str>, TrackerConfig>,
        trackers: &HashMap<Arc<str>, TrackerRef>,
    ) {
        // Sets self.trackers based on bone location
        self.trackers.clear();
        for (id, config) in configs {
            if let Some(location) = config.location {
                self.trackers.insert(location, trackers[id].clone());
            }
        }
    }

    pub fn apply_skeleton_config(&mut self, config: &SkeletonConfig) {
        for (location, bone) in &mut self.bones {
            bone.set_tail_offset(*location, &config.offsets);
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
                let tracker = tracker.lock().unwrap();
                return Some(tracker.data.orientation);
            }
        }

        None
    }

    pub fn update_bone_position_recursive(
        &mut self,
        location: BoneLocation,
        parent_world_position: glam::Vec3A,
    ) {
        let bone = self.bones.get_mut(&location).unwrap();
        bone.update_position(parent_world_position);

        if let Some(children) = BONE_LOCATION_TO_CHILDREN.get(&location) {
            for child_location in children {
                self.update_bone_position_recursive(*child_location, parent_world_position);
            }
        }
    }
}
