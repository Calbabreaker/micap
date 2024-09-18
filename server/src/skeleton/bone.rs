use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::skeleton::BoneOffsetKind;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum BoneLocation {
    /// Also acts as the root joint
    Hip,
    LeftUpperLeg,
    RightUpperLeg,
    LeftLowerLeg,
    RightLowerLeg,
    LeftFoot,
    RightFoot,
    Waist,
    Chest,
    UpperChest,
    Neck,
    Head,
    LeftShoulder,
    RightShoulder,
    LeftUpperArm,
    RightUpperArm,
    LeftLowerArm,
    RightLowerArm,
    LeftHand,
    RightHand,
    /// Connects the hip to the left upper leg
    LeftHip,
    /// Connects the hip to the right upper leg
    RightHip,
}

impl BoneLocation {
    /// Maps to bone names used in unity, this is also what VRM uses
    /// https://docs.unity3d.com/ScriptReference/HumanBodyBones.html
    pub fn as_unity_bone(&self) -> String {
        match self {
            // Only these values are different
            Self::Hip => "Hips".to_string(),
            Self::Waist => "Spine".to_string(),
            bone => format!("{:?}", bone),
        }
    }

    /// Maps to the its parent bone location
    pub const fn get_parent(&self) -> Option<BoneLocation> {
        Some(match self {
            Self::Hip => return None,
            Self::Waist => Self::Hip,
            Self::LeftHip | Self::RightHip => Self::Hip,
            Self::LeftUpperLeg => Self::LeftHip,
            Self::LeftLowerLeg => Self::LeftUpperLeg,
            Self::LeftFoot => Self::LeftLowerLeg,
            Self::RightUpperLeg => Self::RightHip,
            Self::RightLowerLeg => Self::RightUpperLeg,
            Self::RightFoot => Self::RightLowerLeg,
            Self::Chest => Self::Waist,
            Self::UpperChest => Self::Chest,
            Self::LeftShoulder | Self::RightShoulder => Self::UpperChest,
            Self::LeftUpperArm => Self::LeftShoulder,
            Self::LeftLowerArm => Self::LeftShoulder,
            Self::LeftHand => Self::LeftLowerArm,
            Self::RightUpperArm => Self::RightShoulder,
            Self::RightLowerArm => Self::RightShoulder,
            Self::RightHand => Self::RightLowerArm,
            Self::Neck => Self::UpperChest,
            Self::Head => Self::Neck,
        })
    }

    /// Gets a vector of the head to the tail of the bone if the head is at the origin
    pub fn get_tail_offset(&self, offsets: &HashMap<BoneOffsetKind, f32>) -> glam::Vec3A {
        use BoneOffsetKind::*;

        match self {
            Self::Hip => glam::vec3a(0., offsets[&HipLength], 0.),
            Self::Waist => glam::vec3a(0., offsets[&WaistLength], 0.),
            Self::LeftHip => glam::vec3a(-offsets[&HipsWidth] / 2., 0., 0.),
            Self::RightHip => glam::vec3a(offsets[&HipsWidth] / 2., 0., 0.),
            Self::LeftUpperLeg | Self::RightUpperLeg => {
                glam::vec3a(0., -offsets[&UpperLegLength], 0.)
            }
            Self::LeftLowerLeg | Self::RightLowerLeg => {
                glam::vec3a(0., -offsets[&LowerLegLength], 0.)
            }
            Self::LeftFoot | Self::RightFoot => glam::vec3a(0., -offsets[&FootLength], 0.),
            Self::Chest => glam::vec3a(0., offsets[&ChestLength], 0.),
            Self::UpperChest => glam::vec3a(0., offsets[&UpperChestLength], 0.),
            Self::LeftShoulder => {
                glam::vec3a(-offsets[&ShouldersWidth] / 2., offsets[&ShoulderOffset], 0.)
            }
            Self::RightShoulder => {
                glam::vec3a(offsets[&ShouldersWidth] / 2., offsets[&ShoulderOffset], 0.)
            }
            Self::LeftUpperArm | Self::RightUpperArm => {
                glam::vec3a(0., -offsets[&UpperArmLength], 0.)
            }
            Self::LeftLowerArm | Self::RightLowerArm => {
                glam::vec3a(0., -offsets[&LowerArmLength], 0.)
            }
            Self::LeftHand | Self::RightHand => glam::vec3a(0., -offsets[&HandLength], 0.),
            Self::Neck => glam::vec3a(0., -offsets[&NeckLength], 0.),
            Self::Head => glam::vec3a(0., 0., 0.),
        }
    }
}

#[derive(Serialize, TS)]
pub struct Bone {
    /// Positional offset of the joint
    #[ts(type = "[number, number, number]")]
    pub tail_offset: glam::Vec3A,
    /// Orientation of joint
    #[ts(type = "[number, number, number, number]")]
    pub orientation: glam::Quat,
    pub location: BoneLocation,
}

impl Bone {
    pub fn new(location: BoneLocation) -> Self {
        Self {
            tail_offset: glam::Vec3A::ZERO,
            orientation: glam::Quat::IDENTITY,
            location,
        }
    }

    pub fn set_tail_offset(&mut self, offsets: &HashMap<BoneOffsetKind, f32>) {
        self.tail_offset = self.location.get_tail_offset(offsets);
    }

    pub fn get_head_position(&self, bones: &HashMap<BoneLocation, Bone>) -> glam::Vec3A {
        if let Some(location) = self.location.get_parent() {
            bones[&location].tail_offset
        } else {
            glam::Vec3A::ZERO
        }
    }
}
