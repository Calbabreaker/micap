use std::{net::Ipv4Addr, time::SystemTime};
use tokio::net::UdpSocket;

pub mod vmc_connector;
pub mod vrchat_connector;

pub struct OscConnector {
    socket: UdpSocket,
}

impl OscConnector {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        Ok(Self { socket })
    }

    pub async fn send_bundle(
        &mut self,
        messages: impl Iterator<Item = rosc::OscPacket>,
    ) -> anyhow::Result<()> {
        let msg_buf = rosc::encoder::encode(&rosc::OscPacket::Bundle(rosc::OscBundle {
            timetag: SystemTime::now().try_into()?,
            content: messages.collect(),
        }))?;
        self.socket.send(&msg_buf).await?;
        Ok(())
    }

    pub async fn connect(&mut self, port: u16) -> anyhow::Result<()> {
        self.socket.connect((Ipv4Addr::LOCALHOST, port)).await?;
        // Test send (2 sees to make it consistent)
        self.socket.send(&[]).await?;
        self.socket.send(&[]).await?;
        log::info!("Sending OSC packets to {}", self.socket.peer_addr()?);
        Ok(())
    }
}
