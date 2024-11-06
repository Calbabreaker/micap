use std::io::Write;

use crate::{
    looper::Looper,
    math::to_euler_angles_vector,
    record::motion_recorder::MotionFrame,
    skeleton::{BoneLocation, SkeletonManager},
};

pub struct BvhSaver<'a, W: Write> {
    buf: W,
    skeleton: &'a SkeletonManager,
    indent_level: usize,
}

impl<'a, W: Write> BvhSaver<'a, W> {
    const INDENT_SIZE: usize = 4;

    pub fn new(buf: W, skeleton: &'a SkeletonManager) -> Self {
        Self {
            buf,
            skeleton,
            indent_level: 0,
        }
    }

    pub fn save(&mut self, frames: &[MotionFrame]) -> std::io::Result<()> {
        self.write("HIERARCHY")?;
        self.write(format!("ROOT {:?}", BoneLocation::ROOT))?;
        self.write_brace()?;
        self.write_vec("OFFSET", glam::Vec3A::ZERO)?;
        // Only the root bone will have the position
        self.write("CHANNELS 6 Xposition Yposition ZPosition Zrotation Xrotation Yrotation")?;

        for location in BoneLocation::ROOT.get_children() {
            self.write_bone_recursive(*location)?;
        }

        self.end_brace()?;

        self.write("MOTION")?;
        self.write(format!("Frames: {}", frames.len()))?;
        self.write(format!(
            "Frame Time: {:.6}",
            Looper::TARGET_LOOP_DELTA.as_secs_f32()
        ))?;

        for frame in frames {
            // Add the root_position and all the orientations
            let mut frame_data = frame.root_position.to_array().to_vec();
            frame_data.extend(
                frame
                    .orientations
                    .iter()
                    .flat_map(|quat| to_euler_angles_vector(*quat, glam::EulerRot::ZXY).to_array()),
            );

            assert_eq!(frame_data.len(), BoneLocation::COUNT * 3 + 3);

            // Convert floats to a long string with only the first decimial digit
            let data_string = frame_data
                .iter()
                .map(|value| format!("{:.1}", value))
                .collect::<Vec<String>>()
                .join(" ");
            writeln!(self.buf, "{}", data_string)?;
        }

        Ok(())
    }

    fn write_bone_recursive(&mut self, location: BoneLocation) -> std::io::Result<()> {
        let bone = &self.skeleton.bones[&location];
        self.write(format!("JOINT {:?}", location))?;
        self.write_brace()?;
        self.write_vec("OFFSET", bone.get_head_offset(&self.skeleton.bones))?;
        // Everything except the root bone will only have rotation
        self.write("CHANNELS 3 Zrotation Xrotation Yrotation")?;

        if location.get_children().is_empty() {
            self.write("End Site")?;
            self.write_brace()?;
            self.write_vec("OFFSET", bone.tail_offset)?;
            self.end_brace()?;
        } else {
            for child_location in location.get_children() {
                self.write_bone_recursive(*child_location)?;
            }
        }

        self.end_brace()?;

        Ok(())
    }

    fn write_vec(&mut self, label: &str, vec: glam::Vec3A) -> std::io::Result<()> {
        self.write(format!("{label} {} {} {}", vec.x, vec.y, vec.z))
    }

    fn write_brace(&mut self) -> std::io::Result<()> {
        self.write("{")?;
        self.indent_level += 1;
        Ok(())
    }

    fn end_brace(&mut self) -> std::io::Result<()> {
        self.indent_level -= 1;
        self.write("}")
    }

    fn write(&mut self, text: impl AsRef<str>) -> std::io::Result<()> {
        writeln!(
            self.buf,
            "{}{}",
            " ".repeat(self.indent_level * Self::INDENT_SIZE),
            text.as_ref()
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        record::{BvhSaver, MotionRecorder},
        skeleton::{BoneLocation, SkeletonConfig, SkeletonManager},
    };

    #[test]
    fn save_correctly() -> anyhow::Result<()> {
        let mut skeleton = SkeletonManager::default();
        skeleton.apply_skeleton_config(&SkeletonConfig::default());
        let bone = skeleton.bones.get_mut(&BoneLocation::RightHip).unwrap();
        bone.local_orientation = glam::Quat::from_euler(glam::EulerRot::ZXY, 0.5, 0.8, 0.8);

        let mut recorder = MotionRecorder::default();
        recorder.start_record();
        recorder.update(&skeleton);

        let mut buf = Vec::new();
        BvhSaver::new(&mut buf, &skeleton).save(recorder.stop_record())?;
        std::fs::write("data/test-out.bvh", &buf)?;
        assert_eq!(
            std::fs::read_to_string("data/test.bvh")?,
            String::from_utf8(buf)?
        );
        std::fs::remove_file("data/test-out.bvh")?;
        Ok(())
    }
}
