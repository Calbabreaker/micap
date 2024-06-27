use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::{TrackerConfig, TrackerData, TrackerStatus},
    udp_packet::{UdpPacket, UdpPacketHandshake, UdpPacketPingPong},
};

pub const UDP_PORT: u16 = 5828;
pub const MULTICAST_IP: Ipv4Addr = Ipv4Addr::new(239, 255, 0, 123);

const DEVICE_TIMEOUT: Duration = Duration::from_millis(4000);
const UPKEEP_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct UdpDevice {
    pub(super) index: usize,
    pub(super) last_packet_received_time: Instant,
    pub(super) last_packet_number: u32,
    /// Maps the udp device's tracker index to the tracker's global index
    tracker_indexs: Vec<usize>,
    timed_out: bool,
    mac: String,
    address: SocketAddr,
    current_ping_start_time: Option<Instant>,
    current_ping_id: u8,
}

impl UdpDevice {
    fn new(index: usize, address: SocketAddr, mac: String) -> Self {
        Self {
            tracker_indexs: Vec::default(),
            index,
            address,
            mac,
            last_packet_received_time: Instant::now(),
            last_packet_number: 0,
            timed_out: false,
            current_ping_id: 0,
            current_ping_start_time: None,
        }
    }

    fn set_global_tracker_index(&mut self, local_index: u8, global_index: usize) {
        if local_index as usize >= self.tracker_indexs.len() {
            self.tracker_indexs
                .resize_with(local_index as usize + 1, usize::default);
        }

        self.tracker_indexs[local_index as usize] = global_index;
    }

    fn get_global_tracker_index(&mut self, main: &mut MainServer, local_index: u8) -> usize {
        match self.tracker_indexs.get(local_index as usize) {
            Some(index) => *index,
            None => {
                // Register the tracker and add the index into the udp device array to know
                let id = format!("{}/{}", self.mac, local_index);
                let name = format!("UDP Tracker {}", self.address);
                let index = main.register_tracker(
                    id,
                    TrackerConfig {
                        name,
                        ..Default::default()
                    },
                );
                self.set_global_tracker_index(local_index, index);
                index
            }
        }
    }

    fn set_timed_out(&mut self, main: &mut MainServer, timed_out: bool) {
        if timed_out == self.timed_out {
            return;
        }

        self.timed_out = timed_out;

        for global_index in &self.tracker_indexs {
            let info = &main.trackers[*global_index].info;

            // Only allow changing status to TimedOut if tracker is Ok and vice-versa
            if timed_out && info.status == TrackerStatus::Ok {
                main.trackers[*global_index].info.status = TrackerStatus::TimedOut;
                main.tracker_info_updated(*global_index);
            } else if !timed_out && info.status == TrackerStatus::TimedOut {
                main.trackers[*global_index].info.status = TrackerStatus::Ok;
                main.tracker_info_updated(*global_index);
            };
        }
    }
}

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_to_device_index: HashMap<String, usize>,
    address_to_device_index: HashMap<SocketAddr, usize>,

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
            mac_to_device_index: Default::default(),
            address_to_device_index: Default::default(),
            last_upkeep_time: Instant::now(),
            socket,
        })
    }

    pub async fn tick(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
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

            // Ping has been acknowledge so start a new ping id
            if device.current_ping_start_time.is_none() {
                device.current_ping_start_time = Some(Instant::now());
                device.current_ping_id = device.current_ping_id.wrapping_add(1);
            }

            let ping_packet = UdpPacketPingPong::to_bytes(device.current_ping_id);
            self.socket.send_to(&ping_packet, device.address).await?;
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
            .address_to_device_index
            .get(&peer_addr)
            .and_then(|i| self.devices.get_mut(*i));

        match UdpPacket::parse(&mut bytes, device) {
            Ok(UdpPacket::PingPong((packet, device))) => {
                Self::handle_pong(main, packet, device);
            }
            Ok(UdpPacket::Handshake(packet)) => {
                self.socket
                    .send_to(&UdpPacketHandshake::to_bytes(), peer_addr)
                    .await?;
                if let Some(device) = self.handle_handshake(packet, peer_addr) {
                    device.last_packet_number = 0;
                }
            }
            Ok(UdpPacket::TrackerData((mut packet, device))) => {
                while let Ok(data) = packet.next() {
                    let global_index = device.get_global_tracker_index(main, data.tracker_index);
                    main.update_tracker_data(global_index, data.accleration, data.orientation);
                }
            }
            Ok(UdpPacket::TrackerStatus((packet, device))) => {
                log::trace!("Got status: {:?}", packet);

                self.socket.send_to(&packet.to_bytes(), peer_addr).await?;
                let global_index = device.get_global_tracker_index(main, packet.tracker_index);

                main.trackers[global_index].info.status = packet.tracker_status;
                main.trackers[global_index].data = TrackerData::default();
                main.tracker_info_updated(global_index);
            }
            Err(_) => (),
        }

        Ok(())
    }

    fn handle_handshake(
        &mut self,
        packet: UdpPacketHandshake,
        peer_addr: SocketAddr,
    ) -> Option<&mut UdpDevice> {
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
                return Some(device);
            } else if device.timed_out {
                log::info!("Reconnected from {peer_addr}");
                return Some(device);
            } else {
                log::warn!("Received handshake packet while already connected");
                return None;
            }
        }

        // Create a new udp device
        let index = self.devices.len();
        let device = UdpDevice::new(index, peer_addr, packet.mac_string.clone());
        self.mac_to_device_index.insert(packet.mac_string, index);
        self.address_to_device_index.insert(peer_addr, index);
        self.devices.push(device);
        log::info!("New device connected from {peer_addr}");
        self.devices.get_mut(index)
    }

    fn handle_pong(main: &mut MainServer, packet: UdpPacketPingPong, device: &mut UdpDevice) {
        if packet.id != device.current_ping_id {
            return;
        }

        if let Some(start_time) = device.current_ping_start_time {
            for global_index in &device.tracker_indexs {
                let latency = start_time.elapsed() / 2;
                main.trackers[*global_index].info.latency_ms = Some(latency.as_millis() as u32);
                main.tracker_info_updated(*global_index);
            }

            device.current_ping_start_time = None;
        }
    }
}
