use std::{borrow::BorrowMut, collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

use crate::{
    server_state::Device,
    udp_packet::{UdpPacket, UdpPacketHandshake, PACKET_HEARTBEAT},
    ServerState,
};

pub const UDP_PORT: u16 = 5828;
const DEVICE_TIMEOUT: Duration = Duration::from_millis(5000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);
const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

pub struct UdpDevice {
    index: usize,
    last_packet_received_time: Instant,
    timed_out: bool,
    address: SocketAddr,
}

impl UdpDevice {
    fn new(index: usize, address: SocketAddr) -> Self {
        Self {
            index,
            address,
            last_packet_received_time: Instant::now(),
            timed_out: false,
        }
    }
}

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_to_device_index: HashMap<String, usize>,
    address_to_device_index: HashMap<SocketAddr, usize>,

    socket: UdpSocket,
    buffer: [u8; 256],
    last_upkeep_time: Instant,
    state: Arc<RwLock<ServerState>>,
}

impl UdpServer {
    async fn new(state: Arc<RwLock<ServerState>>) -> tokio::io::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)).await?;
        log::info!("Bound UDP on {}", socket.local_addr()?);

        Ok(Self {
            buffer: [0; 256],
            devices: Default::default(),
            mac_to_device_index: Default::default(),
            address_to_device_index: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
            state,
        })
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        loop {
            // Have receiving data timeout so that the upkeep check can happen continously
            if let Ok(Ok((amount, src))) =
                tokio::time::timeout(SOCKET_TIMEOUT, self.socket.recv_from(&mut self.buffer)).await
            {
                log::trace!(
                    "Received {amount} bytes from {src} ({:#02x})",
                    self.buffer[0]
                );
                self.handle_packet(src).await?;
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

    async fn handle_packet(&mut self, src: SocketAddr) -> tokio::io::Result<()> {
        if let Some(index) = self.address_to_device_index.get(&src) {
            // Reset last packet time
            let device = &mut self.devices[*index];
            device.last_packet_received_time = Instant::now();
        }

        let mut bytes = self.buffer.iter();
        match UdpPacket::from_bytes(&mut bytes) {
            Some(UdpPacket::Handshake(handshake)) => {
                self.handle_handshake(handshake, src).await?;
            }
            Some(UdpPacket::TrackerData(mut packet)) => {
                if let Some(index) = self.address_to_device_index.get(&src) {
                    let device = &mut self.state.write().await.devices[*index];

                    while let Some(data) = packet.next(&mut bytes) {
                        let tracker = device.get_tracker_mut(data.tracker_id);
                        tracker.orientation = data.orientation;
                        tracker.acceleration = data.accleration;
                    }
                }
            }
            Some(UdpPacket::Heartbeat) => {}
            Some(UdpPacket::TrackerStatus(packet)) => {
                log::trace!("Got {:?}", packet);
                if let Some(index) = self.address_to_device_index.get(&src) {
                    let device = &mut self.state.write().await.devices[*index];
                    device.get_tracker_mut(packet.tracker_id).status = packet.tracker_status;

                    // Send back the tracker status so the device knows the server knows it
                    self.socket.send_to(&self.buffer[0..3], src).await?;
                }
            }
            _ => {
                log::warn!(
                    "Received invalid packet: {}",
                    String::from_utf8_lossy(&self.buffer)
                )
            }
        }

        Ok(())
    }

    async fn handle_handshake(
        &mut self,
        packet: UdpPacketHandshake,
        src: SocketAddr,
    ) -> tokio::io::Result<()> {
        self.socket
            .send_to(UdpPacketHandshake::RESPONSE, src)
            .await?;

        // First check if the device allready has connected with a mac address
        if let Some(index) = self.mac_to_device_index.get(&packet.mac_string) {
            let device = &mut self.devices[*index];
            let id = device.index;
            let old_address = device.address;

            device.address = src;

            // Move over to the new address if the device has a new ip
            self.address_to_device_index.remove(&old_address);
            self.address_to_device_index.insert(src, id);
            log::info!("Reconnected from {src} from old: {old_address}");
            return Ok(());
        }

        if self.address_to_device_index.get(&src).is_some() {
            log::info!("Reconnected from {src}");
            return Ok(());
        }

        let id = self.devices.len();
        self.mac_to_device_index.insert(packet.mac_string, id);
        self.address_to_device_index.insert(src, id);
        self.devices.push(UdpDevice::new(id, src));
        self.state.write().await.devices.push(Device::default());
        log::info!("New device connected from {src}");
        Ok(())
    }
}

pub async fn start_server(state: Arc<RwLock<ServerState>>) -> tokio::io::Result<()> {
    UdpServer::new(state).await?.run().await?;
    Ok(())
}
