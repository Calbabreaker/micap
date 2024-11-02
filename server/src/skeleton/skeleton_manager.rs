use std::{collections::HashMap, sync::Arc};

use crate::{
    math::locked_with_y,
    skeleton::{Bone, BoneLocation, BoneOffsetKind, SkeletonConfig},
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
        self.update_leg(LeftUpperLeg, LeftLowerLeg, LeftFoot);
        self.update_leg(RightUpperLeg, RightLowerLeg, RightFoot);
        self.update_arm(LeftShoulder, LeftUpperArm, LeftLowerArm, LeftHand);
        self.update_arm(RightShoulder, RightUpperArm, RightLowerArm, RightHand);

        // Update the hip and consequently every bone
        self.update_bone_recursive(BoneLocation::Hip, self.root_position, glam::Quat::IDENTITY);
    }

    fn update_head(&mut self) {
        if let Some(quat) =
            self.get_tracker_orientation(&[Head, Neck, UpperChest, Chest, Waist, Hip])
        {
            self.set_bone_orientation(&[Head, Neck], quat);
        }
    }

    fn update_spine(&mut self) {
        if self.check_any_trackers_exist(&[UpperChest, Chest, Waist, Hip]) {
            if let Some(quat) = self.get_tracker_orientation(&[Hip, UpperChest, Waist, Chest]) {
                self.set_bone_orientation(&[Hip], locked_with_y(quat, glam::EulerRot::XYZ));
            }

            if let Some(quat) = self.get_tracker_orientation(&[UpperChest, Chest, Waist, Hip]) {
                self.set_bone_orientation(&[UpperChest], quat);
            }

            if let Some(quat) = self.get_tracker_orientation(&[Chest, UpperChest, Waist, Hip]) {
                self.set_bone_orientation(&[Chest], quat);
            }

            if let Some(quat) = self.get_tracker_orientation(&[Waist, Hip, Chest, UpperChest]) {
                self.set_bone_orientation(&[Waist], quat);
            }
        } else {
            // Use the head yaw instead
            let quat = locked_with_y(self.bones[&Head].orientation, glam::EulerRot::XYZ);
            self.set_bone_orientation(&[Waist, UpperChest, Chest], quat);
        }
    }

    fn update_leg(&mut self, upper_leg: BoneLocation, lower_leg: BoneLocation, foot: BoneLocation) {
        let waist_quat = locked_with_y(self.bones[&Waist].orientation, glam::EulerRot::XYZ);
        let mut leg_quat = self
            .get_tracker_orientation(&[upper_leg])
            .unwrap_or(waist_quat);
        self.set_bone_orientation(&[upper_leg], leg_quat);

        // Use the lower leg tracker or the upper leg (locked to yaw)
        leg_quat = self
            .get_tracker_orientation(&[lower_leg])
            .unwrap_or(locked_with_y(leg_quat, glam::EulerRot::XYZ));
        self.set_bone_orientation(&[lower_leg], leg_quat);

        leg_quat = self.get_tracker_orientation(&[foot]).unwrap_or(leg_quat);
        self.set_bone_orientation(&[foot], leg_quat);
    }

    fn update_arm(
        &mut self,
        shoulder: BoneLocation,
        upper_arm: BoneLocation,
        lower_arm: BoneLocation,
        hand: BoneLocation,
    ) {
        let upper_chest_quat =
            locked_with_y(self.bones[&UpperChest].orientation, glam::EulerRot::XZY);
        let mut arm_quat = self
            .get_tracker_orientation(&[shoulder])
            .unwrap_or(upper_chest_quat);
        self.set_bone_orientation(&[shoulder], arm_quat);

        arm_quat = self
            .get_tracker_orientation(&[upper_arm])
            .unwrap_or(arm_quat);
        self.set_bone_orientation(&[upper_arm], arm_quat);

        arm_quat = self
            .get_tracker_orientation(&[lower_arm])
            .unwrap_or(arm_quat);
        self.set_bone_orientation(&[lower_arm], arm_quat);

        arm_quat = self.get_tracker_orientation(&[hand]).unwrap_or(arm_quat);
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
        use BoneOffsetKind::*;
        let leg_length = config.offsets[&LowerLegLength] + config.offsets[&UpperLegLength];
        self.root_position.y = leg_length;

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
            bone.orientation = orientation;
        }
    }

    /// Gets the first avaliable tracker's orientation based on provided locations
    fn get_tracker_orientation(&self, locations: &[BoneLocation]) -> Option<glam::Quat> {
        locations.iter().find_map(|location| {
            let tracker = self.trackers.get(location)?;
            let tracker = tracker.lock().unwrap();
            if tracker.info().status == TrackerStatus::Ok {
                Some(tracker.data().orientation)
            } else {
                None
            }
        })
    }

    /// Update the world position and orientation of the bone and its children
    fn update_bone_recursive(
        &mut self,
        location: BoneLocation,
        parent_world_position: glam::Vec3A,
        parent_world_orientation: glam::Quat,
    ) {
        let bone = self.bones.get_mut(&location).unwrap();

        let world_orientation = bone.orientation * parent_world_orientation;
        bone.world_orientation = world_orientation;

        let local_position = world_orientation * bone.tail_offset;
        let world_position = local_position + parent_world_position;
        bone.tail_world_position = world_position;

        // Recursively update the children positions
        for child_location in location.get_children() {
            self.update_bone_recursive(*child_location, world_position, world_orientation);
        }
    }
}
