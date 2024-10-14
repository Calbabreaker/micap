use crate::skeleton::{BoneLocation, SkeletonManager};

#[derive(Default, Clone)]
pub struct MotionFrame {
    pub euler_orientations: [glam::Quat; BoneLocation::COUNT],
    pub root_position: glam::Vec3A,
}

#[derive(Default)]
pub struct MotionRecorder {
    frames: Vec<MotionFrame>,
    current_frame: MotionFrame,
    recording: bool,
}

impl MotionRecorder {
    pub fn start_record(&mut self) {
        self.frames.clear();
        self.recording = true;
    }

    pub fn stop_record(&mut self) -> &Vec<MotionFrame> {
        self.recording = false;
        &self.frames
    }

    pub fn update(&mut self, skeleton: &SkeletonManager) {
        if !self.recording {
            return;
        }

        self.add_orientations_recursive(skeleton, BoneLocation::ROOT, &mut 0);
        self.current_frame.root_position = skeleton.root_position;
        self.frames.push(self.current_frame.clone());
        self.current_frame = MotionFrame::default();
    }

    pub fn add_orientations_recursive(
        &mut self,
        skeleton: &SkeletonManager,
        location: BoneLocation,
        orientations_count: &mut usize,
    ) {
        self.current_frame.euler_orientations[*orientations_count] =
            skeleton.bones[&location].orientation;
        *orientations_count += 1;

        for child_location in location.get_children() {
            self.add_orientations_recursive(skeleton, *child_location, orientations_count);
        }
    }
}
