use std::io::Read;

use crate::math::Vector3;

pub const PACKET_HEARTBEAT: u8 = 0x00;
pub const PACKET_HANDSHAKE: u8 = 0x01;
pub const PACKET_TRACKER_INFO: u8 = 0x02;
pub const PACKET_ACCELERATION: u8 = 0x10;

pub enum UdpPacket {
    Handshake(UdpPacketHandshake),
    Acceleration(UdpPacketAcceleration),
    Hearbeat,
}

impl UdpPacket {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let packet_type = *bytes.first()?;
        let packet_data = bytes.get(1..)?;
        Some(match packet_type {
            PACKET_HEARTBEAT => Self::Hearbeat,
            PACKET_HANDSHAKE => Self::Handshake(UdpPacketHandshake::from_bytes(packet_data)?),
            PACKET_ACCELERATION => {
                Self::Acceleration(UdpPacketAcceleration::from_bytes(packet_data)?)
            }
            _ => return None,
        })
    }
}

pub struct UdpPacketHandshake {
    pub mac_string: String,
}

impl UdpPacketHandshake {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.get(0..12)? != b"MYCAP-DEVICE" {
            return None;
        }

        let mac = bytes.get(12..18)?;
        let mac_string = format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );

        Some(Self { mac_string })
    }

    // \u[1] here means packet handshake (can't combine slices so do it this way)
    pub const RESPONSE: &'static [u8] = "\u{1}MYCAP-SERVER".as_bytes();
}

pub struct UdpPacketAcceleration {
    pub acceleration: Vector3,
}

impl UdpPacketAcceleration {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            acceleration: Vector3::new(
                f32_safe_from_raw(bytes.get(0..4)?),
                f32_safe_from_raw(bytes.get(4..8)?),
                f32_safe_from_raw(bytes.get(8..12)?),
            ),
        })
    }
}

pub fn f32_safe_from_raw(bytes: &[u8]) -> f32 {
    f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
