use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::UdpSocket;

const VMC_PORT: u16 = 39539;

pub struct VmcServer {
    socket: UdpSocket,
}

impl VmcServer {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        socket.connect(("127.0.0.1", VMC_PORT)).await?;
        Ok(Self { socket })
    }

    pub async fn send(&mut self) -> anyhow::Result<()> {
        let msg_buf = rosc::encoder::encode(&rosc::OscPacket::Message(rosc::OscMessage {
            addr: "/VMC/Ext/OK".to_string(),
            args: vec![rosc::OscType::Int(1)],
        }))?;
        self.socket.send(&msg_buf).await?;

        Ok(())
    }
}
