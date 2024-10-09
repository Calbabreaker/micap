use std::time::Duration;

use micap_server::{
    tracker::TrackerStatus,
    udp::{client::UdpTrackerClient, packet::UdpTrackerData},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let count: u8 = std::env::args()
        .nth(1)
        .unwrap_or("1".to_string())
        .parse()
        .expect("Invalid number");

    println!("Spawning {count} tasks");

    let mut handles = Vec::new();

    for i in 0..count {
        handles.push(tokio::spawn(async move {
            loop {
                if let Err(err) = connect(i).await {
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

async fn connect(id: u8) -> anyhow::Result<()> {
    let mut client = UdpTrackerClient::new().await?;

    client.send_handshake([id, 0, 0, 0, 0, 0]).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    client.send_tracker_status(0, TrackerStatus::Ok).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut count = 2_u32;
    loop {
        let x = (count as f32) + (id as f32);
        let orientation = glam::Quat::from_axis_angle(
            glam::Vec3::new(1., 0., 0.).normalize(),
            f32::sin(count as f32 / 100.),
        );
        let acceleration = glam::Vec3A::new(0., 0., -f32::sin(x / 10.) * 3.);

        let data = UdpTrackerData {
            tracker_index: 0,
            orientation,
            acceleration,
        };

        client.send_tracker_data(&[&data]).await?;
        count += 1;
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}
