use std::time::{Duration, Instant};

#[derive(Default, PartialEq, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Default, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TrackerLocation {
    /// Not attached to any body part, free to move anywhere
    #[default]
    Free,
    Head,
    Hand,
    // TODO: add more locations
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerInfo {
    pub index: usize,
    pub status: TrackerStatus,
    pub config: TrackerConfig,
    pub latency_ms: Option<u32>,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: glam::Quat,
    pub acceleration: glam::Vec3A,
    pub velocity: glam::Vec3A,
    pub position: glam::Vec3A,
}

#[derive(Clone)]
pub struct Tracker {
    pub id: String,
    pub info: TrackerInfo,
    pub data: TrackerData,
}

impl Tracker {
    pub fn new(id: String, index: usize, config: TrackerConfig) -> Self {
        Self {
            info: TrackerInfo {
                index,
                config,
                status: TrackerStatus::default(),
                latency_ms: None,
            },
            id,
            data: TrackerData::default(),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.data.position += self.data.velocity * delta.as_secs_f32();
    }
}

/// Seperate from TrackerInfo to be used to save to a file
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    pub location: TrackerLocation,
}
