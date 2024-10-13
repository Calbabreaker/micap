use std::io::Write;

use crate::{
    looper::Looper,
    record::motion_recorder::MotionFrame,
    skeleton::{to_euler_angles_vector, BoneLocation, SkeletonManager},
};

pub struct BvhSaver<'a, W: Write> {
    buf: W,
    skeleton: &'a SkeletonManager,
}

impl<'a, W: Write> BvhSaver<'a, W> {
    pub fn new(buf: W, skeleton: &'a SkeletonManager) -> Self {
        Self { buf, skeleton }
    }

    pub fn save(&mut self, frames: &[MotionFrame]) -> anyhow::Result<()> {
        writeln!(self.buf, "HIERARCHY")?;
        writeln!(self.buf, "ROOT {:?}", BoneLocation::ROOT)?;
        writeln!(self.buf, "{{")?;
        self.write_vec("OFFSET", glam::Vec3A::ZERO)?;
        // Only the root bone will have the position
        writeln!(
            self.buf,
            "CHANNELS 6 Xposition Yposition ZPosition Zrotation Xrotation Yrotation"
        )?;

        for location in BoneLocation::ROOT.get_children() {
            self.write_bone_recursive(*location)?;
        }

        writeln!(self.buf, "}}")?;

        writeln!(self.buf, "MOTION")?;
        writeln!(self.buf, "Frames: {}", frames.len())?;
        writeln!(
            self.buf,
            "Frame Time: {:.6}",
            Looper::TARGET_LOOP_DELTA.as_secs_f32()
        )?;

        for frame in frames {
            // Add the root_position and all the orientations
            let mut frame_data = frame.root_position.to_array().to_vec();
            frame_data.extend(
                frame
                    .euler_orientations
                    .iter()
                    .flat_map(|quat| to_euler_angles_vector(*quat, glam::EulerRot::ZXY).to_array()),
            );

            assert_eq!(frame_data.len(), BoneLocation::BONE_LOCATION_COUNT * 3 + 3);

            let data_string = frame_data
                .iter()
                .map(|value| format!("{:.1}", value))
                .collect::<Vec<String>>()
                .join(" ");
            writeln!(self.buf, "{}", data_string)?;
        }

        Ok(())
    }

    fn write_bone_recursive(&mut self, location: BoneLocation) -> anyhow::Result<()> {
        let bone = &self.skeleton.bones[&location];
        writeln!(self.buf, "JOINT {:?}", location)?;
        writeln!(self.buf, "{{")?;
        self.write_vec("OFFSET", bone.get_head_offset(&self.skeleton.bones))?;
        // Everything except the root bone will only have rotation
        writeln!(self.buf, "CHANNELS 3 Zrotation Xrotation Yrotation")?;

        if location.get_children().is_empty() {
            writeln!(self.buf, "End Site")?;
            writeln!(self.buf, "{{")?;
            self.write_vec("OFFSET", bone.tail_offset)?;
            writeln!(self.buf, "}}")?;
        } else {
            for child_location in location.get_children() {
                self.write_bone_recursive(*child_location)?;
            }
        }

        writeln!(self.buf, "}}")?;

        Ok(())
    }

    fn write_vec(&mut self, label: &str, vec: glam::Vec3A) -> anyhow::Result<()> {
        writeln!(self.buf, "{label} {} {} {}", vec.x, vec.y, vec.z)?;
        Ok(())
    }
}
