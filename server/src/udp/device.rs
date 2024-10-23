use std::{
    net::SocketAddr,
    sync::{Arc, MutexGuard},
    time::{Duration, Instant},
};

use crate::{
    main_server::MainServer,
    tracker::{Tracker, TrackerRef},
    udp::packet::{
        UdpPacketBatteryLevel, UdpPacketPingPong, UdpPacketTrackerStatus, UdpTrackerData,
    },
};

pub struct UdpDevice {
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
    pub(super) global_trackers: Vec<Option<TrackerRef>>,
    pub(super) mac: Arc<str>,
    pub(super) address: SocketAddr,
    current_ping_start_time: Option<Instant>,
    current_ping_id: u8,
}

impl UdpDevice {
    const TIMEOUT: Duration = Duration::from_millis(2000);

    pub fn new(address: SocketAddr, mac: Arc<str>) -> Self {
        Self {
            global_trackers: Vec::default(),
            address,
            mac,
            last_packet_received_time: Instant::now(),
            last_packet_number: 0,
            current_ping_id: 0,
            current_ping_start_time: None,
        }
    }

    fn add_global_tracker(&mut self, local_index: u8, main: &mut MainServer) {
        let local_index = local_index as usize;
        if local_index >= self.global_trackers.len() {
            self.global_trackers.resize(local_index + 1, None);
        }

        // Register the tracker and add the id into the udp device array to know
        let id: Arc<str> = format!("{}/{}", self.mac, local_index).into();
        self.global_trackers[local_index] = main.add_tracker(&id);
    }

    fn get_tracker(&self, local_index: u8) -> Option<MutexGuard<'_, Tracker>> {
        Some(
            self.global_trackers
                .get(local_index as usize)?
                .as_ref()?
                .lock()
                .unwrap(),
        )
    }

    fn global_trackers_iter(&self) -> impl Iterator<Item = MutexGuard<'_, Tracker>> {
        self.global_trackers
            .iter()
            .filter_map(|tracker| Some(tracker.as_ref()?.lock().unwrap()))
    }

    pub fn check_latest_packet_number(&mut self, packet_number: u32) -> bool {
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

    pub fn is_timed_out(&self) -> bool {
        self.last_packet_received_time.elapsed() > Self::TIMEOUT
    }

    pub fn update_timed_out(&mut self, timed_out: bool) {
        for mut tracker in self.global_trackers_iter() {
            tracker.set_timed_out(timed_out);
        }
    }

    // Gets the ping packet with the current ping id
    pub fn check_get_ping_packet(&mut self) -> UdpPacketPingPong {
        // If ping has been acknowledge (when set to none) start a new ping id
        if self.current_ping_start_time.is_none() {
            self.current_ping_start_time = Some(Instant::now());
            self.current_ping_id = self.current_ping_id.wrapping_add(1);
        }

        UdpPacketPingPong::new(self.current_ping_id)
    }

    pub fn handle_pong(&mut self, packet: UdpPacketPingPong) {
        if packet.id != self.current_ping_id {
            return;
        }

        if let Some(start_time) = self.current_ping_start_time.take() {
            for mut tracker in self.global_trackers_iter() {
                let latency = start_time.elapsed() / 2;
                tracker.update_info().latency_ms = Some(latency.as_millis() as u32);
            }
        }
    }

    pub fn update_tracker_data(&mut self, data: UdpTrackerData) {
        if let Some(mut tracker) = self.get_tracker(data.tracker_index) {
            tracker.update_data(data.acceleration, data.orientation);
        }
    }

    pub fn update_tracker_status(&mut self, main: &mut MainServer, packet: UdpPacketTrackerStatus) {
        if self.get_tracker(packet.tracker_index).is_none() {
            self.add_global_tracker(packet.tracker_index, main);
        }

        let address = self.address;
        if let Some(mut tracker) = self.get_tracker(packet.tracker_index) {
            tracker.reset_data();
            tracker.update_info().status = packet.tracker_status;
            tracker.update_info().address = Some(address);
        }
    }

    pub fn update_battery_level(&self, packet: UdpPacketBatteryLevel) {
        for mut tracker in self.global_trackers_iter() {
            tracker.update_info().battery_level = packet.battery_level;
        }
    }

    pub fn all_trackers_removed(&mut self) -> bool {
        let mut count = 0;
        let all_removed = self
            .global_trackers_iter()
            .inspect(|_| count += 1)
            .all(|tracker| tracker.info().to_be_removed);
        count != 0 && all_removed
    }
}
