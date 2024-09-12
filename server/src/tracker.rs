use std::{net::SocketAddr, time::Instant};

use crate::bone::BoneKind;

#[derive(Default, PartialEq, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TrackerInfo {
    pub status: TrackerStatus,
    pub latency_ms: Option<u32>,
    pub battery_level: f32,
    pub address: Option<SocketAddr>,
    #[serde(skip)]
    pub was_updated: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: glam::Quat,
    pub acceleration: glam::Vec3A,
    #[serde(skip)]
    pub velocity: glam::Vec3A,
    pub position: glam::Vec3A,
    #[serde(skip)]
    pub was_updated: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Tracker {
    pub info: TrackerInfo,
    pub data: TrackerData,
    #[serde(skip)]
    pub to_be_removed: bool,
    #[serde(skip)]
    pub time_data_last_updated: Instant,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            info: TrackerInfo::default(),
            data: TrackerData::default(),
            to_be_removed: false,
            time_data_last_updated: Instant::now(),
        }
    }
}

impl Tracker {
    pub fn update_data(&mut self, acceleration: glam::Vec3A, orientation: glam::Quat) {
        self.data.orientation = orientation;
        self.data.acceleration = acceleration;

        if self.info.status == TrackerStatus::Ok {
            let delta = self.time_data_last_updated.elapsed().as_secs_f32();
            self.data.velocity += self.data.acceleration * delta;
            self.data.position += self.data.velocity * delta;
        }

        self.time_data_last_updated = Instant::now();
        self.data.was_updated = true;
    }

    pub fn update_info(&mut self) -> &mut TrackerInfo {
        self.info.was_updated = true;
        &mut self.info
    }
}

/// Seperate from TrackerInfo to be used to save to a file
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<BoneKind>,
}

impl TrackerConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
