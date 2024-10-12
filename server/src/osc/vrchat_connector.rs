use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{main_server::MainServer, osc::OscConnector, skeleton::BoneLocation};

#[derive(Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
pub struct VrChatConfig {
    enabled: bool,
    send_port: u16,
    bones_to_send: Vec<BoneLocation>,
}

impl Default for VrChatConfig {
    fn default() -> Self {
        use BoneLocation::*;
        Self {
            enabled: false,
            send_port: 9000,
            bones_to_send: vec![
                Hip,
                Chest,
                LeftFoot,
                RightFoot,
                RightLowerLeg,
                LeftLowerLeg,
                LeftUpperArm,
                RightUpperArm,
            ],
        }
    }
}

pub struct VrChatConnector {
    osc: OscConnector,
}

impl VrChatConnector {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            osc: OscConnector::new().await?,
        })
    }

    pub async fn update(&mut self, main: &MainServer) -> anyhow::Result<()> {
        if !main.config.vrchat.enabled {
            return Ok(());
        }

        let bones = &main.skeleton_manager.bones;

        let bones_to_send = &main.config.vrchat.bones_to_send;

        let osc_messages = bones_to_send.iter().enumerate().flat_map(|(i, location)| {
            let bone = &bones[location];
            let rotation = bone.get_euler_rotation();
            let position = bone.tail_world_position;
            [
                make_pos_message(format!("/tracking/trackers/{i}/position"), position),
                make_pos_message(format!("/tracking/trackers/{i}/position"), rotation),
            ]
        });

        self.osc.send_bundle(osc_messages).await.ok();
        Ok(())
    }

    pub async fn apply_config(&mut self, config: &VrChatConfig) -> anyhow::Result<()> {
        if config.enabled {
            self.osc.connect(config.send_port).await?;
        }

        Ok(())
    }
}

fn make_pos_message(addr: impl ToString, vector: glam::Vec3A) -> rosc::OscPacket {
    rosc::OscPacket::Message(rosc::OscMessage {
        addr: addr.to_string(),
        args: vec![
            rosc::OscType::Float(vector.x),
            rosc::OscType::Float(vector.y),
            rosc::OscType::Float(vector.z),
        ],
    })
}
