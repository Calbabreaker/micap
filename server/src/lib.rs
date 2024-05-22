use futures_util::StreamExt;
use warp::{filters::ws::WebSocket, Filter};

pub const WEBSOCKET_PORT: u16 = 8298;
pub const UDP_PORT: u16 = 5828;

pub fn setup_log() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();
}

pub async fn start_server() {
    tokio::spawn(start_udp_server());

    let websocket = warp::ws().map(|ws: warp::ws::Ws| ws.on_upgrade(on_connect));

    warp::serve(websocket)
        .run(([127, 0, 0, 1], WEBSOCKET_PORT))
        .await;
}

async fn on_connect(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    log::info!("Client connected");
    while let Some(message) = rx.next().await.and_then(|result| {
        result
            .inspect_err(|e| log::error!("websocket error: {e}"))
            .ok()
    }) {
        if let Ok(message) = message.to_str() {
            log::info!("{:?}", message);
            if message.starts_with("SERIAL:") {
                write_serial(message.replace("SERIAL:", "").as_bytes())
                    .inspect_err(|e| log::error!("Failed to write to serial port: {e}"))
                    .ok();
            }
        }
    }
}

fn write_serial(data: &[u8]) -> serialport::Result<()> {
    let ports = serialport::available_ports()?;
    log::info!("Got ports: {ports:?}");
    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .timeout(std::time::Duration::from_millis(10))
        .open()?;
    port.write_all(data)?;
    Ok(())
}

async fn start_udp_server() -> tokio::io::Result<()> {
    let socket = tokio::net::UdpSocket::bind(("255.255.255.255", UDP_PORT)).await?;
    log::info!("Started UDP on {}", socket.local_addr()?);
    loop {
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 24];
        let (amt, src) = socket.recv_from(&mut buf).await?;

        println!("Received from {src}: {:?}", String::from_utf8_lossy(&buf));

        // let mut imu_values = [0f32; 6];
        // f32_from_raw_bytes(&mut imu_values, &buf);

        // println!(
        //     "GYRO: {:?}, \t ACCEL: {:?}",
        //     &imu_values[0..3],
        //     &imu_values[3..6]
        // );
    }
}

fn f32_from_raw_bytes(out: &mut [f32], raw: &[u8]) {
    for (i, value) in out.iter_mut().enumerate() {
        let offset = i * 4;
        *value = f32::from_le_bytes([
            raw[offset],
            raw[offset + 1],
            raw[offset + 2],
            raw[offset + 3],
        ]);
    }
}
