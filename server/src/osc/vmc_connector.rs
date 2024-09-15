use std::{net::Ipv4Addr, time::SystemTime};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use ts_rs::TS;

use crate::main_server::MainServer;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct VmcConfig {
    enabled: bool,
    send_port: u16,
    receive_port: u16,
}

impl Default for VmcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            send_port: 39539,
            receive_port: 39540,
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

    pub async fn update(&mut self, main: &MainServer) -> anyhow::Result<()> {
        if !main.config.vmc.enabled {
            return Ok(());
        }

        let bones = &main.skeleton_manager.bones;

        let osc_messages = bones
            .values()
            .map(|bone| {
                let mut args = vec![rosc::OscType::String(bone.location.as_unity_bone())];
                add_osc_transform_args(
                    &mut args,
                    bone.get_head_position(bones),
                    bone.get_orientation(),
                );
                rosc::OscPacket::Message(rosc::OscMessage {
                    addr: "/VMC/Ext/Bone/Pos".to_string(),
                    args,
                })
            })
            .collect();

        self.send_osc_bundle(osc_messages).await.ok();
        Ok(())
    }

    async fn send_osc_bundle(&mut self, messages: Vec<rosc::OscPacket>) -> anyhow::Result<()> {
        let msg_buf = rosc::encoder::encode(&rosc::OscPacket::Bundle(rosc::OscBundle {
            timetag: SystemTime::now().try_into()?,
            content: messages,
        }))?;
        self.socket.send(&msg_buf).await?;
        Ok(())
    }

    pub async fn apply_config(&mut self, config: &VmcConfig) -> anyhow::Result<()> {
        self.socket
            .connect((Ipv4Addr::LOCALHOST, config.send_port))
            .await?;
        log::info!("Sending VMC packets to {}", self.socket.peer_addr()?);
        Ok(())
    }
}

/// Adds position and orientation data to the current osc message arguments
fn add_osc_transform_args(
    args: &mut Vec<rosc::OscType>,
    position: glam::Vec3A,
    orientation: glam::Quat,
) {
    args.extend([
        rosc::OscType::Float(position.x),
        rosc::OscType::Float(position.y),
        rosc::OscType::Float(position.z),
        rosc::OscType::Float(orientation.x),
        rosc::OscType::Float(orientation.y),
        rosc::OscType::Float(orientation.z),
        rosc::OscType::Float(orientation.w),
    ]);
}
