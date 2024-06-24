use std::time::Instant;

use crate::tracker::TrackerStatus;
use crate::udp_server::UdpDevice;

pub const PACKET_PING_PONG: u8 = 0x00;
pub const PACKET_HANDSHAKE: u8 = 0x01;
pub const PACKET_TRACKER_STATUS: u8 = 0x02;
pub const PACKET_TRACKER_DATA: u8 = 0x03;

pub enum UdpPacket<'a> {
    Handshake(UdpPacketHandshake),
    TrackerData((UdpPacketTrackerData<'a>, &'a mut UdpDevice)),
    TrackerStatus((UdpPacketTrackerStatus, &'a mut UdpDevice)),
    PingPong((UdpPacketPingPong, &'a mut UdpDevice)),
}

impl<'a> UdpPacket<'a> {
    pub fn parse(
        bytes: &'a mut std::slice::Iter<'a, u8>,
        mut device: Option<&'a mut UdpDevice>,
    ) -> Option<Self> {
        let packet_type = *bytes.next()?;

        if let Some(ref mut device) = device {
            match packet_type {
                // These packets don't send a packet number so they will never be discarded
                PACKET_HANDSHAKE | PACKET_PING_PONG => (),
                _ => {
                    // Discard the packet if not the latest
                    let packet_number = u32_parse(bytes)?;
                    if packet_number <= device.last_packet_number {
                        log::warn!("Received out of order packet {packet_number}");
                        return None;
                    }

                    device.last_packet_number = packet_number;
                }
            };

            device.last_packet_received_time = Instant::now();
        }

        Some(match packet_type {
            PACKET_PING_PONG => Self::PingPong((UdpPacketPingPong::from_bytes(bytes)?, device?)),
            PACKET_HANDSHAKE => Self::Handshake(UdpPacketHandshake::from_bytes(bytes)?),
            PACKET_TRACKER_DATA => {
                Self::TrackerData((UdpPacketTrackerData::from_bytes(bytes)?, device?))
            }
            PACKET_TRACKER_STATUS => {
                Self::TrackerStatus((UdpPacketTrackerStatus::from_bytes(bytes)?, device?))
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

    // \u[1] here means packet handshake
    pub const RESPONSE: &'static [u8] = "\u{1}MYCAP-SERVER".as_bytes();
}

pub struct UdpPacketPingPong {
    pub id: u8,
}

impl UdpPacketPingPong {
    pub fn from_bytes(bytes: &mut std::slice::Iter<u8>) -> Option<Self> {
        Some(Self { id: *bytes.next()? })
    }

    pub fn to_bytes(id: u8) -> [u8; 2] {
        [PACKET_PING_PONG, id]
    }
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

    pub const fn to_bytes(&self) -> [u8; 3] {
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
    bytes: &'a mut std::slice::Iter<'a, u8>,
}

impl<'a> UdpPacketTrackerData<'a> {
    fn from_bytes(bytes: &'a mut std::slice::Iter<'a, u8>) -> Option<Self> {
        Some(Self { bytes })
    }

    pub fn next(&mut self) -> Option<UdpTrackerData> {
        let tracker_index = *self.bytes.next()?;
        // 0xff where the tracker id would usually go signifies the end of the packet
        if tracker_index == 0xff {
            return None;
        }

        Some(UdpTrackerData {
            tracker_index,
            orientation: glam::Quat::from_xyzw(
                f32_parse(self.bytes)?,
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
