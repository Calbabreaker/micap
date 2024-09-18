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

    // Maps a bone location to its parent
    pub const SELF_TO_PARENT: &[(Self, Option<Self>)] = &[
        (Self::Hip, None),
        (Self::Waist, Some(Self::Hip)),
        (Self::LeftHip, Some(Self::Hip)),
        (Self::RightHip, Some(Self::Hip)),
        (Self::LeftUpperLeg, Some(Self::LeftHip)),
        (Self::LeftLowerLeg, Some(Self::LeftUpperLeg)),
        (Self::LeftFoot, Some(Self::LeftLowerLeg)),
        (Self::RightUpperLeg, Some(Self::RightHip)),
        (Self::RightLowerLeg, Some(Self::RightUpperLeg)),
        (Self::RightFoot, Some(Self::RightLowerLeg)),
        (Self::Chest, Some(Self::Waist)),
        (Self::UpperChest, Some(Self::Chest)),
        (Self::LeftShoulder, Some(Self::UpperChest)),
        (Self::RightShoulder, Some(Self::UpperChest)),
        (Self::LeftUpperArm, Some(Self::LeftShoulder)),
        (Self::LeftLowerArm, Some(Self::LeftShoulder)),
        (Self::LeftHand, Some(Self::LeftLowerArm)),
        (Self::RightUpperArm, Some(Self::RightShoulder)),
        (Self::RightLowerArm, Some(Self::RightShoulder)),
        (Self::RightHand, Some(Self::RightLowerArm)),
        (Self::Neck, Some(Self::UpperChest)),
        (Self::Head, Some(Self::Neck)),
    ];
}

#[derive(Serialize, TS)]
pub struct Bone {
    /// Positional offset of the joint
    #[ts(type = "[number, number, number]")]
    pub tail_offset: glam::Vec3A,
    /// Orientation of joint
    #[ts(type = "[number, number, number, number]")]
    pub orientation: glam::Quat,
    pub parent: Option<BoneLocation>,
}

impl Bone {
    pub fn new(parent: Option<BoneLocation>) -> Self {
        Self {
            tail_offset: glam::Vec3A::ZERO,
            orientation: glam::Quat::IDENTITY,
            parent,
        }
    }

    pub fn set_tail_offset(
        &mut self,
        location: BoneLocation,
        offsets: &HashMap<BoneOffsetKind, f32>,
    ) {
        self.tail_offset = location.get_tail_offset(offsets);
    }

    pub fn get_head_position(&self, bones: &HashMap<BoneLocation, Bone>) -> glam::Vec3A {
        if let Some(location) = self.parent {
            bones[&location].tail_offset
        } else {
            glam::Vec3A::ZERO
        }
    }
}
