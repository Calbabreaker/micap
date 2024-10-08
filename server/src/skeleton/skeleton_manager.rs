use std::{collections::HashMap, sync::Arc};

use crate::{
    skeleton::{Bone, BoneLocation, BoneOffsetKind, SkeletonConfig},
    tracker::{TrackerConfig, TrackerRef, TrackerStatus},
};

pub struct SkeletonManager {
    pub bones: HashMap<BoneLocation, Bone>,
    trackers: HashMap<BoneLocation, TrackerRef>,
    leg_length: f32,
}

impl Default for SkeletonManager {
    fn default() -> Self {
        let bones = BoneLocation::SELF_AND_PARENT
            .iter()
            .map(|(location, parent)| (*location, Bone::new(*parent)))
            .collect();

        Self {
            bones,
            trackers: HashMap::new(),
            leg_length: 0.,
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

        // self.set_bone_orientation(
        //     Hip,
        //     self.get_tracker_orientation(&[Hip, Waist, Chest, UpperChest]),
        // );

        self.set_bone_orientation(
            LeftUpperLeg,
            self.get_tracker_orientation(&[LeftUpperLeg, LeftLowerLeg]),
        );
        self.set_bone_orientation(
            LeftLowerLeg,
            self.get_tracker_orientation(&[LeftLowerLeg, LeftUpperLeg]),
        );

        self.update_bone_recursive(Hip, glam::Vec3A::ZERO, glam::Quat::IDENTITY);
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
        use BoneOffsetKind::*;
        self.leg_length = config.offsets[&LowerLegLength] + config.offsets[&UpperLegLength];

        for (location, bone) in &mut self.bones {
            bone.tail_offset = location.get_tail_offset(&config.offsets);
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
                if tracker.info().status == TrackerStatus::Ok {
                    return Some(tracker.data().orientation);
                }
            }
        }

        None
    }

    /// Update the world position of the bone and its children
    pub fn update_bone_recursive(
        &mut self,
        location: BoneLocation,
        parent_world_position: glam::Vec3A,
        parent_world_orientation: glam::Quat,
    ) {
        let bone = self.bones.get_mut(&location).unwrap();
        let orientation = parent_world_orientation.mul_quat(bone.orientation);
        let local_position = orientation.mul_vec3a(bone.tail_offset);
        let world_position = local_position + parent_world_position;
        bone.tail_world_position = world_position;
        bone.tail_world_position.y += self.leg_length;

        // Recursively update the children positions
        for child_location in location.get_children() {
            self.update_bone_recursive(*child_location, world_position, orientation);
        }
    }
}
