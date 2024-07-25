use std::{net::Ipv4Addr, time::Duration};

use mycap_server::udp::{
    packet::{PACKET_HANDSHAKE, PACKET_TRACKER_DATA, PACKET_TRACKER_STATUS},
    server::UDP_PORT,
};
use rand::Rng;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let count: u32 = std::env::args()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);

    println!("Spawning {count} tasks");

    let mut handles = Vec::new();
    for i in 0..count {
        handles.push(tokio::spawn(task(i as u8)));
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

async fn task(id: u8) -> anyhow::Result<()> {
    let socket = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    socket.connect((Ipv4Addr::LOCALHOST, UDP_PORT)).await?;

    #[rustfmt::skip]
    socket.send(&[PACKET_HANDSHAKE, b'M', b'C', b'D', b'E', b'V', id, 0xff, 0xff, 0xff, 0xff, 0xff]).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    #[rustfmt::skip]
    socket.send(&[PACKET_TRACKER_STATUS, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut count = 1_u32;
    let mut buffer = Vec::new();
    loop {
        buffer.push(PACKET_TRACKER_DATA);
        buffer.extend(count.to_le_bytes());
        buffer.push(0x00);

        for _ in 0..7 {
            let float: f32 = rand::thread_rng().gen_range((0.)..5.);
            buffer.extend(float.to_le_bytes());
        }

        socket.send(&buffer).await?;
        count += 1;
        buffer.clear();
        tokio::time::sleep(Duration::from_millis(6)).await;
    }
}
