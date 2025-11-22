use std::fmt;
use std::io::BufReader;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::mpsc::Receiver;

use crate::protocols::frame::{Frame, FrameDataType};

// TODO: find this
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MDRPacketType {
    ConnectGetProtocolInfo = 0x00,
    ConnectRetProtocolInfo = 0x01,
    ConnectGetCapabilityInfo = 0x02,
    ConnectGetDeviceInfo = 0x04,
    ConnectRetDeviceInfo = 0x05,
    ConnectGetSupportFunction = 0x06,
    CommonNtfyBatteryLevel = 0x13,
    ConnectedDeviecesGet = 0x36,
    ConnectedDeviecesRet = 0x37,
    MultipointPinningSet = 0x38,
    MultipointActiveDeviceSet = 0x3C,
    VolumeChangedNotify = 0xA9,
    Test = 0xFF, // its reserved for testing tho
}

// TODO: check v2, this is probably v1
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceInfoInquiredType {
    ModelName = 0x01,
    FwVersion = 0x02,
    SeriesAndColorInfo = 0x03,
    InstructionGuide = 0x04,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum ModelSeries {
    NoSeries = 0x00,
    ExtraBass = 0x10,
    Hear = 0x20,
    Premium = 0x30,
    Sports = 0x40,
    Casual = 0x50,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum ModelColor {
    Default = 0x00,
    Black = 0x01,
    White = 0x02,
    Silver = 0x03,
    Red = 0x04,
    Blue = 0x05,
    Pink = 0x06,
    Yellow = 0x07,
    Green = 0x08,
    Gray = 0x09,
    Gold = 0x0a,
    Cream = 0x0b,
    Orange = 0x0c,
    Brown = 0x0d,
    Violet = 0x0e,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum BatteryInquiredType {
    Battery = 0x00,
    LeftRightBattery = 0x02,
    CradleBattery = 0x03,
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum FunctionType {
    BatteryLevel = 0x11,
    UpscalingIndicator = 0x12,
    CodecIndicator = 0x13,
    BleSetup = 0x14,
    LeftRightBatteryLevel = 0x15,
    LeftRightConnectionStatus = 0x17,
    CradleBatteryLevel = 0x18,
    PowerOff = 0x21,
    ConciergeData = 0x22,
    TandemKeepAlive = 0x23,
    FwUpdate = 0x30,
    PairingDeviceManagementClassicBt = 0x38,
    VoiceGuidance = 0x39,
    Vpt = 0x41,
    SoundPosition = 0x42,
    PresetEq = 0x51,
    Ebb = 0x52,
    PresetEqNoncustomizable = 0x53,
    NoiseCancelling = 0x61,
    NoiseCancellingAndAmbientSoundMode = 0x62,
    AmbientSoundMode = 0x63,
    AutoNcAsm = 0x71,
    NcOptimizer = 0x81,
    VibratorAlertNotification = 0x92,
    PlaybackController = 0xa1,
    TrainingMode = 0xb1,
    ActionLogNotifier = 0xc1,
    GeneralSetting1 = 0xd1,
    GeneralSetting2 = 0xd2,
    GeneralSetting3 = 0xd3,
    ConnectionMode = 0xe1,
    Upscaling = 0xe2,
    Vibrator = 0xf1,
    PowerSavingMode = 0xf2,
    ControlByWearing = 0xf3,
    SmartTalkingMode = 0xf5,
    AutoPowerOff = 0xf4,
    AssignableSettings = 0xf6,
}

#[derive(Debug)]
pub enum PacketError {
    BufferTooShort,
    InvalidUtf8(std::string::FromUtf8Error),
    UnimplementedPacketType(u8),
    InvalidPacketBody(u8),
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketError::BufferTooShort => write!(f, "Buffer too short"),
            PacketError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            PacketError::UnimplementedPacketType(t) => {
                write!(f, "Unimplemented packet type: 0x{:02x}", t)
            }
            PacketError::InvalidPacketBody(t) => {
                write!(f, "Invalid packet body for type: 0x{:02x}", t)
            }
        }
    }
}

impl std::error::Error for PacketError {}

impl From<std::string::FromUtf8Error> for PacketError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        PacketError::InvalidUtf8(err)
    }
}

