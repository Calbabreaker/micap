use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::UdpSocket;
use tokio::time::{Duration, Instant};

use crate::udp_packet::{UdpPacket, UdpPacketHandshake, PACKET_HEARTBEAT};

pub const DEVICE_TIMEOUT: Duration = Duration::from_millis(5000);
pub const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);
pub const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 1, 1, 1);

#[derive(Default)]
pub struct Tracker {
    id: u32,
}

pub struct UdpDevice {
    index: usize,
    last_packet_received_time: Instant,
    trackers: HashMap<u32, Tracker>,
    timed_out: bool,
    address: SocketAddr,
}

impl UdpDevice {
    fn new(index: usize, address: SocketAddr) -> Self {
        Self {
            index,
            address,
            last_packet_received_time: Instant::now(),
            trackers: Default::default(),
            timed_out: false,
        }
    }
}

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_to_device_index: HashMap<String, usize>,
    address_to_device_index: HashMap<SocketAddr, usize>,

    socket: UdpSocket,
    buffer: [u8; 64],
    last_upkeep_time: Instant,
}

impl UdpServer {
    pub const UDP_PORT: u16 = 5828;

    async fn new() -> tokio::io::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", Self::UDP_PORT)).await?;
        log::info!("Bound UDP on {}", socket.local_addr()?);

        Ok(Self {
            buffer: [0; 64],
            devices: Default::default(),
            mac_to_device_index: Default::default(),
            address_to_device_index: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
        })
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        self.socket
            .join_multicast_v4(MULTICAST_ADDR, Ipv4Addr::UNSPECIFIED)?;
        assert!(MULTICAST_ADDR.is_multicast());

        loop {
            // Have receiving data timeout so that the upkeep check can happen continously
            if let Ok(Ok((amount, src))) =
                tokio::time::timeout(SOCKET_TIMEOUT, self.socket.recv_from(&mut self.buffer)).await
            {
                log::info!("Received {amount} bytes from {src}");
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
        if let Some(device) = self.device_by_addr(&src) {
            // Reset last packet time
            device.last_packet_received_time = Instant::now();
        }

        match UdpPacket::from_bytes(&self.buffer) {
            Some(UdpPacket::Handshake(handshake)) => {
                self.handle_handshake(handshake, src).await?;
            }
            Some(UdpPacket::Acceleration(accel)) => {
                log::info!("ACCEL: {:?}", &accel.acceleration);
            }
            Some(UdpPacket::Hearbeat) => {}
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
        handshake: UdpPacketHandshake,
        src: SocketAddr,
    ) -> tokio::io::Result<()> {
        self.socket
            .send_to(UdpPacketHandshake::RESPONSE, src)
            .await?;

        // First check if the device allready has connected with a mac address
        if let Some(device) = self.device_by_mac(&handshake.mac_string) {
            let index = device.index;
            let old_address = device.address;

            device.address = src;

            // Move over to the new address if the device has a new ip
            self.address_to_device_index.remove(&old_address);
            self.address_to_device_index.insert(src, index);
            log::info!("Reconnected from {src} from old: {old_address}");
            return Ok(());
        }

        if self.device_by_addr(&src).is_some() {
            log::info!("Reconnected from {src}");
            return Ok(());
        }

        let index = self.devices.len();
        self.mac_to_device_index.insert(handshake.mac_string, index);
        self.devices.push(UdpDevice::new(index, src));
        log::info!("New device connected from {src}");
        Ok(())
    }

    fn device_by_mac(&mut self, mac_string: &String) -> Option<&mut UdpDevice> {
        let index = self.mac_to_device_index.get(mac_string)?;
        Some(&mut self.devices[*index])
    }

    fn device_by_addr(&mut self, addr: &SocketAddr) -> Option<&mut UdpDevice> {
        let index = self.address_to_device_index.get(addr)?;
        Some(&mut self.devices[*index])
    }
}

pub async fn start_server() -> tokio::io::Result<()> {
    UdpServer::new().await?.run().await?;
    Ok(())
}
