use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Offset type for a specific body part used to offset the bone (joints) in meters
/// See BoneLocation::get_offset
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub enum BoneOffsetKind {
    /// Y distance from base of head to top of head
    HeadLength,
    /// Y distance from base of neck to base of head
    NeckLength,
    /// Y distance from top of hip to base of chest
    WaistLength,
    /// Y distance from base of chest to base of upper chest
    ChestLength,
    /// Y distance from upper chest to base of neck
    UpperChestLength,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct SkeletonConfig {
    /// Contains the length offset in meters from a bone to its connecting one
    pub offsets: HashMap<BoneOffsetKind, f32>,
    pub user_height: f32,
}

impl Default for SkeletonConfig {
    fn default() -> Self {
        use BoneOffsetKind::*;
        let mut this = Self {
            // Some default values for an average body probably
            offsets: HashMap::from([
                (HeadLength, 0.24),
                (NeckLength, 0.10),
                (WaistLength, 0.20),
                (ChestLength, 0.18),
                (UpperChestLength, 0.12),
                (HipsWidth, 0.36),
                (UpperLegLength, 0.45),
                (LowerLegLength, 0.44),
                (ShouldersWidth, 0.40),
                (ShoulderOffset, 0.08),
                (UpperArmLength, 0.30),
                (LowerArmLength, 0.26),
                (FootLength, 0.26),
                (HandLength, 0.18),
            ]),
            user_height: 0.0,
        };
        this.user_height = this.get_total_height();
        this
    }
}

impl SkeletonConfig {
    fn get_offset_sum(&self, offset_kinds: &[BoneOffsetKind]) -> f32 {
        offset_kinds.iter().map(|kind| self.offsets[kind]).sum()
    }

    pub fn get_leg_length(&self) -> f32 {
        use BoneOffsetKind::*;
        self.get_offset_sum(&[UpperLegLength, LowerLegLength])
    }

    pub fn get_total_height(&self) -> f32 {
        use BoneOffsetKind::*;
        self.get_offset_sum(&[
            LowerLegLength,
            UpperLegLength,
            ChestLength,
            UpperChestLength,
            NeckLength,
            HeadLength,
        ])
    }

    pub fn update_height(&mut self) {
        let scale = self.user_height / self.get_total_height();
        for offset in self.offsets.values_mut() {
            *offset *= scale;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::skeleton::SkeletonConfig;

    #[test]
    fn height_update_correct() {
        let mut skel_conf = SkeletonConfig {
            user_height: 1.2,
            ..Default::default()
        };
        skel_conf.update_height();
        assert!(
            skel_conf.get_total_height() > skel_conf.user_height - 0.001
                && skel_conf.get_total_height() < skel_conf.user_height + 0.001
        );
    }
}
