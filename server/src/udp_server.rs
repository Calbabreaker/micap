use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use futures_util::{lock::Mutex, SinkExt};
use warp::filters::ws::Message;

use crate::{
    udp_packet::{UdpPacket, UdpPacketHandshake, PACKET_HANDSHAKE},
    ServerState,
};

#[derive(Default)]
pub struct Tracker {
    id: u32,
}

pub struct UdpDevice {
    last_packet_time: Instant,
    trackers: HashMap<u32, Tracker>,
    timed_out: bool,
}

impl Default for UdpDevice {
    fn default() -> Self {
        Self {
            last_packet_time: Instant::now(),
            trackers: Default::default(),
            timed_out: false,
        }
    }
}

pub struct UdpServer {
    devices: Vec<UdpDevice>,
    mac_to_device_index: HashMap<String, usize>,
    buffer: [u8; 64],
    socket: tokio::net::UdpSocket,
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
            socket,
        })
    }

    async fn run(&mut self) -> tokio::io::Result<()> {
        loop {
            let (amount, src) = self.socket.recv_from(&mut self.buffer).await?;

            log::info!("Received {amount} bytes from {src}");

            match UdpPacket::from_bytes(&self.buffer) {
                Some(UdpPacket::Handshake(handshake)) => {
                    self.handle_handshake(handshake).await?;
                }
                Some(UdpPacket::Acceleration(accel)) => {
                    log::info!("ACCEL: {:?}", &accel.acceleration);
                }
                _ => (),
            }
        }
    }

    async fn handle_handshake(&mut self, handshake: UdpPacketHandshake) -> tokio::io::Result<()> {
        self.mac_to_device_index
            .insert(handshake.mac_string, self.devices.len());
        self.devices.push(UdpDevice::default());

        let src = self.socket.peer_addr()?;
        log::info!("Received handshake from: {}", src);
        self.socket
            .send_to(UdpPacketHandshake::RESPONSE, src)
            .await?;
        Ok(())
    }
}

pub async fn start_server() -> tokio::io::Result<()> {
    UdpServer::new().await?.run().await?;
    Ok(())
}
