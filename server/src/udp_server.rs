use anyhow::Context;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::{
    main_server::MainServer,
    tracker::{TrackerConfig, TrackerStatus},
    udp_packet::{
        UdpPacket, UdpPacketHandshake, UdpPacketTrackerData, UdpPacketTrackerStatus,
        PACKET_HEARTBEAT,
    },
};

pub const UDP_PORT: u16 = 5828;
pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

const DEVICE_TIMEOUT: Duration = Duration::from_millis(5000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);
const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

pub struct UdpDevice {
    pub(super) index: usize,
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
    /// Maps the udp device's tracker index to the tracker's global index
    tracker_indexs: Vec<usize>,
    timed_out: bool,
    mac: String,
    address: SocketAddr,
}

impl UdpDevice {
    fn new(index: usize, address: SocketAddr, mac: String) -> Self {
        Self {
            tracker_indexs: Default::default(),
            index,
            address,
            mac,
            last_packet_received_time: Instant::now(),
            last_packet_number: 0,
            timed_out: false,
        }
    }

    fn set_global_tracker_index(&mut self, local_index: u8, global_index: usize) {
        if local_index as usize >= self.tracker_indexs.len() {
            self.tracker_indexs
                .resize_with(local_index as usize + 1, usize::default);
        }

        self.tracker_indexs[local_index as usize] = global_index;
    }

    async fn get_global_tracker_index(
        &mut self,
        main: &Arc<RwLock<MainServer>>,
        local_index: u8,
    ) -> usize {
        match self.tracker_indexs.get(local_index as usize) {
            Some(index) => *index,
            None => {
                // Register the tracker and add the index into the udp device array to know
                let id = format!("{}/{}", self.mac, local_index);
                let name = format!("UDP Tracker {}", self.address);
                let index = main
                    .write()
                    .await
                    .register_tracker(id, TrackerConfig::with_name(name));
                self.set_global_tracker_index(local_index, index);
                index
            }
        }
    }

    async fn set_timed_out(&mut self, main: &Arc<RwLock<MainServer>>, timed_out: bool) {
        if timed_out == self.timed_out {
            return;
        }

        self.timed_out = timed_out;

        let mut main = main.write().await;
        for i in &self.tracker_indexs {
            let info = &main.trackers[*i].info;

            // Only allow changing status to TimedOut if tracker is Ok and vice-versa
            if timed_out && info.status == TrackerStatus::Ok {
                main.update_tracker_status(*i, TrackerStatus::TimedOut);
            } else if !timed_out && info.status == TrackerStatus::TimedOut {
                main.update_tracker_status(*i, TrackerStatus::Ok);
            }
        }
    }
}

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_to_device_index: HashMap<String, usize>,
    address_to_device_index: HashMap<SocketAddr, usize>,

    socket: UdpSocket,
    last_upkeep_time: Instant,
    main: Arc<RwLock<MainServer>>,
}

