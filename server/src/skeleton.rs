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

pub struct Skeleton {
    root_bone: Bone,
}

// impl Default for Skeleton {
//     fn default() -> Self {
//         Self { root_bone: Bone }
//     }
// }

