use futures_util::{StreamExt, TryFutureExt};
use warp::{filters::ws::WebSocket, Filter};

pub const WEBSOCKET_PORT: u16 = 35871;

pub async fn start_server() {
    let websocket = warp::ws().map(|ws: warp::ws::Ws| ws.on_upgrade(on_connect));

    log::info!("Starting server on port {WEBSOCKET_PORT}...");
    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket) {
    // Just echo all messages back...
    let (mut tx, mut rx) = ws.split();
    log::info!("Client connected");
    while let Some(message) = rx.next().await.and_then(|result| {
        result
            .inspect_err(|e| log::error!("websocket error: {e}"))
            .ok()
    }) {
        log::info!("{:?}", message.to_str());
    }
}

// fn main() -> std::io::Result<()> {
//     loop {
//         let socket = UdpSocket::bind("0.0.0.0:1234")?;

//         // Receives a single datagram message on the socket. If `buf` is too small to hold
//         // the message, it will be cut off.
//         let mut buf = [0; 24];
//         let (amt, src) = socket.recv_from(&mut buf)?;

//         let mut imu_values = [0f32; 6];
//         f32_from_raw_bytes(&mut imu_values, &buf);

//         println!(
//             "GYRO: {:?}, \t ACCEL: {:?}",
//             &imu_values[0..3],
//             &imu_values[3..6]
//         );
//     }
// }

// fn f32_from_raw_bytes(out: &mut [f32], raw: &[u8]) {
//     for (i, value) in out.iter_mut().enumerate() {
//         let offset = i * 4;
//         *value = f32::from_le_bytes([
//             raw[offset],
//             raw[offset + 1],
//             raw[offset + 2],
//             raw[offset + 3],
//         ]);
//     }
// }
