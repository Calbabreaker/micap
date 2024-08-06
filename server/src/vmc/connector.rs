use std::{net::Ipv4Addr, time::SystemTime};

use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::TrackerStatus,
    vmc::packet::{IntoOscMessage, VmcBoneTransformPacket},
};

const VMC_PORT: u16 = 39539;

pub struct VmcConnector {
    socket: UdpSocket,
}

impl VmcConnector {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        socket.connect(("127.0.0.1", VMC_PORT)).await?;
        Ok(Self { socket })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        let osc_messages = main
            .trackers
            .iter()
            .filter_map(|tracker| {
                if tracker.info.status != TrackerStatus::Ok {
                    return None;
                }

                Some(rosc::OscPacket::Message(
                    VmcBoneTransformPacket {
                        bone: tracker.info.config.location?.as_unity_bone().to_string(),
                        position: tracker.data.position,
                        orientation: tracker.data.orientation,
                    }
                    .into_osc_message(),
                ))
            })
            .collect();
        self.send_osc_bundle(osc_messages).await.ok();

        Ok(())
    }

    pub async fn send_osc_packet(&mut self, packet: impl IntoOscMessage) -> anyhow::Result<()> {
        let msg_buf = rosc::encoder::encode(&rosc::OscPacket::Message(packet.into_osc_message()))?;
        self.socket.send(&msg_buf).await?;
        Ok(())
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
