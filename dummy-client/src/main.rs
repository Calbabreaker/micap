use std::{net::Ipv4Addr, time::Duration};

use micap_server::udp::{
    packet::{PACKET_HANDSHAKE, PACKET_TRACKER_DATA, PACKET_TRACKER_STATUS},
    server::UDP_PORT,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let count: u8 = std::env::args()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(1);

    println!("Spawning {count} tasks");

    let mut handles = Vec::new();

    for i in 0..count {
        handles.push(tokio::spawn(async move {
            loop {
                if let Err(err) = task(i).await {
                    eprintln!("{err}, reconnecting");
                }
            }
        }));
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

async fn task(id: u8) -> anyhow::Result<()> {
    let socket = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    socket.connect((Ipv4Addr::LOCALHOST, UDP_PORT)).await?;

    let mut buffer = Vec::new();
    buffer.push(PACKET_HANDSHAKE);
    buffer.extend(0_u32.to_le_bytes());
    buffer.extend(b"MCDEV");
    buffer.extend(&[id, 0, 0, 0, 0, 0]);
    socket.send(&buffer).await?;
    buffer.clear();
    tokio::time::sleep(Duration::from_secs(1)).await;

    buffer.push(PACKET_TRACKER_STATUS);
    buffer.extend(1_u32.to_le_bytes());
    buffer.extend(&[0, 0]);
    socket.send(&buffer).await?;
    buffer.clear();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut count = 2_u32;
    loop {
        buffer.push(PACKET_TRACKER_DATA);
        buffer.extend(count.to_le_bytes());
        buffer.push(0x00);

        let quat = glam::Quat::from_axis_angle(
            glam::Vec3::new(1., 0., 0.).normalize(),
            f32::sin(count as f32 / 100.),
        );
        buffer.extend(quat.to_array().iter().flat_map(|x| x.to_le_bytes()));

        let vec = glam::Vec3::new(0., 0., -f32::sin(count as f32 / 10.) * 3.);
        buffer.extend(vec.to_array().iter().flat_map(|x| x.to_le_bytes()));

        buffer.push(0xff);
        socket.send(&buffer).await?;
        count += 1;
        buffer.clear();
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}
