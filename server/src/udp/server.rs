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
    // Maps a network address to a udp device
    devices_address_map: HashMap<SocketAddr, UdpDevice>,
    mac_to_address_map: HashMap<String, SocketAddr>,

    socket: tokio::net::UdpSocket,
    last_upkeep_time: Instant,
}

impl UdpServer {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, UDP_PORT)).await?;
        socket.join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)?;
        log::info!("Started UDP server on {}", socket.local_addr()?);

        Ok(Self {
            devices_address_map: HashMap::new(),
            mac_to_address_map: HashMap::new(),
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
                    let bytes = &buffer[0..amount];
                    if let Err(err) = self.handle_packet(bytes, peer_addr, main).await {
                        log::warn!("Received invalid packet 0x{:02x}: {err:?}", bytes[0]);
                    }
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
        let mut to_remove = None;

        for device in self.devices_address_map.values_mut() {
            // When the user has removed every tracker from the device prevent it from connecting anymore
            let still_exists = device
                .tracker_ids
                .iter()
                .any(|id| main.trackers.contains_key(id));
            if !device.tracker_ids.is_empty() && !still_exists {
                to_remove = Some((device.mac.clone(), device.address))
            }

            if device.last_packet_received_time.elapsed() > DEVICE_TIMEOUT {
                device.set_timed_out(main, true);
            } else {
                device.set_timed_out(main, false);
            }

            device.check_send_new_ping(&self.socket).await?;
        }

        if let Some((mac, address)) = to_remove {
            self.devices_address_map.remove(&address);
            self.mac_to_address_map.remove(&mac);
            main.address_blacklist.insert(address);
        }

        self.last_upkeep_time = Instant::now();
        Ok(())
    }

    async fn handle_packet(
        &mut self,
        mut bytes: &[u8],
        peer_addr: SocketAddr,
        main: &mut MainServer,
    ) -> anyhow::Result<()> {
        let mut device = self
            .devices_address_map
            .get_mut(&peer_addr)
            .ok_or_else(|| anyhow::anyhow!("No device with address: {peer_addr}"));

        match UdpPacket::parse(&mut bytes, &mut device)? {
            UdpPacket::PingPong(packet) => {
                device?.handle_pong(main, packet);
            }
            UdpPacket::Handshake(packet) => {
                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                self.handle_handshake(packet, peer_addr);
            }
            UdpPacket::TrackerData(mut packet) => {
                let device = device?;
                while let Ok(data) = packet.next_data() {
                    device.update_tracker_data(main, data);
                }
            }
            UdpPacket::TrackerStatus(packet) => {
                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                device?.update_tracker_status(main, packet);
            }
            UdpPacket::BatteryLevel(packet) => {
                device?.update_battery_level(main, packet);
            }
        }

        Ok(())
    }

    fn handle_handshake(&mut self, packet: UdpPacketHandshake, peer_addr: SocketAddr) {
        // Check if the device already has connected with a mac address
        if let Some(address) = self.mac_to_address_map.get(&packet.mac_address) {
            let device = self.devices_address_map.get_mut(address).unwrap();
            device.last_packet_number = 0;

            // Move over to the new address if the device has a new ip
            if *address != peer_addr {
                log::info!("Reconnected from {peer_addr} from old: {address}");
                device.address = peer_addr;

                // Swap in the map
                let device = self.devices_address_map.remove(address).unwrap();
                self.devices_address_map.insert(peer_addr, device);
                self.mac_to_address_map
                    .insert(packet.mac_address, peer_addr);
            } else if device.timed_out {
                log::info!("Reconnected from {peer_addr}");
            } else {
                log::warn!("Received handshake packet while already connected");
            }

            return;
        }

        // Create a new udp device
        let device = UdpDevice::new(peer_addr, packet.mac_address.clone());
        self.mac_to_address_map
            .insert(packet.mac_address, peer_addr);
        self.devices_address_map.insert(peer_addr, device);
        log::info!("New device connected from {peer_addr}");
    }
}