// 17 bytes of mac addr string 💀💀💀 + 4 bytes flags + name.len() + name
#[derive(Debug, Clone)]
pub struct ConnectedDevice {
    pub mac_address: String,
    pub flags: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ConnectRetDeviceInfo {
    ModelName(String),
    FwVersion(String),
    SeriesAndColorInfo(ModelSeries, ModelColor),
    InstructionGuide(Vec<u8>),
}

impl ConnectRetDeviceInfo {
    pub fn from_bytes(payload: &[u8]) -> Result<(Self, usize), PacketError> {
        if payload.len() < 2 {
            return Err(PacketError::BufferTooShort);
        }
        let inquired_type = DeviceInfoInquiredType::try_from(payload[0])
            .map_err(|_| PacketError::InvalidPacketBody(payload[0]))?;

        match inquired_type {
            DeviceInfoInquiredType::ModelName => {
                let len = payload[1] as usize;
                if payload.len() < 2 + len {
                    return Err(PacketError::BufferTooShort);
                }
                let name = String::from_utf8(payload[2..2 + len].to_vec())?;
                Ok((ConnectRetDeviceInfo::ModelName(name), 2 + len))
            }
            DeviceInfoInquiredType::FwVersion => {
                let len = payload[1] as usize;
                if payload.len() < 2 + len {
                    return Err(PacketError::BufferTooShort);
                }
                let version = String::from_utf8(payload[2..2 + len].to_vec())?;
                Ok((ConnectRetDeviceInfo::FwVersion(version), 2 + len))
            }
            DeviceInfoInquiredType::SeriesAndColorInfo => {
                if payload.len() < 3 {
                    return Err(PacketError::BufferTooShort);
                }
                let series = ModelSeries::try_from(payload[1])
                    .map_err(|_| PacketError::InvalidPacketBody(payload[1]))?;
                let color = ModelColor::try_from(payload[2])
                    .map_err(|_| PacketError::InvalidPacketBody(payload[2]))?;
                Ok((ConnectRetDeviceInfo::SeriesAndColorInfo(series, color), 3))
            }
            DeviceInfoInquiredType::InstructionGuide => {
                let len = payload[1] as usize;
                if payload.len() < 2 + len {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((
                    ConnectRetDeviceInfo::InstructionGuide(payload[2..2 + len].to_vec()),
                    2 + len,
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommonRetBatteryLevel {
    Battery {
        level: u8,
        is_charging: bool,
    },
    LeftRightBattery {
        left_level: u8,
        left_charging: bool,
        right_level: u8,
        right_charging: bool,
    },
    CradleBattery {
        level: u8,
        is_charging: bool,
    },
}

impl CommonRetBatteryLevel {
    pub fn from_bytes(payload: &[u8]) -> Result<(Self, usize), PacketError> {
        if payload.len() < 1 {
            return Err(PacketError::BufferTooShort);
        }
        let inquired_type = BatteryInquiredType::try_from(payload[0])
            .map_err(|_| PacketError::InvalidPacketBody(payload[0]))?;

        match inquired_type {
            BatteryInquiredType::Battery => {
                if payload.len() < 3 {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((
                    CommonRetBatteryLevel::Battery {
                        level: payload[1],
                        is_charging: payload[2] != 0,
                    },
                    3,
                ))
            }
            BatteryInquiredType::LeftRightBattery => {
                if payload.len() < 5 {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((
                    CommonRetBatteryLevel::LeftRightBattery {
                        left_level: payload[1],
                        left_charging: payload[2] != 0,
                        right_level: payload[3],
                        right_charging: payload[4] != 0,
                    },
                    5,
                ))
            }
            BatteryInquiredType::CradleBattery => {
                if payload.len() < 3 {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((
                    CommonRetBatteryLevel::CradleBattery {
                        level: payload[1],
                        is_charging: payload[2] != 0,
                    },
                    3,
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MDRPacket {
    ConnectGetProtocolInfo,
    ConnectRetProtocolInfo {
        protocol_version: u16,
    },
    ConnectGetCapabilityInfo,
    ConnectGetDeviceInfo {
        inquired_type: DeviceInfoInquiredType,
    },
    ConnectRetDeviceInfo(ConnectRetDeviceInfo),
    ConnectGetSupportFunction,
    CommonNtfyBatteryLevel(CommonRetBatteryLevel),
    ConnectedDeviecesGet {
        b1: u8,
    },
    ConnectedDeviecesRet {
        connected_count: u8,
        paired_count: u8,
        devices: Vec<ConnectedDevice>,
    },
    MultipointPinningSet {
        payload: Vec<u8>,
    },
    MultipointActiveDeviceSet {
        flag1: u8,
        mac_address: String,
    },
    VolumeChangedNotify {
        volume: u8,
    },
    Unknown {
        payload: Vec<u8>,
    },
}

impl MDRPacket {
    // TODO: make this a result
    pub fn from_frame(frame: Frame) -> Vec<MDRPacket> {
        if frame.content.is_empty() || frame.data_type != FrameDataType::DataMdr {
            return vec![];
        }

        let packet_type = MDRPacketType::try_from(frame.content[0]).unwrap_or(MDRPacketType::Test);
        let payload = &frame.content;

        match Self::parse_packet(packet_type, payload) {
            Ok((packet, _size)) => vec![packet],
            Err(e) => {
                println!("Error parsing packet: {}", e);
                vec![]
            }
        }
    }

    fn parse_packet(
        packet_type: MDRPacketType,
        payload: &[u8],
    ) -> Result<(MDRPacket, usize), PacketError> {
        match packet_type {
            MDRPacketType::ConnectRetProtocolInfo => {
                if payload.len() < 3 {
                    return Err(PacketError::BufferTooShort);
                }
                let protocol_version = u16::from_be_bytes([payload[1], payload[2]]);
                Ok((MDRPacket::ConnectRetProtocolInfo { protocol_version }, 3))
            }
            MDRPacketType::ConnectRetDeviceInfo => {
                let (info, size) = ConnectRetDeviceInfo::from_bytes(payload)?;
                Ok((MDRPacket::ConnectRetDeviceInfo(info), size))
            }
            MDRPacketType::CommonNtfyBatteryLevel => {
                let (info, size) = CommonRetBatteryLevel::from_bytes(payload)?;
                Ok((MDRPacket::CommonNtfyBatteryLevel(info), size))
            }
            MDRPacketType::ConnectedDeviecesRet => {
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

                    println!("{} {}", connected_count, mac_address);

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
                        println!("[short] 4");
                        return Err(PacketError::BufferTooShort);
                    }
                    let name = String::from_utf8(payload[index..index + name_len].to_vec())?;
                    index += name_len;

                    println!("{:.?}", devices);

                    devices.push(ConnectedDevice {
                        mac_address,
                        flags,
                        name,
                    });
                }

                if index + 3 <= payload.len() {
                    index += 3;
                }

                Ok((
                    MDRPacket::ConnectedDeviecesRet {
                        connected_count,
                        paired_count,
                        devices,
                    },
                    index,
                ))
            }
            MDRPacketType::MultipointActiveDeviceSet => {
                if payload.len() < 19 {
                    return Err(PacketError::BufferTooShort);
                }
                let flag1 = payload[1];
                let mac_address = String::from_utf8(payload[2..19].to_vec())?;
                Ok((
                    MDRPacket::MultipointActiveDeviceSet { flag1, mac_address },
                    19,
                ))
            }
            MDRPacketType::MultipointPinningSet => {
                if payload.len() < 1 {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((
                    MDRPacket::MultipointPinningSet {
                        payload: payload[1..].to_vec(),
                    },
                    payload.len(),
                ))
            }
            MDRPacketType::VolumeChangedNotify => {
                if payload.len() < 3 {
                    return Err(PacketError::BufferTooShort);
                }
                Ok((MDRPacket::VolumeChangedNotify { volume: payload[2] }, 3))
            }
            _ => Ok((
                MDRPacket::Unknown {
                    payload: payload.to_vec(),
                },
                payload.len(),
            )),
        }
    }

    pub fn from_frame_stream(mut frame_rx: Receiver<Frame>) -> Receiver<MDRPacket> {
        let (tx, rx) = tokio::sync::mpsc::channel(512);
        tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                let packets = MDRPacket::from_frame(frame);

                for packet in packets {
                    tx.send(packet).await.unwrap();
                }
            }
            println!("Done (mdr) frame_rx.is_closed:{}", frame_rx.is_closed());
        });

        rx
    }

    // we cant do this unless we have full table of mdr packet because size is unknown
    // fn from_byte_stream(bytes_rx: Receiver<u8>) -> Receiver<MDRPacket> {
    //     let (tx, rx) = tokio::sync::mpsc::channel(512);
    //     tokio::spawn(async move {
    //         let mut buffer: Vec<u8> = vec![];
    //         let mut escape_next = false;
    //         while let Some(byte) = bytes_rx.recv().await {
    //             // print!("{:02x} ", byte);
    //             match byte {
    //             };
    //         }
    //         println!("Done (mdr) bytes_rx.is_closed:{}", bytes_rx.is_closed());
    //     });

    //     rx
    // }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            MDRPacket::ConnectGetProtocolInfo => {
                Some(vec![MDRPacketType::ConnectGetProtocolInfo.into(), 0x00])
            }
            MDRPacket::ConnectGetCapabilityInfo => {
                Some(vec![MDRPacketType::ConnectGetCapabilityInfo.into(), 0x00])
            }
            MDRPacket::ConnectGetDeviceInfo { inquired_type } => Some(vec![
                MDRPacketType::ConnectGetDeviceInfo.into(),
                (*inquired_type).into(),
            ]),
            MDRPacket::ConnectGetSupportFunction => {
                Some(vec![MDRPacketType::ConnectGetSupportFunction.into(), 0x00])
            }
            MDRPacket::ConnectedDeviecesGet { b1 } => {
                Some(vec![MDRPacketType::ConnectedDeviecesGet.into(), *b1])
            }
            MDRPacket::MultipointPinningSet { payload } => {
                let mut bytes = vec![MDRPacketType::MultipointPinningSet.into()];
                bytes.extend_from_slice(payload);
                Some(bytes)
            }
            MDRPacket::MultipointActiveDeviceSet { flag1, mac_address } => {
                let mut bytes = vec![MDRPacketType::MultipointActiveDeviceSet.into(), *flag1];
                bytes.extend(mac_address.as_bytes());
                Some(bytes)
            }
            _ => None,
        }
    }
}
