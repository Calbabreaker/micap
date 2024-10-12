use std::time::Duration;

use crate::{
    skeleton::BoneLocation,
    tracker::{TrackerConfig, TrackerStatus},
    udp::{
        client::UdpTrackerClient,
        packet::{UdpPacketPingPong, UdpTrackerData},
    },
    *,
};

// Run tests sequentially since server requires listenting to the port
#[tokio::test]
async fn tests_sequential() -> anyhow::Result<()> {
    // Use spawn to check for Send + Sync
    tokio::spawn(async {
        test_tracker_device_send().await.unwrap();
        test_config().await.unwrap();
    })
    .await?;
    Ok(())
}

async fn test_tracker_device_send() -> anyhow::Result<()> {
    let mut main = MainServer::default();
    let mut modules = ServerModules::new().await?;
    let mut client = UdpTrackerClient::new().await?;

    let mac = [0x69, 0x42, 0, 0, 0, 0];
    client.send_handshake(mac).await?;

    client.send_tracker_status(3, TrackerStatus::Ok).await?;

    let data = UdpTrackerData {
        tracker_index: 3,
        orientation: glam::Quat::from_xyzw(1., 2., 3., 4.),
        acceleration: glam::Vec3A::new(1., 2., 3.),
    };
    client.send_tracker_data(&[&data]).await?;

    client.send_battery_level(0.2).await?;

    let packet = UdpPacketPingPong { id: 1 };
    client.socket.send(&packet.to_bytes()).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    modules.udp_server.update(&mut main).await?;

    let tracker = &main.trackers["69:42:00:00:00:00/3"].clone();
    {
        let tracker = tracker.lock().unwrap();
        assert_eq!(tracker.info().status, TrackerStatus::Ok);
        assert_eq!(tracker.info().battery_level, 0.2);
        assert_eq!(tracker.info().address, client.socket.local_addr().ok());
        assert_eq!(tracker.data().acceleration, glam::vec3a(1., 3., 2.));
        assert_eq!(tracker.data().orientation, glam::quat(1., 3., 2., 4.));
    }

    modules.udp_server.upkeep().await?;

    // Check for latency set
    client.send_ping(1).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    modules.udp_server.update(&mut main).await?;

    assert!(tracker.lock().unwrap().info().latency_ms == Some(250));
    Ok(())
}

async fn test_config() -> anyhow::Result<()> {
    let config_dir = std::env::current_dir()?.join("config_test");
    std::env::set_var("MICAP_CONFIG_DIR", config_dir.clone());

    let tracker_config = TrackerConfig {
        name: Some("hello".to_string()),
        location: Some(BoneLocation::Hip),
    };

    let mut global_config = GlobalConfig::default();
    global_config.trackers.insert("test".into(), tracker_config);
    global_config.save()?;

    let mut main = MainServer::default();
    let mut modules = ServerModules::new().await?;
    main.apply_config(GlobalConfig::load()?, &mut modules)
        .await?;

    assert_eq!(main.config, global_config);
    std::fs::remove_dir_all(config_dir)?;
    Ok(())
}
