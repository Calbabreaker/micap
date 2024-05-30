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
    pub index: u8,
    pub status: TrackerStatus,
    pub orientation: Quaternion,
    pub acceleration: Vector3,
}

#[derive(Default)]
pub struct Device {
    pub trackers: Vec<Tracker>,
}

impl Device {
    pub fn get_tracker_mut(&mut self, index: u8) -> &mut Tracker {
        if index as usize >= self.trackers.len() {
            self.trackers
                .resize_with((index + 1) as usize, Default::default);
        }

        &mut self.trackers[index as usize]
    }
}

#[derive(Default)]
pub struct ServerState {
    pub devices: Vec<Device>,
}
