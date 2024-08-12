use std::{net::Ipv4Addr, time::SystemTime};

use anyhow::Context;
use futures_util::FutureExt;
use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::TrackerStatus,
    vmc::packet::{IntoOscMessage, VmcBoneTransformPacket, VmcStatePacket},
};

const VMC_PORT: u16 = 39540;

pub struct VmcConnector {
    socket: UdpSocket,
}

impl VmcConnector {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        socket.connect(("127.0.0.1", VMC_PORT)).await?;
        // socket.send(&[]).await?;
        Ok(Self { socket })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        let osc_messages = std::iter::empty()
            .chain(main.trackers.iter().filter_map(|tracker| {
                if tracker.info.status != TrackerStatus::Ok {
                    return None;
                }

                Some(
                    VmcBoneTransformPacket {
                        bone: tracker.info.config.location?.as_unity_bone().to_string(),
                        position: tracker.data.position,
                        orientation: tracker.data.orientation,
                    }
                    .into_osc_message(),
                )
            }))
            .chain([VmcStatePacket { loaded: false }.into_osc_message()])
            .map(rosc::OscPacket::Message)
            .collect();
        self.send_osc_bundle(osc_messages).await.ok();

        let mut buffer = [0_u8; 256];
        loop {
            // Try and get all the packets that were received
            match self.socket.recv_from(&mut buffer).now_or_never() {
                Some(Ok((amount, peer_addr))) => {
                    log::trace!(
                        "Received {amount} bytes from {peer_addr} (0x{:02x})",
                        buffer[0]
                    );

                    dbg!(&buffer[0..amount]);
                }
                // No more packets
                None => {
                    return Ok(());
                }
                Some(Err(e)) => Err(e)?,
            }
        }
    }

    pub async fn send_osc_bundle(&mut self, messages: Vec<rosc::OscPacket>) -> anyhow::Result<()> {
        let msg_buf = rosc::encoder::encode(&rosc::OscPacket::Bundle(rosc::OscBundle {
            timetag: SystemTime::now().try_into()?,
            content: messages,
        }))?;
        self.socket.send(&msg_buf).await?;
        Ok(())
    }
}
