use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::{
    server_state::Device,
    udp_packet::{UdpPacket, UdpPacketHandshake, PACKET_HEARTBEAT},
    ServerState,
};

pub const UDP_PORT: u16 = 5828;
const DEVICE_TIMEOUT: Duration = Duration::from_millis(5000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);
const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

pub struct UdpDevice {
    index: usize,
    pub last_packet_received_time: Instant,
    pub last_packet_number: u32,
    timed_out: bool,
    address: SocketAddr,
}

impl UdpDevice {
    fn new(index: usize, address: SocketAddr) -> Self {
        Self {
            index,
            address,
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
    state: Arc<RwLock<ServerState>>,
}

impl UdpServer {
    async fn new(state: Arc<RwLock<ServerState>>) -> tokio::io::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)).await?;
        log::info!("Started UDP server on {}", socket.local_addr()?);

        Ok(Self {
            devices: Default::default(),
            mac_to_device_index: Default::default(),
            address_to_device_index: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
            state,
        })
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        self.socket
            .join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)?;

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
        let device_index = self.address_to_device_index.get(&peer_addr).copied();
        let udp_device = device_index.and_then(|i| self.devices.get_mut(i));

        let mut byte_iter = bytes.iter();
        match UdpPacket::parse(&mut byte_iter, udp_device) {
            Some(UdpPacket::Handshake(packet)) => {
                self.handle_handshake(packet, peer_addr).await?;
            }
            Some(UdpPacket::TrackerData(mut packet)) => {
                if let Some(index) = device_index {
                    let device = &mut self.state.write().await.devices[index];

                    while let Some(data) = packet.next(&mut byte_iter) {
                        let tracker = device.get_tracker_mut(data.tracker_index);
                        tracker.orientation = data.orientation;
                        tracker.acceleration = data.accleration;
                    }
                }
            }
            Some(UdpPacket::Heartbeat) => {}
            Some(UdpPacket::TrackerStatus(packet)) => {
                log::trace!("Got {:?}", packet);
                if let Some(index) = device_index {
                    let device = &mut self.state.write().await.devices[index];
                    device.get_tracker_mut(packet.tracker_index).status = packet.tracker_status;

                    // Send back the tracker status so the device knows the server knows it
                    self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                }
            }
            _ => {
                log::warn!("Received invalid packet")
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

        // First check if the device allready has connected with a mac address
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

        if self.address_to_device_index.contains_key(&peer_addr) {
            log::info!("Reconnected from {peer_addr}");
            return Ok(());
        }

        let index = self.devices.len();
        self.mac_to_device_index.insert(packet.mac_string, index);
        self.address_to_device_index.insert(peer_addr, index);
        self.devices.push(UdpDevice::new(index, peer_addr));
        self.state.write().await.devices.push(Device::default());
        log::info!("New device connected from {peer_addr}");
        Ok(())
    }
}

pub async fn start_server(state: Arc<RwLock<ServerState>>) -> tokio::io::Result<()> {
    UdpServer::new(state).await?.run().await?;
    Ok(())
}
