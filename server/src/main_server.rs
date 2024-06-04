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
    pub id: String,
    pub index: usize,
    pub status: TrackerStatus,
    pub orientation: Quaternion,
    pub acceleration: Vector3,
}

pub enum ServerEvent {
    TrackerStatus { id: u32, status: TrackerStatus },
    TrackerData,
}

#[derive(Default)]
pub struct MainServer {
    pub trackers: Vec<Tracker>,
}

impl MainServer {
    pub fn register_tracker(&mut self, id: String) -> usize {
        let index = self.trackers.len();
        self.trackers.push(Tracker {
            id,
            index,
            ..Default::default()
        });
        index
    }

    pub fn handle_event(event: ServerEvent) {}
}
