use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{main_server::MainServer, osc::OscConnector};

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(default)]
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
    osc: OscConnector,
}

impl VmcConnector {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(VmcConnector {
            osc: OscConnector::new().await?,
        })
    }

    pub async fn update(&mut self, main: &MainServer) -> anyhow::Result<()> {
        if !main.config.vmc.enabled {
            return Ok(());
        }

        let bones = &main.skeleton_manager.bones;

        let osc_messages = std::iter::once(rosc::OscPacket::Message(rosc::OscMessage {
            addr: "/VMC/Ext/OK".to_string(),
            args: vec![rosc::OscType::Int(1)],
        }))
        .chain(bones.iter().filter_map(|(location, bone)| {
            let mut args = vec![rosc::OscType::String(location.as_unity_bone()?)];
            // add_osc_transform_args(&mut args, bone.get_head_position(bones), bone.orientation);
            add_osc_transform_args(&mut args, glam::Vec3A::ZERO, bone.orientation);
            Some(rosc::OscPacket::Message(rosc::OscMessage {
                addr: "/VMC/Ext/Bone/Pos".to_string(),
                args,
            }))
        }));

        self.osc.send_bundle(osc_messages).await.ok();
        Ok(())
    }

    pub async fn apply_config(&mut self, config: &VmcConfig) -> anyhow::Result<()> {
        if config.enabled {
            self.osc.connect(config.send_port).await?;
        }
        Ok(())
    }
}

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
