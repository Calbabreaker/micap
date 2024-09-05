use std::{net::Ipv4Addr, time::SystemTime};

use tokio::net::UdpSocket;

use crate::{
    main_server::MainServer,
    tracker::TrackerStatus,
    vmc::packet::{IntoOscMessage, VmcBoneTransformPacket, VmcStatePacket},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VmcConfig {
    enabled: bool,
    marionette_port: u16,
}

impl Default for VmcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            marionette_port: 39540,
        }
    }
}

pub struct VmcConnector {
    socket: UdpSocket,
}

impl VmcConnector {
    pub async fn new() -> anyhow::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        Ok(Self { socket })
    }

    pub async fn update(&mut self, main: &mut MainServer) -> anyhow::Result<()> {
        if !main.config.vmc.enabled {
            return Ok(());
        }

        if self
            .socket
            .peer_addr()
            .map_or(true, |addr| addr.port() != main.config.vmc.marionette_port)
        {
            self.socket
                .connect((Ipv4Addr::LOCALHOST, main.config.vmc.marionette_port))
                .await?;
        }

        let osc_messages = std::iter::empty()
            .chain(main.trackers.iter().filter_map(|(id, tracker)| {
                if tracker.info.status != TrackerStatus::Ok {
                    return None;
                }

                Some(
                    VmcBoneTransformPacket {
                        bone: main.config.trackers[id]
                            .location?
                            .as_unity_bone()
                            .to_string(),
                        position: tracker.data.position,
                        orientation: tracker.data.orientation,
                    }
                    .into_osc_message(),
                )
            }))
            .chain([VmcStatePacket { loaded: true }.into_osc_message()])
            .map(rosc::OscPacket::Message)
            .collect();
        self.send_osc_bundle(osc_messages).await.ok();
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
