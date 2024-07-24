use std::time::Duration;

#[derive(Default, PartialEq, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum TrackerLocation {
    Hip,
    LeftUpperLeg,
    RightUpperLeg,
    LeftLowerLeg,
    RightLowerLeg,
    LeftFoot,
    RightFoot,
    Waist,
    Chest,
    Neck,
    Head,
    LeftShoulder,
    RightShoulder,
    LeftUpperArm,
    RightUpperArm,
    LeftLowerArm,
    RightLowerArm,
    LeftHand,
    RightHand,
}

impl TrackerLocation {
    // Maps to bone names used in unity, this is also what VRM uses
    // https://docs.unity3d.com/ScriptReference/HumanBodyBones.html
    pub fn to_unity_bone(&self) -> String {
        match self {
            Self::Hip => "Hips",
            Self::LeftUpperLeg => "LeftUpperLeg",
            Self::RightUpperLeg => "RightUpperLeg",
            Self::LeftLowerLeg => "LeftLowerLeg",
            Self::RightLowerLeg => "RightLowerLeg",
            Self::LeftFoot => "LeftFoot",
            Self::RightFoot => "RightFoot",
            Self::Waist => "Waist",
            Self::Chest => "Chest",
            Self::Neck => "Neck",
            Self::Head => "Head",
            Self::LeftShoulder => "LeftShoulder",
            Self::RightShoulder => "RightShoulder",
            Self::LeftUpperArm => "LeftUpperArm",
            Self::RightUpperArm => "RightUpperArm",
            Self::LeftLowerArm => "LeftLowerArm",
            Self::RightLowerArm => "RightLowerArm",
            Self::LeftHand => "LeftHand",
            Self::RightHand => "RightHand",
        }
        .to_string()
    }
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerInfo {
    pub index: usize,
    pub status: TrackerStatus,
    pub config: TrackerConfig,
    pub latency_ms: Option<u32>,
    pub level: Option<f32>,
}

#[derive(Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: glam::Quat,
    pub acceleration: glam::Vec3A,
    pub velocity: glam::Vec3A,
    pub position: glam::Vec3A,
}

#[derive(Clone)]
pub struct Tracker {
    pub id: String,
    pub info: TrackerInfo,
    pub data: TrackerData,
}

impl Tracker {
    pub fn new(id: String, index: usize, config: TrackerConfig) -> Self {
        Self {
            info: TrackerInfo {
                index,
                config,
                status: TrackerStatus::default(),
                latency_ms: None,
                level: None,
            },
            id,
            data: TrackerData::default(),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.data.position += self.data.velocity * delta.as_secs_f32();
    }
}

/// Seperate from TrackerInfo to be used to save to a file
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    pub location: Option<TrackerLocation>,
}
