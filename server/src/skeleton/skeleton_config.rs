use std::collections::HashMap;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SkeletonOffsetKind {
    NeckLength,
    WaistLength,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SkeletonConfig {
    /// Contains the length offset in meters from a bone to its connecting one
    pub offsets: HashMap<SkeletonOffsetKind, f32>,
}

impl Default for SkeletonConfig {
    fn default() -> Self {
        use SkeletonOffsetKind::*;

        Self {
            offsets: HashMap::from([(NeckLength, 0.1)]),
        }
    }
}
