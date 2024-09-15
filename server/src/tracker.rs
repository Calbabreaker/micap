use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};
use ts_rs::TS;

use crate::skeleton::BoneLocation;

#[derive(Default, PartialEq, Debug, Clone, Copy, Serialize, TS)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
pub struct TrackerInfo {
    pub status: TrackerStatus,
    pub latency_ms: Option<u32>,
    pub battery_level: f32,
    pub address: Option<SocketAddr>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
pub struct TrackerData {
    #[ts(type = "[number, number, number, number]")]
    pub orientation: glam::Quat,
    #[ts(type = "[number, number, number]")]
    pub acceleration: glam::Vec3A,
    #[ts(type = "[number, number, number]")]
    pub position: glam::Vec3A,

    #[serde(skip)]
    pub velocity: glam::Vec3A,
}

#[derive(Clone, Debug)]
pub struct Tracker {
    pub info: TrackerInfo,
    pub data: TrackerData,
    pub to_be_removed: bool,
    pub time_data_last_updated: Instant,
    pub info_was_updated: bool,
    pub data_was_updated: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            info: TrackerInfo::default(),
            data: TrackerData::default(),
            to_be_removed: false,
            time_data_last_updated: Instant::now(),
            info_was_updated: true,
            data_was_updated: true,
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
        self.data_was_updated = true;
    }

    pub fn update_info(&mut self) -> &mut TrackerInfo {
        self.info_was_updated = true;
        &mut self.info
    }
}

/// Seperated from TrackerInfo to be used to save to a file
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
pub struct TrackerConfig {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub location: Option<BoneLocation>,
}
