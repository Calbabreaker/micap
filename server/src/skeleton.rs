use std::collections::HashMap;

use crate::{
    bone::{Bone, BoneKind},
    main_server::TrackerRef,
    tracker::Tracker,
};

pub struct Skeleton {
    root_bone: Bone,
    trackers: HashMap<BoneKind, TrackerRef>,
}

impl Skeleton {
    pub fn update(&mut self, trackers: impl Iterator<Item = Tracker>) {}
}

// impl Default for Skeleton {
//     fn default() -> Self {
//         Self { root_bone: Bone }
//     }
// }
