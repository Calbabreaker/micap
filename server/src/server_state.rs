use std::net::{IpAddr, SocketAddr};

pub struct Device {
    pub address: SocketAddr,
}

pub struct ServerState {
    devices: Vec<Device>,
    local_ip: Option<IpAddr>,
}

impl ServerState {
    pub async fn new() -> Self {
        Self {
            devices: Vec::new(),
            local_ip: get_local_ip().await,
        }
    }
}

async fn get_local_ip() -> Option<IpAddr> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0").await.ok()?;
    socket.connect(("1.1.1.1", 80)).await.ok()?;
    let local_ip = socket.local_addr().ok()?.ip();
    log::info!("Found local ip: {local_ip}");
    Some(local_ip)
}
