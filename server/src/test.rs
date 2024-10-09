use std::time::Duration;

use crate::{
    tracker::TrackerStatus,
    udp::{
        client::UdpTrackerClient,
        packet::{UdpPacketPingPong, UdpTrackerData},
    },
    *,
};

#[tokio::test]
async fn tracker_device_send() {
    let mut main = MainServer::default();
    let mut modules = ServerModules::new().await.unwrap();
    let mut client = UdpTrackerClient::new().await.unwrap();

    let mac = [0x69, 0x42, 0, 0, 0, 0];
    client.send_handshake(mac).await.unwrap();

    client
        .send_tracker_status(3, TrackerStatus::Ok)
        .await
        .unwrap();

    let data = UdpTrackerData {
        tracker_index: 3,
        orientation: glam::Quat::from_xyzw(1., 2., 3., 4.),
        acceleration: glam::Vec3A::new(1., 2., 3.),
    };
    client.send_tracker_data(&[&data]).await.unwrap();

    client.send_battery_level(0.2).await.unwrap();

    let packet = UdpPacketPingPong { id: 1 };
    client.socket.send(&packet.to_bytes()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    modules.udp_server.update(&mut main).await.unwrap();

    let tracker = &main.trackers["69:42:00:00:00:00/3"].clone();
    {
        let tracker = tracker.lock().unwrap();
        assert_eq!(tracker.info().status, TrackerStatus::Ok);
        assert_eq!(tracker.info().battery_level, 0.2);
        assert_eq!(
            tracker.info().address.unwrap(),
            client.socket.local_addr().unwrap()
        );
        assert_eq!(tracker.data().acceleration, glam::vec3a(1., 3., 2.));
        assert_eq!(tracker.data().orientation, glam::quat(1., 3., 2., 4.));
    }

    modules.udp_server.upkeep().await.unwrap();

    // Check for latency set
    client.send_ping(1).await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    modules.udp_server.update(&mut main).await.unwrap();

    assert!(tracker.lock().unwrap().info().latency_ms.unwrap() == 250);
}
