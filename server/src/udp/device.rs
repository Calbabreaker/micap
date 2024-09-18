use futures_util::{Stream, StreamExt};
use std::{net::SocketAddr, time::Instant};
use tokio::sync::RwLockWriteGuard;

use crate::{
    main_server::{MainServer, TrackerRef},
    tracker::{Tracker, TrackerData, TrackerStatus},
    udp::packet::{
        UdpPacketBatteryLevel, UdpPacketPingPong, UdpPacketTrackerStatus, UdpTrackerData,
    },
};

#[derive(Debug)]
pub struct UdpDevice {
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
    /// Maps the udp device's tracker index to the global tracker
    pub(super) global_trackers: Vec<Option<TrackerRef>>,
    pub(super) timed_out: bool,
    pub(super) mac: String,
    pub(super) address: SocketAddr,
    current_ping_start_time: Option<Instant>,
    current_ping_id: u8,
}

impl UdpDevice {
    pub fn new(address: SocketAddr, mac: String) -> Self {
        Self {
            global_trackers: Vec::default(),
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
        if local_index as usize >= self.global_trackers.len() {
            self.global_trackers.resize(local_index as usize + 1, None);
        }

        // Register the tracker and add the id into the udp device array to know
        let id = format!("{}/{}", self.mac, local_index);
        main.add_tracker(id.clone());

        self.global_trackers[local_index as usize] = main.trackers.get(&id).cloned();
    }

    async fn get_tracker(&mut self, local_index: u8) -> Option<RwLockWriteGuard<'_, Tracker>> {
        Some(
            self.global_trackers
                .get(local_index as usize)?
                .as_ref()?
                .write()
                .await,
        )
    }

    fn global_trackers_iter(&self) -> impl Stream<Item = RwLockWriteGuard<Tracker>> {
        futures_util::stream::iter(self.global_trackers.iter())
            .filter_map(|tracker| async { Some(tracker.as_ref()?.write().await) })
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

    pub async fn set_timed_out(&mut self, timed_out: bool) {
        if timed_out == self.timed_out {
            return;
        }

        self.timed_out = timed_out;

        self.global_trackers_iter()
            .for_each(|mut tracker| async move {
                // Only allow changing status to TimedOut if tracker is Ok and vice-versa
                if timed_out && tracker.info.status == TrackerStatus::Ok {
                    tracker.update_info().status = TrackerStatus::TimedOut;
                } else if !timed_out && tracker.info.status == TrackerStatus::TimedOut {
                    tracker.update_info().status = TrackerStatus::Ok;
                };
            })
            .await;
    }

    pub fn check_get_ping_packet(&mut self) -> UdpPacketPingPong {
        // If ping has been acknowledge (when set to none) start a new ping id
        if self.current_ping_start_time.is_none() {
            self.current_ping_start_time = Some(Instant::now());
            self.current_ping_id = self.current_ping_id.wrapping_add(1);
        }

        UdpPacketPingPong::new(self.current_ping_id)
    }

    pub async fn handle_pong(&mut self, packet: UdpPacketPingPong) {
        if packet.id != self.current_ping_id {
            return;
        }

        if let Some(start_time) = self.current_ping_start_time {
            self.global_trackers_iter()
                .for_each(|mut tracker| async move {
                    let latency = start_time.elapsed() / 2;
                    tracker.update_info().latency_ms = Some(latency.as_millis() as u32);
                })
                .await;

            self.current_ping_start_time = None;
        }
    }

    pub async fn update_tracker_data(&mut self, data: UdpTrackerData) {
        if let Some(mut tracker) = self.get_tracker(data.tracker_index).await {
            tracker.update_data(data.accleration, data.orientation);
        }
    }

    pub async fn update_tracker_status(
        &mut self,
        main: &mut MainServer,
        packet: UdpPacketTrackerStatus,
    ) {
        if self.get_tracker(packet.tracker_index).await.is_none() {
            self.add_global_tracker(packet.tracker_index, main);
        }

        let address = self.address;
        let mut tracker = self.get_tracker(packet.tracker_index).await.unwrap();

        tracker.data = TrackerData::default();
        tracker.update_info().status = packet.tracker_status;
        tracker.update_info().address = Some(address);
    }

    pub async fn update_battery_level(&self, packet: UdpPacketBatteryLevel) {
        self.global_trackers_iter()
            .for_each(|mut tracker| async move {
                tracker.update_info().battery_level = packet.battery_level;
            })
            .await;
    }

    pub async fn all_trackers_removed(&mut self) -> bool {
        let all_removed = self
            .global_trackers_iter()
            .all(|tracker| async move { tracker.to_be_removed })
            .await;
        !self.global_trackers.is_empty() && all_removed
    }
}
