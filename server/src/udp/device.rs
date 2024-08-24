use std::{net::SocketAddr, time::Instant};
use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::{Tracker, TrackerConfig, TrackerData, TrackerStatus},
    udp::packet::{
        UdpPacketBatteryLevel, UdpPacketPingPong, UdpPacketTrackerStatus, UdpTrackerData,
    },
};

#[derive(Debug)]
pub struct UdpDevice {
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
    /// Maps the udp device's tracker index to the tracker's global id
    pub(super) tracker_ids: Vec<String>,
    pub(super) timed_out: bool,
    pub(super) mac: String,
    pub(super) address: SocketAddr,
    current_ping_start_time: Option<Instant>,
    current_ping_id: u8,
}

impl UdpDevice {
    pub fn new(address: SocketAddr, mac: String) -> Self {
        Self {
            tracker_ids: Vec::default(),
            address,
            mac,
            last_packet_received_time: Instant::now(),
            last_packet_number: 0,
            timed_out: false,
            current_ping_id: 0,
            current_ping_start_time: None,
        }
    }

    fn add_global_tracker(&mut self, local_index: u8, main: &mut MainServer) {
        if local_index as usize >= self.tracker_ids.len() {
            self.tracker_ids
                .resize(local_index as usize + 1, String::new());
        }

        // Register the tracker and add the id into the udp device array to know
        let id = format!("{}/{}", self.mac, local_index);
        let name = format!("UDP {}/{}", self.address, local_index);
        let config = TrackerConfig::new(name);
        main.add_tracker(id.clone(), Tracker::new(config));
        if let Err(err) = main.save_config() {
            log::error!("Failed to save tracker: {err:?}");
        }
        self.tracker_ids[local_index as usize] = id;
    }

    fn get_tracker_id(&mut self, local_index: u8) -> Option<&String> {
        self.tracker_ids
            .get(local_index as usize)
            .filter(|id| !id.is_empty())
    }

    fn tracker_id_iter(&self) -> impl Iterator<Item = &String> {
        self.tracker_ids.iter().filter(|id| !id.is_empty())
    }

    pub fn is_latest_packet_number(&mut self, packet_number: u32) -> bool {
        // Pass through packet with 0 packet number (eg. handshakes)
        if packet_number == 0 {
            return true;
        }

        if packet_number <= self.last_packet_number {
            false
        } else {
            self.last_packet_number = packet_number;
            true
        }
    }

    pub fn set_timed_out(&mut self, main: &mut MainServer, timed_out: bool) {
        if timed_out == self.timed_out {
            return;
        }

        self.timed_out = timed_out;

        for id in self.tracker_id_iter() {
            if let Some(tracker) = main.tracker_info_update(id) {
                // Only allow changing status to TimedOut if tracker is Ok and vice-versa
                if timed_out && tracker.info.status == TrackerStatus::Ok {
                    tracker.info.status = TrackerStatus::TimedOut;
                } else if !timed_out && tracker.info.status == TrackerStatus::TimedOut {
                    tracker.info.status = TrackerStatus::Ok;
                };
            }
        }
    }

    pub async fn check_send_new_ping(&mut self, socket: &UdpSocket) -> anyhow::Result<()> {
        // If ping has been acknowledge (when set to none) start a new ping id
        if self.current_ping_start_time.is_none() {
            self.current_ping_start_time = Some(Instant::now());
            self.current_ping_id = self.current_ping_id.wrapping_add(1);
        }

        let ping_packet = UdpPacketPingPong::to_bytes(self.current_ping_id);
        socket.send_to(&ping_packet, self.address).await?;
        Ok(())
    }

    pub fn handle_pong(&mut self, main: &mut MainServer, packet: UdpPacketPingPong) {
        if packet.id != self.current_ping_id {
            return;
        }

        if let Some(start_time) = self.current_ping_start_time {
            for id in self.tracker_id_iter() {
                let latency = start_time.elapsed() / 2;
                if let Some(tracker) = main.tracker_info_update(id) {
                    tracker.info.latency_ms = Some(latency.as_millis() as u32);
                }
            }

            self.current_ping_start_time = None;
        }
    }

    pub fn update_tracker_data(&mut self, main: &mut MainServer, data: UdpTrackerData) {
        if let Some(id) = self.get_tracker_id(data.tracker_index) {
            main.update_tracker_data(id, data.accleration, data.orientation);
        }
    }

    pub fn update_tracker_status(&mut self, main: &mut MainServer, packet: UdpPacketTrackerStatus) {
        if self.get_tracker_id(packet.tracker_index).is_none() {
            self.add_global_tracker(packet.tracker_index, main);
        }

        let id = self.get_tracker_id(packet.tracker_index).unwrap();
        if let Some(tracker) = main.tracker_info_update(id) {
            tracker.info.status = packet.tracker_status;
            tracker.data = TrackerData::default();
        }
    }

    pub fn update_battery_level(&self, main: &mut MainServer, packet: UdpPacketBatteryLevel) {
        for id in self.tracker_id_iter() {
            if let Some(tracker) = main.tracker_info_update(id) {
                tracker.info.battery_level = packet.battery_level;
            }
        }
    }
}
