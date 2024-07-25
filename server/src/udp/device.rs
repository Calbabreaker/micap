use std::{collections::HashMap, net::SocketAddr, time::Instant};
use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::{TrackerConfig, TrackerData, TrackerStatus},
    udp::packet::{
        UdpPacketBatteryLevel, UdpPacketPingPong, UdpPacketTrackerStatus, UdpTrackerData,
    },
};

#[derive(Debug)]
pub struct UdpDevice {
    index: usize,
    pub(super) last_packet_received_time: Instant,
    last_packet_number: u32,
    /// Maps the udp device's tracker index to the tracker's global index
    tracker_indexs: Vec<usize>,
    timed_out: bool,
    mac: String,
    address: SocketAddr,
    current_ping_start_time: Option<Instant>,
    current_ping_id: u8,
}

impl UdpDevice {
    pub fn new(index: usize, address: SocketAddr, mac: String) -> Self {
        Self {
            tracker_indexs: Vec::default(),
            index,
            address,
            mac,
            last_packet_received_time: Instant::now(),
            last_packet_number: 0,
            timed_out: false,
            current_ping_id: 0,
            current_ping_start_time: None,
        }
    }

    fn set_global_tracker_index(&mut self, local_index: u8, global_index: usize) {
        if local_index as usize >= self.tracker_indexs.len() {
            self.tracker_indexs
                .resize_with(local_index as usize + 1, usize::default);
        }

        self.tracker_indexs[local_index as usize] = global_index;
    }

    fn get_global_tracker_index(&mut self, main: &mut MainServer, local_index: u8) -> usize {
        match self.tracker_indexs.get(local_index as usize) {
            Some(index) => *index,
            None => {
                // Register the tracker and add the index into the udp device array to know
                let id = format!("{}/{}", self.mac, local_index);
                let name = format!("UDP Tracker {}", self.address);
                let index = main.register_tracker(
                    id,
                    TrackerConfig {
                        name,
                        ..Default::default()
                    },
                );
                self.set_global_tracker_index(local_index, index);
                index
            }
        }
    }

    pub fn latest_packet_number(&mut self, packet_number: u32) -> bool {
        if packet_number <= self.last_packet_number {
            return false;
        }
        self.last_packet_number = packet_number;
        true
    }

    pub fn set_timed_out(&mut self, main: &mut MainServer, timed_out: bool) {
        if timed_out == self.timed_out {
            return;
        }

        self.timed_out = timed_out;

        for global_index in &self.tracker_indexs {
            let info = &mut main.trackers[*global_index].info;

            // Only allow changing status to TimedOut if tracker is Ok and vice-versa
            if timed_out && info.status == TrackerStatus::Ok {
                info.status = TrackerStatus::TimedOut;
                main.tracker_info_updated(*global_index);
            } else if !timed_out && info.status == TrackerStatus::TimedOut {
                info.status = TrackerStatus::Ok;
                main.tracker_info_updated(*global_index);
            };
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
            for global_index in &self.tracker_indexs {
                let latency = start_time.elapsed() / 2;
                main.trackers[*global_index].info.latency_ms = Some(latency.as_millis() as u32);
                main.tracker_info_updated(*global_index);
            }

            self.current_ping_start_time = None;
        }
    }

    pub fn check_address_handshake(
        &mut self,
        peer_addr: SocketAddr,
        address_index_map: &mut HashMap<SocketAddr, usize>,
    ) {
        // Move over to the new address if the device has a new ip
        if self.address != peer_addr {
            log::info!("Reconnected from {peer_addr} from old: {}", self.address);
            address_index_map.remove(&self.address);
            address_index_map.insert(peer_addr, self.index);
            self.address = peer_addr;
            self.last_packet_number = 0;
        } else if self.timed_out {
            log::info!("Reconnected from {peer_addr}");
            self.last_packet_number = 0;
        } else {
            log::warn!("Received handshake packet while already connected");
        }
    }

    pub fn update_tracker_data(&mut self, main: &mut MainServer, data: UdpTrackerData) {
        let global_index = self.get_global_tracker_index(main, data.tracker_index);
        main.update_tracker_data(global_index, data.accleration, data.orientation);
    }

    pub fn update_tracker_status(&mut self, main: &mut MainServer, packet: UdpPacketTrackerStatus) {
        let global_index = self.get_global_tracker_index(main, packet.tracker_index);
        main.trackers[global_index].info.status = packet.tracker_status;
        main.trackers[global_index].data = TrackerData::default();
        main.tracker_info_updated(global_index);
    }

    pub fn update_battery_level(&self, main: &mut MainServer, packet: UdpPacketBatteryLevel) {
        for global_index in &self.tracker_indexs {
            main.trackers[*global_index].info.battery_level = Some(packet.battery_level);
            main.tracker_info_updated(*global_index);
        }
    }
}
