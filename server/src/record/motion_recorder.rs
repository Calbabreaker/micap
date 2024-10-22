use crate::skeleton::{BoneLocation, SkeletonManager};

#[derive(Default)]
pub struct MotionFrame {
    pub orientations: [glam::Quat; BoneLocation::COUNT],
    pub root_position: glam::Vec3A,
}

impl MotionFrame {
    pub fn add_orientations_recursive(
        &mut self,
        skeleton: &SkeletonManager,
        location: BoneLocation,
        index: &mut usize,
    ) {
        self.orientations[*index] = skeleton.bones[&location].orientation;
        *index += 1;

        for child_location in location.get_children() {
            self.add_orientations_recursive(skeleton, *child_location, index);
        }
    }
}

#[derive(Default)]
pub struct MotionRecorder {
    frames: Vec<MotionFrame>,
    recording: bool,
}

impl MotionRecorder {
    pub fn start_record(&mut self) {
        log::info!("Started recording");
        self.frames.clear();
        self.recording = true;
    }

    pub fn stop_record(&mut self) -> &Vec<MotionFrame> {
        log::info!("Stopped recording");
        self.recording = false;
        &self.frames
    }

    pub fn update(&mut self, skeleton: &SkeletonManager) {
        if !self.recording {
            return;
        }

        let mut frame = MotionFrame {
            root_position: skeleton.root_position,
            ..Default::default()
        };

        frame.add_orientations_recursive(skeleton, BoneLocation::ROOT, &mut 0);
        self.frames.push(frame);
    }
}
