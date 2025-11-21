use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::protocols::mdr::packet::ConnectedDeviecesRet;

// TODO: find this
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MDRPacketType {
    ConnectedDeviecesGet = 0x36,
    ConnectedDeviecesRet = 0x37,
    MultipointPinningSet = 0x38,
    MultipointActiveDeviceSet = 0x3C,
}

pub enum MDRPacket {
    ConnectedDeviecesGet(packet::ConnectedDeviecesGet),
    ConnectedDeviecesRet(packet::ConnectedDeviecesRet),
    MultipointPinningSet(packet::MultipointPinningSet),
    MultipointActiveDeviceSet(packet::MultipointActiveDeviceSet),
}

impl TryFrom<&[u8]> for MDRPacket {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // TODO: unwrap
        let packet_type = MDRPacketType::try_from(value[0]).unwrap();

        Ok(match packet_type {
            MDRPacketType::ConnectedDeviecesRet => {
                Self::ConnectedDeviecesRet(ConnectedDeviecesRet::try_from(value).unwrap())
            }
            _ => todo!(),
        })
    }
}

// this is for wh-1000xm5 only, at least for now
pub mod packet {
    use std::convert::TryFrom;
    use std::fmt;

    #[derive(Debug)]
    pub enum PacketError {
        BufferTooShort,
        InvalidUtf8(std::string::FromUtf8Error),
    }

    impl fmt::Display for PacketError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                PacketError::BufferTooShort => write!(f, "Buffer too short"),
                PacketError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            }
        }
    }

    impl std::error::Error for PacketError {}

    impl From<std::string::FromUtf8Error> for PacketError {
        fn from(err: std::string::FromUtf8Error) -> Self {
            PacketError::InvalidUtf8(err)
        }
    }

    #[repr(C)]
    pub struct ConnectedDeviecesGet {
        b1: u8, // 02???
    }

    #[derive(Debug)]
    pub struct ConnectedDeviecesRet {
        pub connected_count: u8,
        pub paired_count: u8,
        pub devices: Vec<ConnectedDevice>,
    }

    #[derive(Debug)]
    pub struct MultipointActiveDeviceSet {
        pub flag1: u8,
        pub mac_address: String,
    }

    #[derive(Debug)]
    pub struct MultipointPinningSet {
        // TODO: figure out what these bytes are
        pub payload: Vec<u8>,
    }

    // 17 bytes of mac addr string 💀💀💀 + 4 bytes flags + name.len() + name
    #[derive(Debug)]
    pub struct ConnectedDevice {
        pub mac_address: String,
        pub flags: u32,
        pub name: String,
    }

    impl TryFrom<&[u8]> for MultipointActiveDeviceSet {
        type Error = PacketError;

        fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
            if payload.len() < 19 {
                return Err(PacketError::BufferTooShort);
            }
            let flag1 = payload[1];
            let mac_address = String::from_utf8(payload[2..].to_vec())?;
            Ok(MultipointActiveDeviceSet { flag1, mac_address })
        }
    }

    impl TryFrom<&[u8]> for MultipointPinningSet {
        type Error = PacketError;

        fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
            if payload.len() < 1 {
                return Err(PacketError::BufferTooShort);
            }
            Ok(MultipointPinningSet {
                payload: payload[1..].to_vec(),
            })
        }
    }

    impl TryFrom<&[u8]> for ConnectedDeviecesRet {
        type Error = PacketError;

        fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
            if payload.len() < 3 {
                return Err(PacketError::BufferTooShort);
            }

            let connected_count = payload[1];
            let paired_count = payload[2];
            let mut devices = Vec::new();

            let mut index = 3;
            for _ in 0..paired_count {
                if index + 17 > payload.len() {
                    return Err(PacketError::BufferTooShort);
                }
                let mac_address = String::from_utf8(payload[index..index + 17].to_vec())?;
                index += 17;

                if index + 4 > payload.len() {
                    return Err(PacketError::BufferTooShort);
                }
                let flags = u32::from_be_bytes([
                    payload[index],
                    payload[index + 1],
                    payload[index + 2],
                    payload[index + 3],
                ]);
                index += 4;

                if index + 1 > payload.len() {
                    return Err(PacketError::BufferTooShort);
                }
                let name_len = payload[index] as usize;
                index += 1;

                if index + name_len > payload.len() {
                    return Err(PacketError::BufferTooShort);
                }
                let name = String::from_utf8(payload[index..index + name_len].to_vec())?;
                index += name_len;

                devices.push(ConnectedDevice {
                    mac_address,
                    flags,
                    name,
                });
            }

            // There are 3 more bytes at the end of the payload
            // We don't know what they are yet, but we should probably consume them
            // or at least check if they exist to be safe.
            // For now, we just ignore them as per the python script which returns early.

            Ok(ConnectedDeviecesRet {
                connected_count,
                paired_count,
                devices,
            })
        }
    }
}
