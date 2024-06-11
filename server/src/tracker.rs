use crate::math::{Quaternion, Vector3};

#[derive(Default, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerInfo {
    pub id: String,
    pub index: usize,
    pub status: TrackerStatus,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: Quaternion,
    pub acceleration: Vector3,
    pub position: Vector3,
}

#[derive(Clone)]
pub struct Tracker {
    pub info: TrackerInfo,
    pub data: TrackerData,
}

impl Tracker {
    pub fn new(id: String, index: usize) -> Self {
        Self {
            info: TrackerInfo {
                id,
                index,
                status: TrackerStatus::Off,
            },
            data: TrackerData::default(),
        }
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {}
