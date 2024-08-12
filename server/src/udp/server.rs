use futures_util::FutureExt;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};

use crate::{
    main_server::MainServer,
    udp::{
        device::UdpDevice,
        packet::{UdpPacket, UdpPacketHandshake},
    },
};

pub const UDP_PORT: u16 = 5828;
pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

const DEVICE_TIMEOUT: Duration = Duration::from_millis(3000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_index_map: HashMap<String, usize>,
    address_index_map: HashMap<SocketAddr, usize>,

    socket: tokio::net::UdpSocket,
    last_upkeep_time: Instant,
}

impl UdpServer {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, UDP_PORT)).await?;
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
            match self.socket.recv_from(&mut buffer).now_or_never() {
                Some(Ok((amount, peer_addr))) => {
                    if main.address_blacklist.contains(&peer_addr) {
                        continue;
                    }

                    log::trace!(
                        "Received {amount} bytes from {peer_addr} (0x{:02x})",
                        buffer[0]
                    );

                    // Only pass through the amount received
                    self.handle_packet(&buffer[0..amount], peer_addr, main)
                        .await?;
                }
                // No more packets
                None => {
                    return Ok(());
                }
                Some(Err(e)) => Err(e)?,
            }
        }
    }

    async fn upkeep(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        for device in self.devices.iter_mut() {
            if main.address_blacklist.contains(&device.address) {
                continue;
            }

            if !device.tracker_indexs.is_empty() {
                // When the user has removed every tracker from the device prevent it from connecting anymore
                let all_removed = device.tracker_indexs.iter().all(|index| {
                    main.trackers[*index].info.removed //
                });
                if all_removed {
                    main.address_blacklist.insert(device.address);
                }
            }

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
                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
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
            Err(_) => log::warn!("Received invalid packet 0x{:02x}", bytes[0]),
        }

        Ok(())
    }

    fn handle_handshake(&mut self, packet: UdpPacketHandshake, peer_addr: SocketAddr) {
        // Check if the device already has connected with a mac address
        if let Some(index) = self.mac_index_map.get(&packet.mac_address) {
            let device = &mut self.devices[*index];
            // Move over to the new address if the device has a new ip
            if device.address != peer_addr {
                log::info!("Reconnected from {peer_addr} from old: {}", device.address);
                self.address_index_map.remove(&device.address);
                self.address_index_map.insert(peer_addr, *index);
                device.address = peer_addr;
            } else if device.timed_out {
                log::info!("Reconnected from {peer_addr}");
            } else {
                log::warn!("Received handshake packet while already connected");
            }

            device.last_packet_number = 0;
            return;
        }

        // Create a new udp device
        let index = self.devices.len();
        let device = UdpDevice::new(peer_addr, packet.mac_address.clone());
        self.mac_index_map.insert(packet.mac_address, index);
        self.address_index_map.insert(peer_addr, index);
        self.devices.push(device);
        log::info!("New device connected from {peer_addr}");
    }
}
