use std::net::{IpAddr, SocketAddr};

use crate::math::{Quaternion, Vector3};

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
}

#[derive(Default)]
pub struct Tracker {
    pub id: u8,
    pub status: TrackerStatus,
    pub orientation: Quaternion,
    pub acceleration: Vector3,
}

#[derive(Default)]
pub struct Device {
    pub trackers: Vec<Tracker>,
}

impl Device {
    pub fn get_tracker_mut(&mut self, id: u8) -> &mut Tracker {
        if id as usize >= self.trackers.len() {
            self.trackers
                .resize_with((id + 1) as usize, Default::default);
        }

        &mut self.trackers[id as usize]
    }
}

#[derive(Default)]
pub struct ServerState {
    pub devices: Vec<Device>,
}
