use std::net::Ipv4Addr;

use tokio::net::UdpSocket;

use crate::{
    tracker::TrackerStatus,
    udp::{
        packet::{
            UdpTrackerData, PACKET_BATTERY_LEVEL, PACKET_HANDSHAKE, PACKET_PING_PONG,
            PACKET_TRACKER_DATA, PACKET_TRACKER_STATUS,
        },
        server::UDP_PORT,
    },
};

/// This client is mainly for testing purposes and will not be used in the app
pub struct UdpTrackerClient {
    pub socket: UdpSocket,
    packet_number: u32,
    buffer: Vec<u8>,
}

impl UdpTrackerClient {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        socket.connect((Ipv4Addr::LOCALHOST, UDP_PORT)).await?;
        Ok(Self {
            socket,
            packet_number: 0,
            buffer: Vec::new(),
        })
    }

    pub async fn send_handshake(&mut self, mac: [u8; 6]) -> anyhow::Result<()> {
        self.begin_packet(PACKET_HANDSHAKE);
        self.buffer.extend(b"MCDEV");
        self.buffer.extend(mac);
        self.send_buffer().await
    }

    pub async fn send_battery_level(&mut self, level: f32) -> anyhow::Result<()> {
        self.begin_packet(PACKET_BATTERY_LEVEL);
        self.buffer.extend(level.to_le_bytes());
        self.send_buffer().await
    }

    pub async fn send_ping(&mut self, id: u8) -> anyhow::Result<()> {
        self.begin_packet(PACKET_PING_PONG);
        self.buffer.push(id);
        self.send_buffer().await
    }

    pub async fn send_tracker_status(
        &mut self,
        index: u8,
        status: TrackerStatus,
    ) -> anyhow::Result<()> {
        self.begin_packet(PACKET_TRACKER_STATUS);
        self.buffer.push(index);
        self.buffer.push(status as u8);
        self.send_buffer().await
    }

    pub async fn send_tracker_data(&mut self, datas: &[&UdpTrackerData]) -> anyhow::Result<()> {
        self.begin_packet(PACKET_TRACKER_DATA);

        for data in datas {
            self.buffer.push(data.tracker_index);
            let orien = data.orientation.to_array();
            let accel = data.acceleration.to_array();

            self.buffer
                .extend(orien.iter().flat_map(|x| x.to_le_bytes()));
            self.buffer
                .extend(accel.iter().flat_map(|x| x.to_le_bytes()));
        }

        self.buffer.push(0xff);
        self.send_buffer().await
    }

    fn begin_packet(&mut self, id: u8) {
        self.buffer.push(id);
        self.buffer.extend(self.packet_number.to_le_bytes());
    }

    async fn send_buffer(&mut self) -> anyhow::Result<()> {
        self.socket.send(&self.buffer).await?;
        self.buffer.clear();
        Ok(())
    }
}
