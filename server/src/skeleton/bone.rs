#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoneLocation {
    Hip,
    LeftThigh,
    RightThigh,
    LeftKnee,
    RightKnee,
    LeftFoot,
    RightFoot,
    Waist,
    Chest,
    Neck,
    Head,
    LeftShoulder,
    RightShoulder,
    LeftArm,
    RightArm,
    LeftElbow,
    RightElbow,
    LeftHand,
    RightHand,
}

impl BoneLocation {
    // Maps to bone names used in unity, this is also what VRM uses
    // https://docs.unity3d.com/ScriptReference/HumanBodyBones.html
    pub const fn as_unity_bone(&self) -> &'static str {
        match self {
            Self::Hip => "Hips",
            Self::LeftThigh => "LeftUpperLeg",
            Self::RightThigh => "RightUpperLeg",
            Self::LeftKnee => "LeftLowerLeg",
            Self::RightKnee => "RightLowerLeg",
            Self::LeftFoot => "LeftFoot",
            Self::RightFoot => "RightFoot",
            Self::Waist => "Spine",
            Self::Chest => "Chest",
            Self::Neck => "Neck",
            Self::Head => "Head",
            Self::LeftShoulder => "LeftShoulder",
            Self::RightShoulder => "RightShoulder",
            Self::LeftArm => "LeftUpperArm",
            Self::RightArm => "RightUpperArm",
            Self::LeftElbow => "LeftLowerArm",
            Self::RightElbow => "RightLowerArm",
            Self::LeftHand => "LeftHand",
            Self::RightHand => "RightHand",
        }
    }
}

pub struct Bone {
    /// Position of the tail/end of the bone
    position: glam::Vec3A,
    /// Orientation of the head/start of the bone
    orientation: glam::Quat,
    parent: Option<BoneLocation>,
    children: Vec<BoneLocation>,
    location: BoneLocation,
}

impl Bone {
    pub fn set_length(&mut self, length: f32) {
        self.position.y = length;
    }
}
