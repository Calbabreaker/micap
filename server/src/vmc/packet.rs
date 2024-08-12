pub trait IntoOscMessage {
    fn into_osc_message(self) -> rosc::OscMessage;
}

// /VMC/Ext/OK (int){loaded}
pub struct VmcStatePacket {
    pub loaded: bool,
}

impl IntoOscMessage for VmcStatePacket {
    fn into_osc_message(self) -> rosc::OscMessage {
        use rosc::OscType::*;
        rosc::OscMessage {
            addr: "/VMC/Ext/OK".to_string(),
            args: vec![Int(self.loaded as i32)],
        }
    }
}

/// /VMC/Ext/Bone/Pos (string){name} (float){p.x} (float){p.y} (float){p.z} (float){q.x} (float){q.y} (float){q.z} (float){q.w}  
// p=Position
// q=Orientation
pub struct VmcBoneTransformPacket {
    pub bone: String,
    pub position: glam::Vec3A,
    pub orientation: glam::Quat,
}

impl IntoOscMessage for VmcBoneTransformPacket {
    fn into_osc_message(self) -> rosc::OscMessage {
        use rosc::OscType::*;
        rosc::OscMessage {
            addr: "/VMC/Ext/Bone/Pos".to_string(),
            args: vec![
                String(self.bone),
                Float(self.position.x),
                Float(self.position.y),
                Float(self.position.z),
                Float(self.orientation.x),
                Float(self.orientation.y),
                Float(self.orientation.z),
                Float(self.orientation.w),
            ],
        }
    }
}
