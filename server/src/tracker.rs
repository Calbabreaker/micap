use std::time::Instant;

#[derive(Default, PartialEq, Debug, Clone, Copy, serde::Serialize)]
#[repr(u8)]
pub enum TrackerStatus {
    Ok = 0,
    Error = 1,
    #[default]
    Off = 2,
    TimedOut,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
    pub const fn as_unity_bone(&self) -> &'static str {
        match self {
            Self::Hip => "Hips",
            Self::LeftUpperLeg => "LeftUpperLeg",
            Self::RightUpperLeg => "RightUpperLeg",
            Self::LeftLowerLeg => "LeftLowerLeg",
            Self::RightLowerLeg => "RightLowerLeg",
            Self::LeftFoot => "LeftFoot",
            Self::RightFoot => "RightFoot",
            Self::Waist => "Spine",
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
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TrackerInfo {
    pub status: TrackerStatus,
    pub config: TrackerConfig,
    pub latency_ms: Option<u32>,
    pub battery_level: f32,
    pub removed: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TrackerData {
    pub orientation: glam::Quat,
    pub acceleration: glam::Vec3A,
    #[serde(skip)]
    pub velocity: glam::Vec3A,
    pub position: glam::Vec3A,
}

#[derive(Clone, Debug)]
pub struct Tracker {
    pub info: TrackerInfo,
    pub data: TrackerData,
    pub time_data_received: Instant,
}

impl Tracker {
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            info: TrackerInfo {
                config,
                status: TrackerStatus::default(),
                latency_ms: None,
                battery_level: 0.0,
                removed: false,
            },
            data: TrackerData::default(),
            time_data_received: Instant::now(),
        }
    }

    /// Returns true if data was updated
    pub fn tick(&mut self) -> bool {
        if self.info.status == TrackerStatus::Ok && self.data.acceleration.length() > 3. {
            return false;
        }

        let delta = self.time_data_received.elapsed().as_secs_f32();
        self.data.velocity += self.data.acceleration * delta;
        self.data.position += self.data.velocity * delta;
        true
    }
}

/// Seperate from TrackerInfo to be used to save to a file
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TrackerLocation>,
}

impl TrackerConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
