use std::{collections::HashMap, sync::Arc};

use futures_util::{lock::Mutex, SinkExt};
use warp::filters::ws::Message;

use crate::ServerState;

pub const UDP_PORT: u16 = 5828;

pub const PACKET_HEARTBEAT: u8 = 0x00;
pub const PACKET_HANDSHAKE: u8 = 0x01;
pub const PACKET_ACCELERATION: u8 = 0x03;
pub const PACKET_ACK: u8 = 0xff;

pub async fn bind_udp_socket(state: Arc<Mutex<ServerState>>) -> tokio::io::Result<()> {
    let socket = tokio::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)).await?;
    log::info!("Started UDP on {}", socket.local_addr()?);
    loop {
        let mut buffer = [0; 24];
        let (amount, src) = socket.recv_from(&mut buffer).await?;

        log::info!("Received {amount} bytes from {src}");

        match buffer[0] {
            PACKET_HANDSHAKE => {
                if &buffer[1..6] != b"MYCAP" {
                    log::info!("Received invalid handshake packet.");
                    continue;
                }

                let mut send_buf = [PACKET_ACK, PACKET_HANDSHAKE].to_vec();
                send_buf.extend("MYCAP".as_bytes());
                socket.send_to(&send_buf, src).await?;

                if let Some(tx) = &mut state.lock().await.websocket_tx {
                    tx.send(Message::text(format!(
                        "DEVICE-CONNECT:{}",
                        parse_mac_address(&buffer[6..])
                    )))
                    .await
                    .ok();
                }
            }
            PACKET_ACCELERATION => {
                let mut imu_values = [0f32; 3];
                f32_from_raw_bytes(&mut imu_values, &buffer);

                log::info!("ACCEL: {:?}", &imu_values[0..3]);
            }
            _ => (),
        }
    }
}

fn parse_mac_address(bytes: &[u8]) -> String {
    format!(
        "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
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
