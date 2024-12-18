use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};
use ts_rs::TS;

use crate::skeleton::BoneLocation;

#[derive(Default, PartialEq, Clone, Copy, Serialize, Debug, TS)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Clone, Debug, Default, Serialize, TS)]
pub struct TrackerInfo {
    pub to_be_removed: bool,
    pub status: TrackerStatus,
    #[ts(optional)]
    pub latency_ms: Option<u32>,
    pub battery_level: f32,
    #[ts(optional)]
    pub address: Option<SocketAddr>,
}

#[derive(Default, Debug, Serialize, TS)]
pub struct TrackerData {
    #[ts(type = "[number, number, number, number]")]
    pub orientation: glam::Quat,
    #[ts(type = "[number, number, number]")]
    pub acceleration: glam::Vec3A,
    #[ts(type = "[number, number, number]")]
    pub position: glam::Vec3A,
}

#[derive(Debug)]
pub struct TrackerInternal {
    pub time_data_last_updated: Instant,
    pub velocity: glam::Vec3A,
    pub was_updated: bool,
    /// Offset orientation from when skeleton orientation was reset
    pub orientation_offset: glam::Quat,
    pub mounted_orientation: glam::Quat,
    pub mount_offset: glam::Quat,
}

impl Default for TrackerInternal {
    fn default() -> Self {
        Self {
            time_data_last_updated: Instant::now(),
            velocity: glam::Vec3A::default(),
            was_updated: false,
            orientation_offset: glam::Quat::IDENTITY,
            mounted_orientation: glam::Quat::IDENTITY,
            mount_offset: glam::Quat::IDENTITY,
        }
    }
}

// We're technically not multithreading but we doing async stuff so this needs to by Sync + Send
pub type TrackerRef = std::sync::Arc<std::sync::Mutex<Tracker>>;

#[derive(Debug, Default, Serialize, TS)]
pub struct Tracker {
    info: TrackerInfo,
    data: TrackerData,
    #[serde(skip)]
    pub internal: TrackerInternal,
}

impl Tracker {
    pub fn update_data(&mut self, raw_acceleration: glam::Vec3A, raw_orientation: glam::Quat) {
        let mounted_orientation = raw_orientation * self.internal.mount_offset;
        self.internal.mounted_orientation = mounted_orientation;
        self.data.orientation = self.internal.orientation_offset * mounted_orientation;
        self.data.acceleration = raw_acceleration;

        let delta = self.internal.time_data_last_updated.elapsed().as_secs_f32();
        self.internal.velocity += self.data.acceleration * delta;
        self.data.position += self.internal.velocity * delta;

        self.internal.time_data_last_updated = Instant::now();
        self.internal.was_updated = true;
    }

    pub fn reset_data(&mut self) {
        self.data = TrackerData::default();
    }

    pub fn reset_orientation(&mut self) {
        self.internal.orientation_offset = self.internal.mounted_orientation.inverse();
    }

    pub fn set_mount_offset(&mut self, location: BoneLocation) {
        use BoneLocation::*;
        let rotation = match location {
            LeftUpperArm | LeftLowerArm | LeftHand => {
                glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(-90.))
            }
            RightUpperArm | RightLowerArm | RightHand => {
                glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.))
                    * glam::Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(180.))
            }
            _ => glam::Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(270.)),
        };

        self.internal.mount_offset = rotation;
    }

    pub fn update_info(&mut self) -> &mut TrackerInfo {
        self.internal.was_updated = true;
        &mut self.info
    }

    pub fn info(&self) -> &TrackerInfo {
        &self.info
    }

    pub fn data(&self) -> &TrackerData {
        &self.data
    }

    pub fn set_timed_out(&mut self, timed_out: bool) {
        if self.info.status == TrackerStatus::Ok || self.info.status == TrackerStatus::TimedOut {
            self.update_info().status = if timed_out {
                TrackerStatus::TimedOut
            } else {
                TrackerStatus::Ok
            }
        }
    }
}

// Seperated from TrackerInfo to be used to save to a file
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct TrackerConfig {
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub location: Option<BoneLocation>,
}
