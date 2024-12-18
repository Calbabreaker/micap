use byteorder::{LittleEndian, ReadBytesExt};
use std::{io::Read, sync::Arc};

use crate::tracker::TrackerStatus;

pub const PACKET_PING_PONG: u8 = 0x00;
pub const PACKET_HANDSHAKE: u8 = 0x01;
pub const PACKET_TRACKER_STATUS: u8 = 0x02;
pub const PACKET_TRACKER_DATA: u8 = 0x03;
pub const PACKET_BATTERY_LEVEL: u8 = 0x04;

pub enum UdpPacket<'a, R: Read> {
    Handshake(UdpPacketHandshake),
    TrackerData(UdpPacketTrackerData<'a, R>),
    TrackerStatus(UdpPacketTrackerStatus),
    BatteryLevel(UdpPacketBatteryLevel),
    PingPong(UdpPacketPingPong),
}

impl<'a, R: Read> UdpPacket<'a, R> {
    /// Returns (Self, packet_number)
    pub fn parse(bytes: &'a mut R) -> anyhow::Result<(Self, u32)> {
        let packet_type = bytes.read_u8()?;
        let packet_number = bytes.read_u32::<LittleEndian>()?;

        let packet = match packet_type {
            PACKET_HANDSHAKE => Self::Handshake(UdpPacketHandshake::from_bytes(bytes)?),
            PACKET_PING_PONG => Self::PingPong(UdpPacketPingPong::from_bytes(bytes)?),
            PACKET_TRACKER_DATA => Self::TrackerData(UdpPacketTrackerData::from_bytes(bytes)?),
            PACKET_TRACKER_STATUS => {
                Self::TrackerStatus(UdpPacketTrackerStatus::from_bytes(bytes)?)
            }
            PACKET_BATTERY_LEVEL => Self::BatteryLevel(UdpPacketBatteryLevel::from_bytes(bytes)?),
            _ => anyhow::bail!("Invalid packet id"),
        };

        Ok((packet, packet_number))
    }
}

pub struct UdpPacketHandshake {
    pub mac_address: Arc<str>,
}

impl UdpPacketHandshake {
    fn from_bytes(bytes: &mut impl Read) -> std::io::Result<Self> {
        if !bytes_equal(bytes, b"MCDEV") {
            return Err(std::io::ErrorKind::InvalidData)?;
        }

        let mut mac_bytes = [0_u8; 6];
        bytes.read_exact(&mut mac_bytes)?;
        let mac_string = mac_bytes.map(|b| format!("{b:02x}")).join(":");

        Ok(Self {
            mac_address: mac_string.into(),
        })
    }

    /// Represents a server handshake response
    /// PACKET_HANDSHAKE + MCSVR
    pub const SERVER_RESPONSE: &[u8] = &[PACKET_HANDSHAKE, b'M', b'C', b'S', b'V', b'R'];
}

pub struct UdpPacketPingPong {
    pub id: u8,
}

impl UdpPacketPingPong {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn from_bytes(bytes: &mut impl Read) -> std::io::Result<Self> {
        Ok(Self {
            id: bytes.read_u8()?,
        })
    }

    pub const fn to_response(self) -> [u8; 2] {
        [PACKET_PING_PONG, self.id]
    }
}

pub struct UdpPacketTrackerStatus {
    pub tracker_index: u8,
    pub tracker_status: TrackerStatus,
}

impl UdpPacketTrackerStatus {
    fn from_bytes(bytes: &mut impl Read) -> std::io::Result<Self> {
        Ok(Self {
            tracker_index: bytes.read_u8()?,
            tracker_status: match bytes.read_u8()? {
                0 => TrackerStatus::Ok,
                1 => TrackerStatus::Error,
                2 => TrackerStatus::Off,
                _ => return Err(std::io::ErrorKind::InvalidData)?,
            },
        })
    }

    pub const fn to_response(&self) -> [u8; 3] {
        [
            PACKET_TRACKER_STATUS,
            self.tracker_index,
            self.tracker_status as u8,
        ]
    }
}

pub struct UdpTrackerData {
    pub tracker_index: u8,
    pub orientation: glam::Quat,
    pub acceleration: glam::Vec3A,
}

pub struct UdpPacketTrackerData<'a, R: Read> {
    bytes: &'a mut R,
}

impl<'a, R: Read> UdpPacketTrackerData<'a, R> {
    fn from_bytes(bytes: &'a mut R) -> std::io::Result<Self> {
        Ok(Self { bytes })
    }

    pub fn next_data(&mut self) -> std::io::Result<Option<UdpTrackerData>> {
        let tracker_index = self.bytes.read_u8()?;
        // 0xff where the tracker id would usually go signifies the end of the packet
        if tracker_index == 0xff {
            return Ok(None);
        }

        let mut array = [0_f32; 4];
        self.bytes.read_f32_into::<LittleEndian>(&mut array)?;
        let orientation = glam::Quat::from_xyzw(-array[0], array[1], array[2], -array[3]);

        let mut vec = [0_f32; 3];
        self.bytes.read_f32_into::<LittleEndian>(&mut vec)?;
        let acceleration = glam::Vec3A::new(vec[0], vec[2], vec[1]);

        Ok(Some(UdpTrackerData {
            tracker_index,
            orientation,
            acceleration,
        }))
    }
}

#[derive(Debug)]
pub struct UdpPacketBatteryLevel {
    pub battery_level: f32,
}

impl UdpPacketBatteryLevel {
    fn from_bytes(bytes: &mut impl Read) -> std::io::Result<Self> {
        Ok(Self {
            battery_level: bytes.read_f32::<LittleEndian>()?,
        })
    }
}

fn bytes_equal(bytes: &mut impl Read, slice: &[u8]) -> bool {
    for expected in slice {
        if bytes.read_u8().ok() != Some(*expected) {
            return false;
        }
    }

    true
}
