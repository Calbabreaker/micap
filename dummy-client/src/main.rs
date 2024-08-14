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

    #[rustfmt::skip]
    socket.send(&[PACKET_HANDSHAKE, b'M', b'C', b'D', b'E', b'V', id, 0xff, 0xff, 0xff, 0xff, 0xff]).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    #[rustfmt::skip]
    socket.send(&[PACKET_TRACKER_STATUS, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut count = 2_u32;
    let mut buffer = Vec::new();
    loop {
        buffer.push(PACKET_TRACKER_DATA);
        buffer.extend(count.to_le_bytes());
        buffer.push(0x00);

        let quat = glam::Quat::from_axis_angle(glam::Vec3::Y, f32::sin(count as f32 / 100.));
        buffer.extend(quat.to_array().iter().flat_map(|x| x.to_le_bytes()));

        let vec = glam::Vec3::new(0., 0., -f32::sin(count as f32 / 10.) * 2.);
        buffer.extend(vec.to_array().iter().flat_map(|x| x.to_le_bytes()));

        socket.send(&buffer).await?;
        count += 1;
        buffer.clear();
        tokio::time::sleep(Duration::from_millis(6)).await;
    }
}
