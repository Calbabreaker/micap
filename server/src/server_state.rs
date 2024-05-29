use std::net::{IpAddr, SocketAddr};

use crate::math::{Quaternion, Vector3};

#[derive(Default, Debug)]
pub enum TrackerStatus {
    Ok,
    Error,
    #[default]
    Off,
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

pub struct ServerState {
    pub devices: Vec<Device>,
    local_ip: Option<IpAddr>,
}

impl ServerState {
    pub async fn new() -> Self {
        Self {
            devices: Vec::new(),
            local_ip: get_local_ip().await,
        }
    }
}

async fn get_local_ip() -> Option<IpAddr> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:12345").await.ok()?;
    socket.connect(("1.1.1.1", 80)).await.ok()?;
    let local_ip = socket.local_addr().ok()?.ip();
    log::info!("Found local ip: {local_ip}");
    Some(local_ip)
}