impl UdpServer {
    async fn new(main: Arc<RwLock<MainServer>>) -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)).await?;
        socket.join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)?;
        log::info!("Started UDP server on {}", socket.local_addr()?);

        Ok(Self {
            devices: Default::default(),
            mac_to_device_index: Default::default(),
            address_to_device_index: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
            main,
        })
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        let mut buffer = [0_u8; 256];

        loop {
            // Have receiving data timeout so that the upkeep check can happen continously
            if let Ok(Ok((amount, peer_addr))) =
                tokio::time::timeout(SOCKET_TIMEOUT, self.socket.recv_from(&mut buffer)).await
            {
                log::trace!(
                    "Received {amount} bytes from {peer_addr} ({:#02x})",
                    buffer[0]
                );

                // Only pass through the amount received
                self.handle_packet(&buffer[0..amount], peer_addr)
                    .await
                    .with_context(|| "Failed to handle packet")?;
            }

            if self.last_upkeep_time.elapsed() > UPKEEP_INTERVAL {
                self.upkeep()
                    .await
                    .with_context(|| "Failed to run upkeep")?;
            }
        }
    }

    async fn upkeep(&mut self) -> anyhow::Result<()> {
        for device in &mut self.devices {
            if device.last_packet_received_time.elapsed() > DEVICE_TIMEOUT {
                device.set_timed_out(&self.main, true).await;
            } else {
                device.set_timed_out(&self.main, false).await;
            }

            self.socket
                .send_to(&[PACKET_HEARTBEAT], device.address)
                .await?;
        }

        self.last_upkeep_time = Instant::now();
        Ok(())
    }

    async fn handle_packet(
        &mut self,
        bytes: &[u8],
        peer_addr: SocketAddr,
    ) -> tokio::io::Result<()> {
        let mut byte_iter = bytes.iter();
        let udp_device = self
            .address_to_device_index
            .get(&peer_addr)
            .and_then(|i| self.devices.get_mut(*i));

        match UdpPacket::parse(&mut byte_iter, udp_device) {
            Some(UdpPacket::Heartbeat) => {}
            Some(UdpPacket::Handshake(packet)) => {
                self.socket
                    .send_to(UdpPacketHandshake::RESPONSE, peer_addr)
                    .await?;
                self.handle_handshake(packet, peer_addr).await?;
            }
            Some(UdpPacket::TrackerData((packet, device_index))) => {
                self.handle_tracker_data(packet, device_index).await;
            }
            Some(UdpPacket::TrackerStatus((packet, device_index))) => {
                log::trace!("Got {:?}", packet);

                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                self.handlet_tracker_status(packet, device_index).await;
            }
            None => (),
        }

        Ok(())
    }

    async fn handle_handshake(
        &mut self,
        packet: UdpPacketHandshake,
        peer_addr: SocketAddr,
    ) -> tokio::io::Result<()> {
        // Check if the device already has connected with a mac address
        if let Some(index) = self.mac_to_device_index.get(&packet.mac_string) {
            let device = &mut self.devices[*index];
            let index = device.index;
            let old_address = device.address;

            // Move over to the new address if the device has a new ip
            if device.address != peer_addr {
                self.address_to_device_index.remove(&old_address);
                self.address_to_device_index.insert(peer_addr, index);
                device.address = peer_addr;
                log::info!("Reconnected from {peer_addr} from old: {old_address}");
            } else if device.timed_out {
                log::info!("Reconnected from {peer_addr}");
            } else {
                log::warn!("Received handshake packet while already connected");
            }

            return Ok(());
        }

        // Create a new udp device
        let index = self.devices.len();
        let device = UdpDevice::new(index, peer_addr, packet.mac_string.clone());
        self.mac_to_device_index.insert(packet.mac_string, index);
        self.address_to_device_index.insert(peer_addr, index);
        self.devices.push(device);
        log::info!("New device connected from {peer_addr}");
        Ok(())
    }

    async fn handlet_tracker_status(
        &mut self,
        packet: UdpPacketTrackerStatus,
        device_index: usize,
    ) {
        let device = &mut self.devices[device_index];

        let global_index = device
            .get_global_tracker_index(&self.main, packet.tracker_index)
            .await;

        let mut main = self.main.write().await;
        main.update_tracker_status(global_index, packet.tracker_status);
    }

    async fn handle_tracker_data<'a>(
        &mut self,
        mut packet: UdpPacketTrackerData<'a>,
        device_index: usize,
    ) {
        let device = &mut self.devices[device_index];

        while let Some(data) = packet.next() {
            let global_index = device
                .get_global_tracker_index(&self.main, data.tracker_index)
                .await;

            let mut main = self.main.write().await;
            main.update_tracker_data(global_index, data.accleration, data.orientation);
        }
    }
}

pub async fn start_server(main: Arc<RwLock<MainServer>>) -> anyhow::Result<()> {
    UdpServer::new(main).await?.run().await?;
    Ok(())
}
