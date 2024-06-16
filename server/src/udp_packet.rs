use std::time::Instant;

use crate::tracker::TrackerStatus;
use crate::udp_server::UdpDevice;

pub const PACKET_HEARTBEAT: u8 = 0x00;
pub const PACKET_HANDSHAKE: u8 = 0x01;
pub const PACKET_TRACKER_STATUS: u8 = 0x02;
pub const PACKET_TRACKER_DATA: u8 = 0x03;

pub enum UdpPacket<'a> {
    Handshake(UdpPacketHandshake),
    TrackerData((UdpPacketTrackerData<'a>, usize)),
    TrackerStatus((UdpPacketTrackerStatus, usize)),
    Heartbeat,
}

impl<'a> UdpPacket<'a> {
    pub fn parse(
        bytes: &'a mut std::slice::Iter<'a, u8>,
        mut device: Option<&mut UdpDevice>,
    ) -> Option<Self> {
        let packet_type = *bytes.next()?;

        let packet_number = if packet_type == PACKET_HANDSHAKE {
            // PACKET_HANDSHAKE won't contain the packet number since it should be the first packet in the communication
            0
        } else {
            // Get the packet number from the bytes
            u32_parse(bytes)?
        };

        if let Some(device) = device.as_mut() {
            if packet_number <= device.last_packet_number && packet_type != PACKET_HANDSHAKE {
                log::warn!("Received out of order packet {packet_number}");
                return None;
            }

            device.last_packet_number = packet_number;
            device.last_packet_received_time = Instant::now();
        }

        Some(match packet_type {
            PACKET_HEARTBEAT => Self::Heartbeat,
            PACKET_HANDSHAKE => Self::Handshake(UdpPacketHandshake::from_bytes(bytes)?),
            PACKET_TRACKER_DATA => {
                Self::TrackerData((UdpPacketTrackerData::from_bytes(bytes)?, device?.index))
            }
            PACKET_TRACKER_STATUS => {
                Self::TrackerStatus((UdpPacketTrackerStatus::from_bytes(bytes)?, device?.index))
            }
            _ => return None,
        })
    }
}

pub struct UdpPacketHandshake {
    pub mac_string: String,
}

impl UdpPacketHandshake {
    fn from_bytes(bytes: &mut std::slice::Iter<u8>) -> Option<Self> {
        if !next_equals(bytes, b"MYCAP-DEVICE") {
            return None;
        }

        let mac_string = format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
            bytes.next()?,
        );

        Some(Self { mac_string })
    }

    // \u[1] here means packet handshake (can't combine slices so do it this way)
    pub const RESPONSE: &'static [u8] = "\u{1}MYCAP-SERVER".as_bytes();
}

#[derive(Debug)]
pub struct UdpPacketTrackerStatus {
    pub tracker_index: u8,
    pub tracker_status: TrackerStatus,
}

impl UdpPacketTrackerStatus {
    fn from_bytes(bytes: &mut std::slice::Iter<u8>) -> Option<Self> {
        Some(Self {
            tracker_index: *bytes.next()?,
            tracker_status: match bytes.next()? {
                0 => TrackerStatus::Ok,
                1 => TrackerStatus::Error,
                2 => TrackerStatus::Off,
                _ => return None,
            },
        })
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [
            PACKET_TRACKER_STATUS,
            self.tracker_index,
            self.tracker_status as u8,
        ]
    }
}

#[derive(Debug)]
pub struct UdpTrackerData {
    pub tracker_index: u8,
    pub orientation: glam::Quat,
    pub accleration: glam::Vec3A,
}

pub struct UdpPacketTrackerData<'a> {
    pub num_trackers: usize,
    pub current_tracker_index: usize,
    bytes: &'a mut std::slice::Iter<'a, u8>,
}

impl<'a> UdpPacketTrackerData<'a> {
    fn from_bytes(bytes: &'a mut std::slice::Iter<'a, u8>) -> Option<Self> {
        Some(Self {
            num_trackers: *bytes.next()? as usize,
            current_tracker_index: 0,
            bytes,
        })
    }

    pub fn next(&mut self) -> Option<UdpTrackerData> {
        if self.current_tracker_index >= self.num_trackers {
            return None;
        }

        Some(UdpTrackerData {
            tracker_index: *self.bytes.next()?,
            orientation: glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                f32_parse(self.bytes)?,
                f32_parse(self.bytes)?,
                f32_parse(self.bytes)?,
            ),
            accleration: glam::Vec3A::new(
                f32_parse(self.bytes)?,
                f32_parse(self.bytes)?,
                f32_parse(self.bytes)?,
            ),
        })
    }
}

fn f32_parse(bytes: &mut std::slice::Iter<u8>) -> Option<f32> {
    Some(f32::from_le_bytes([
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
    ]))
}

fn u32_parse(bytes: &mut std::slice::Iter<u8>) -> Option<u32> {
    Some(u32::from_le_bytes([
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
        *bytes.next()?,
    ]))
}

fn next_equals(bytes: &mut std::slice::Iter<u8>, slice: &[u8]) -> bool {
    for expected in slice {
        if bytes.next() != Some(expected) {
            return false;
        }
    }

    true
}
