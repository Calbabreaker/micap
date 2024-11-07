use std::{collections::HashMap, sync::Arc};

use crate::{
    math::locked_with_yaw,
    skeleton::{Bone, BoneLocation, SkeletonConfig},
    tracker::{TrackerConfig, TrackerRef, TrackerStatus},
};
use BoneLocation::*;

pub struct SkeletonManager {
    pub bones: HashMap<BoneLocation, Bone>,
    trackers: HashMap<BoneLocation, TrackerRef>,
    pub root_position: glam::Vec3A,
}

impl Default for SkeletonManager {
    fn default() -> Self {
        let bones = BoneLocation::SELF_AND_PARENT
            .iter()
            .map(|(location, parent)| (*location, Bone::new(*parent)))
            .collect();

        Self {
            root_position: glam::Vec3A::ZERO,
            bones,
            trackers: HashMap::new(),
        }
    }
}

impl SkeletonManager {
    pub fn update(&mut self) {
        self.update_head();
        self.update_spine();
        self.update_leg(LeftHip, LeftUpperLeg, LeftLowerLeg, LeftFoot);
        self.update_leg(RightHip, RightUpperLeg, RightLowerLeg, RightFoot);
        self.update_arm(LeftShoulder, LeftUpperArm, LeftLowerArm, LeftHand);
        self.update_arm(RightShoulder, RightUpperArm, RightLowerArm, RightHand);

        // Update the hip and consequently every bone
        self.update_bone_recursive(
            BoneLocation::CenterHip,
            self.root_position,
            glam::Quat::IDENTITY,
        );
    }

    fn update_head(&mut self) {
        let orientation =
            self.get_tracker_orientation(&[Head, Neck, UpperChest, Chest, Waist, CenterHip]);
        self.set_bone_orientation(&[Head, Neck], orientation);
    }

    fn update_spine(&mut self) {
        if self.check_any_trackers_exist(&[UpperChest, Chest, Waist, CenterHip]) {
            let orientation = self.get_tracker_orientation(&[CenterHip, UpperChest, Waist, Chest]);
            self.set_bone_orientation(&[CenterHip], locked_with_yaw(orientation));

            let orientation = self.get_tracker_orientation(&[UpperChest, Chest, Waist, CenterHip]);
            self.set_bone_orientation(&[UpperChest], orientation);

            let orientation = self.get_tracker_orientation(&[Chest, UpperChest, Waist, CenterHip]);
            self.set_bone_orientation(&[Chest], orientation);

            let orientation = self.get_tracker_orientation(&[Waist, CenterHip, Chest, UpperChest]);
            self.set_bone_orientation(&[Waist], orientation);
        } else {
            // Use the head yaw instead
            let orientation = locked_with_yaw(self.bones[&Head].world_orientation);
            self.set_bone_orientation(&[Waist, UpperChest, Chest, CenterHip], orientation);
        }
    }

    fn update_leg(
        &mut self,
        side_hip: BoneLocation,
        upper_leg: BoneLocation,
        lower_leg: BoneLocation,
        foot: BoneLocation,
    ) {
        let hip_quat = self.bones[&CenterHip].world_orientation;
        self.set_bone_orientation(&[side_hip], hip_quat);
        let mut leg_quat = self.get_tracker_orientation_or_default(&[upper_leg], hip_quat);
        self.set_bone_orientation(&[upper_leg], leg_quat);

        // Use the lower leg tracker or the upper leg (locked to yaw)
        leg_quat = self.get_tracker_orientation_or_default(&[lower_leg], leg_quat);
        self.set_bone_orientation(&[lower_leg], leg_quat);

        leg_quat = self.get_tracker_orientation_or_default(&[foot], leg_quat);
        self.set_bone_orientation(&[foot], leg_quat);
    }

    fn update_arm(
        &mut self,
        shoulder: BoneLocation,
        upper_arm: BoneLocation,
        lower_arm: BoneLocation,
        hand: BoneLocation,
    ) {
        let upper_chest_quat = self.bones[&UpperChest].world_orientation;
        let mut arm_quat = self.get_tracker_orientation_or_default(&[shoulder], upper_chest_quat);
        self.set_bone_orientation(&[shoulder], arm_quat);

        arm_quat = self.get_tracker_orientation_or_default(&[upper_arm], arm_quat);
        self.set_bone_orientation(&[upper_arm], arm_quat);

        arm_quat = self.get_tracker_orientation_or_default(&[lower_arm], arm_quat);
        self.set_bone_orientation(&[lower_arm], arm_quat);

        arm_quat = self.get_tracker_orientation_or_default(&[hand], arm_quat);
        self.set_bone_orientation(&[hand], arm_quat);
    }

    pub fn apply_tracker_config(
        &mut self,
        tracker_configs: &HashMap<Arc<str>, TrackerConfig>,
        trackers: &HashMap<Arc<str>, TrackerRef>,
    ) {
        // Sets self.trackers based on bone location
        self.trackers.clear();
        for (id, tracker_config) in tracker_configs {
            if let Some(location) = tracker_config.location {
                self.trackers.insert(location, trackers[id].clone());
            }
        }
    }

    pub fn apply_skeleton_config(&mut self, config: &SkeletonConfig) {
        self.root_position.y = config.get_leg_length();

        for (location, bone) in &mut self.bones {
            bone.tail_offset = location.get_tail_offset(&config.offsets);
        }
    }

    fn check_any_trackers_exist(&self, locations: &[BoneLocation]) -> bool {
        locations
            .iter()
            .any(|location| self.trackers.contains_key(location))
    }

    fn set_bone_orientation(&mut self, locations: &[BoneLocation], orientation: glam::Quat) {
        for location in locations {
            let bone = self.bones.get_mut(location).unwrap();
            bone.world_orientation = orientation;
        }
    }

    /// Gets the first avaliable tracker's orientation based on provided locations
    fn get_tracker_orientation_or_default(
        &self,
        locations: &[BoneLocation],
        default: glam::Quat,
    ) -> glam::Quat {
        let quat = locations.iter().find_map(|location| {
            let tracker = self.trackers.get(location)?;
            let tracker = tracker.lock().unwrap();
            if tracker.info().status == TrackerStatus::Ok {
                Some(tracker.data().orientation)
            } else {
                None
            }
        });
        quat.unwrap_or(default)
    }

    /// Gets the first avaliable tracker's orientation based on provided locations or identity if none found
    fn get_tracker_orientation(&self, locations: &[BoneLocation]) -> glam::Quat {
        self.get_tracker_orientation_or_default(locations, glam::Quat::IDENTITY)
    }

    /// Update the world position and local orientation of the bone and its children
    fn update_bone_recursive(
        &mut self,
        location: BoneLocation,
        parent_world_position: glam::Vec3A,
        parent_world_orientation: glam::Quat,
    ) {
        let bone = self.bones.get_mut(&location).unwrap();

        // The trackers set the world orientation so we need to go back and calculate the local orientation
        let world_orientation = bone.world_orientation;
        bone.local_orientation = parent_world_orientation.inverse() * world_orientation;

        let local_position = world_orientation * bone.tail_offset;
        let world_position = local_position + parent_world_position;
        bone.tail_world_position = world_position;

        // Recursively update the children positions
        for child_location in location.get_children() {
            self.update_bone_recursive(*child_location, world_position, world_orientation);
        }
    }
}
