use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Offset type for a specific body part used to offset the bone (joints) in meters
/// See BoneLocation::get_offset
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum BoneOffsetKind {
    /// Y distance from base of neck to eyes
    NeckLength,
    /// Y distance from top of hip to base of chest
    WaistLength,
    /// Y distance from base of chest to base of upper chest
    ChestLength,
    /// Y distance from upper chest to base of neck
    UpperChestLength,
    /// Y distance from base of hip to top
    HipLength,
    HipsWidth,
    /// Y distance from upper leg to lower leg
    UpperLegLength,
    /// Y distance from lower leg to foot
    LowerLegLength,
    ShouldersWidth,
    /// Y distance from upper chest to shoulders
    ShoulderOffset,
    /// Y distance from upper arm to lower arm
    UpperArmLength,
    /// Y distance from lower arm to wrist
    LowerArmLength,
    FootLength,
    HandLength,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(default)]
pub struct SkeletonConfig {
    /// Contains the length offset in meters from a bone to its connecting one
    pub offsets: HashMap<BoneOffsetKind, f32>,
}

impl Default for SkeletonConfig {
    fn default() -> Self {
        use BoneOffsetKind::*;
        Self {
            // Some default values for an average body probably
            offsets: HashMap::from([
                (NeckLength, 0.1),
                (WaistLength, 0.2),
                (ChestLength, 0.1),
                (UpperChestLength, 0.1),
                (HipLength, 0.1),
                (HipsWidth, 0.3),
                (UpperLegLength, 0.2),
                (LowerLegLength, 0.3),
                (ShouldersWidth, 0.3),
                (ShoulderOffset, 0.1),
                (UpperArmLength, 0.3),
                (LowerArmLength, 0.3),
                (FootLength, 0.1),
                (HandLength, 0.1),
            ]),
        }
    }
}
