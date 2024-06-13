use std::{default, time::Duration};

use crate::math::{Quaternion, Vector3};

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
    pub id: String,
    pub index: usize,
    pub status: TrackerStatus,
    pub config: TrackerConfig,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: Quaternion,
    pub acceleration: Vector3,
    pub velocity: Vector3,
    pub position: Vector3,
}

#[derive(Clone)]
pub struct Tracker {
    pub info: TrackerInfo,
    pub data: TrackerData,
}

impl Tracker {
    pub fn new(id: String, index: usize, config: TrackerConfig) -> Self {
        Self {
            info: TrackerInfo {
                id,
                index,
                config,
                status: TrackerStatus::default(),
            },
            data: TrackerData::default(),
        }
    }

    pub fn tick(&mut self, delta: Duration) {}
}

/// Seperate from TrackerInfo to be used to save to a file
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    pub location: TrackerLocation,
}

impl TrackerConfig {
    pub fn with_name(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
