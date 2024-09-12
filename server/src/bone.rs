#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum BoneKind {
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

impl BoneKind {
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

#[derive(serde::Serialize)]
pub struct Bone {
    tail_position: glam::Vec3A,
    parent: Box<Bone>,
    children: Vec<Bone>,
}

// impl Bone {
//     pub fn new() -> Self {
//         Self {
//             tail_position: (),
//             parent: (),
//             children: (),
//         }
//     }
// }
