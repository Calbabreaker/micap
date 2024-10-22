use futures_util::FutureExt;
use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
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

const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);

pub struct UdpServer {
    // Maps a network address to a udp device
    devices_map: HashMap<SocketAddr, UdpDevice>,
    mac_to_address_map: HashMap<Arc<str>, SocketAddr>,
    socket: tokio::net::UdpSocket,
    /// Set of address that should not be allowed to connect
    /// This is to allow for servers to ignore ignored trackers that are trying to connect
    address_blacklist: HashSet<SocketAddr>,
    last_upkeep_time: Instant,
}

impl UdpServer {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, UDP_PORT)).await?;
        socket.join_multicast_v4(MULTICAST_IP, Ipv4Addr::UNSPECIFIED)?;
        log::info!("Started UDP server on {}", socket.local_addr()?);

        Ok(Self {
            devices_map: HashMap::new(),
            mac_to_address_map: HashMap::new(),
            address_blacklist: HashSet::new(),
            last_upkeep_time: Instant::now(),
            socket,
        })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        if self.last_upkeep_time.elapsed() > UPKEEP_INTERVAL {
            self.upkeep().await?;
            self.last_upkeep_time = Instant::now();
        }

        let mut buffer = [0; 256];
        loop {
            // Try and get all the packets that were received
            match self.socket.recv_from(&mut buffer).now_or_never() {
                Some(Ok((amount, peer_addr))) => {
                    if self.address_blacklist.contains(&peer_addr) {
                        continue;
                    }

                    log::trace!(
                        "Received {amount} bytes from {peer_addr} (0x{:02x})",
                        buffer[0]
                    );

                    // Only pass through the amount received
                    let bytes = &buffer[0..amount];
                    if let Err(err) = self.handle_packet(bytes, peer_addr, main).await {
                        log::trace!("Received invalid packet 0x{:02x}: {err:?}", bytes[0]);
                    }
                }
                // No new data currently
                None => return Ok(()),
                Some(Err(e)) => return Err(e)?,
            }
        }
    }

    pub(crate) async fn upkeep(&mut self) -> anyhow::Result<()> {
        let mut to_remove = None;

        for device in self.devices_map.values_mut() {
            device.update_timed_out(device.is_timed_out());

            let bytes = device.check_get_ping_packet().to_response();
            self.socket.send_to(&bytes, device.address).await?;

            // When the user has removed every tracker from the device prevent it from connecting anymore
            if device.all_trackers_removed() {
                self.mac_to_address_map.remove(&device.mac);
                self.address_blacklist.insert(device.address);
                log::info!("Added {} to blacklist", device.address);
                to_remove = Some(device.address);
            }
        }

        if let Some(address) = to_remove {
            self.devices_map.remove(&address);
        }

        Ok(())
    }

    async fn handle_packet(
        &mut self,
        mut bytes: &[u8],
        peer_addr: SocketAddr,
        main: &mut MainServer,
    ) -> anyhow::Result<()> {
        let mut device = self
            .devices_map
            .get_mut(&peer_addr)
            .ok_or_else(|| anyhow::anyhow!("No device with address: {peer_addr}"));

        let (packet, packet_number) = UdpPacket::parse(&mut bytes)?;

        if let Ok(device) = device.as_mut() {
            device.last_packet_received_time = Instant::now();

            // Discard the packet if not the latest
            if !device.check_latest_packet_number(packet_number) {
                anyhow::bail!("Out of order #{packet_number}");
            }
        }

        match packet {
            UdpPacket::Handshake(packet) => {
                let bytes = UdpPacketHandshake::SERVER_RESPONSE;
                self.socket.send_to(bytes, peer_addr).await?;
                self.handle_handshake(packet, peer_addr);
            }
            UdpPacket::PingPong(packet) => {
                device?.handle_pong(packet);
            }
            UdpPacket::TrackerData(mut packet) => {
                let device = device?;
                while let Some(data) = packet.next_data()? {
                    device.update_tracker_data(data);
                }
            }
            UdpPacket::TrackerStatus(packet) => {
                let bytes = packet.to_response();
                self.socket.send_to(&bytes, peer_addr).await?;
                device?.update_tracker_status(main, packet);
            }
            UdpPacket::BatteryLevel(packet) => {
                device?.update_battery_level(packet);
            }
        }

        Ok(())
    }

    fn handle_handshake(&mut self, packet: UdpPacketHandshake, peer_addr: SocketAddr) {
        // Check if the device already has connected with a mac address
        if let Some(address) = self.mac_to_address_map.get(&packet.mac_address) {
            let device = self.devices_map.get_mut(address).unwrap();
            device.last_packet_number = 0;

            // Move over to the new address if the device has a new ip
            if *address != peer_addr {
                log::info!("Reconnected from {peer_addr} from old: {address}");
                device.address = peer_addr;

                // Swap in the map
                let device = self.devices_map.remove(address).unwrap();
                self.devices_map.insert(peer_addr, device);
                self.mac_to_address_map
                    .insert(packet.mac_address, peer_addr);
            } else if device.is_timed_out() {
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
        self.devices_map.insert(peer_addr, device);
        log::info!("New udp device connected from {peer_addr}");
    }
}
