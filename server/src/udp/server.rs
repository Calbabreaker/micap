use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    udp::{
        device::UdpDevice,
        packet::{UdpPacket, UdpPacketHandshake},
    },
};

pub const UDP_PORT: u16 = 5828;
pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

const DEVICE_TIMEOUT: Duration = Duration::from_millis(4000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_index_map: HashMap<String, usize>,
    address_index_map: HashMap<SocketAddr, usize>,

    socket: UdpSocket,
    last_upkeep_time: Instant,
}

impl UdpServer {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)).await?;
        socket.join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)?;
        log::info!("Started UDP server on {}", socket.local_addr()?);

        Ok(Self {
            devices: Default::default(),
            mac_index_map: Default::default(),
            address_index_map: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        if self.last_upkeep_time.elapsed() > UPKEEP_INTERVAL {
            self.upkeep(main).await?;
        }

        let mut buffer = [0_u8; 256];
        loop {
            // Try and get all the packets that were received
            match self.socket.try_recv_from(&mut buffer) {
                Ok((amount, peer_addr)) => {
                    log::trace!(
                        "Received {amount} bytes from {peer_addr} (0x{:02x})",
                        buffer[0]
                    );

                    // Only pass through the amount received
                    self.handle_packet(&buffer[0..amount], peer_addr, main)
                        .await?;
                }
                // No more packets
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(());
                }
                Err(e) => Err(e)?,
            }
        }
    }

    async fn upkeep(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        for device in &mut self.devices {
            if device.last_packet_received_time.elapsed() > DEVICE_TIMEOUT {
                device.set_timed_out(main, true);
            } else {
                device.set_timed_out(main, false);
            }

            device.check_send_new_ping(&self.socket).await?;
        }

        self.last_upkeep_time = Instant::now();
        Ok(())
    }

    async fn handle_packet(
        &mut self,
        mut bytes: &[u8],
        peer_addr: SocketAddr,
        main: &mut MainServer,
    ) -> tokio::io::Result<()> {
        let device = self
            .address_index_map
            .get(&peer_addr)
            .and_then(|i| self.devices.get_mut(*i));

        match UdpPacket::parse(&mut bytes, device) {
            Ok(UdpPacket::PingPong((packet, device))) => {
                device.handle_pong(main, packet);
            }
            Ok(UdpPacket::Handshake(packet)) => {
                self.socket
                    .send_to(&UdpPacketHandshake::to_bytes(), peer_addr)
                    .await?;
                self.handle_handshake(packet, peer_addr);
            }
            Ok(UdpPacket::TrackerData((mut packet, device))) => {
                while let Ok(data) = packet.next_data() {
                    device.update_tracker_data(main, data);
                }
            }
            Ok(UdpPacket::TrackerStatus((packet, device))) => {
                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                device.update_tracker_status(main, packet);
            }
            Ok(UdpPacket::BatteryLevel((packet, device))) => {
                device.update_battery_level(main, packet);
            }
            Err(_) => (),
        }

        Ok(())
    }

    fn handle_handshake(&mut self, packet: UdpPacketHandshake, peer_addr: SocketAddr) {
        // Check if the device already has connected with a mac address
        if let Some(index) = self.mac_index_map.get(&packet.mac_string) {
            let device = &mut self.devices[*index];
            device.check_address_handshake(peer_addr, &mut self.address_index_map);
            return;
        }

        // Create a new udp device
        let index = self.devices.len();
        let device = UdpDevice::new(index, peer_addr, packet.mac_string.clone());
        self.mac_index_map.insert(packet.mac_string, index);
        self.address_index_map.insert(peer_addr, index);
        self.devices.push(device);
        log::info!("New device connected from {peer_addr}");
    }
}
