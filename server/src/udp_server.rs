use std::{
    collections::HashMap,
    fmt::format,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::{
    main_server::MainServer,
    udp_packet::{UdpPacket, UdpPacketHandshake, UdpPacketTrackerStatus, PACKET_HEARTBEAT},
};

pub const UDP_PORT: u16 = 5828;
const DEVICE_TIMEOUT: Duration = Duration::from_millis(5000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);
const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

pub struct UdpDevice {
    // Maps a udp tracker index to the tracker's global index
    tracker_indexs: Vec<usize>,
    index: usize,
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
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
    async fn new(main: Arc<RwLock<MainServer>>) -> tokio::io::Result<Self> {
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

    async fn run(&mut self) -> tokio::io::Result<()> {
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
                self.handle_packet(&buffer[0..amount], peer_addr).await?;
            }

            if self.last_upkeep_time.elapsed() > UPKEEP_INTERVAL {
                self.upkeep().await?;
            }
        }
    }

    async fn upkeep(&mut self) -> tokio::io::Result<()> {
        for device in &mut self.devices {
            if device.last_packet_received_time.elapsed() > DEVICE_TIMEOUT {
                if !device.timed_out {
                    device.timed_out = true;
                    log::info!("Device at {} timed out", device.address);
                }
            } else {
                device.timed_out = false;
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
        let device_index = self.address_to_device_index.get(&peer_addr);
        let udp_device = device_index.and_then(|i| self.devices.get_mut(*i));

        match UdpPacket::parse(&mut byte_iter, udp_device) {
            Some(UdpPacket::Heartbeat) => {}
            Some(UdpPacket::Handshake(packet)) => {
                self.handle_handshake(packet, peer_addr).await?;
            }
            Some(UdpPacket::TrackerData(mut packet)) => {
                if let Some(udp_device) = udp_device {
                    let device = &mut self.main.write().await.devices[index];

                    while let Some(data) = packet.next(&mut byte_iter) {
                        let tracker = device.get_tracker_mut(data.tracker_index);
                        tracker.orientation = data.orientation;
                        tracker.acceleration = data.accleration;
                    }
                }
            }
            Some(UdpPacket::TrackerStatus(packet)) => {
                log::trace!("Got {:?}", packet);

                if let Some(device_index) = device_index {
                    self.handlet_tracker_status(packet, *device_index);
                }
            }
            None => {
                log::warn!("Received invalid packet");
            }
        }

        Ok(())
    }

    async fn handle_handshake(
        &mut self,
        packet: UdpPacketHandshake,
        peer_addr: SocketAddr,
    ) -> tokio::io::Result<()> {
        self.socket
            .send_to(UdpPacketHandshake::RESPONSE, peer_addr)
            .await?;

        if self.address_to_device_index.contains_key(&peer_addr) {
            log::info!("Reconnected from {peer_addr}");
            return Ok(());
        }

        // Check if the device already has connected with a mac address but different address
        if let Some(index) = self.mac_to_device_index.get(&packet.mac_string) {
            let device = &mut self.devices[*index];
            let index = device.index;
            let old_address = device.address;

            device.address = peer_addr;

            // Move over to the new address if the device has a new ip
            self.address_to_device_index.remove(&old_address);
            self.address_to_device_index.insert(peer_addr, index);
            log::info!("Reconnected from {peer_addr} from old: {old_address}");
            return Ok(());
        }

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
        let mut main = self.main.write().await;

        let global_index = match device.tracker_indexs.get(packet.tracker_index as usize) {
            Some(index) => *index,
            None => main.register_tracker(format!("{}/{}", device.mac, packet.tracker_index)),
        };

        main.set_tracker_status(global_index, packet.tracker_status);
    }
}

pub async fn start_server(main: Arc<RwLock<MainServer>>) -> tokio::io::Result<()> {
    UdpServer::new(main).await?.run().await?;
    Ok(())
}
