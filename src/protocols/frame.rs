use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::mpsc::Receiver;

use crate::platforms::utils::U8ArrayExtension;

// from https://github.com/AndreasOlofsson/mdr-protocol

const TANDEM_FRAME_START: u8 = 0x3e; // <
const TANDEM_FRAME_END: u8 = 0x3c; // >
const TANDEM_ESCAPE: u8 = 0x3d; // =
const TANDEM_ESCAPE_MASK: u8 = 0b11101111;

pub fn unescape(byte: u8) -> u8 {
    byte | (!TANDEM_ESCAPE_MASK & 0xFF)
}

pub fn escape(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![];
    for b in bytes {
        match *b {
            TANDEM_FRAME_START | TANDEM_FRAME_END | TANDEM_ESCAPE => {
                out.push(TANDEM_ESCAPE);
                out.push(b & TANDEM_ESCAPE_MASK);
            }
            b => out.push(b),
        };
    }
    out
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum FrameDataType {
    Data = 0x00,
    Ack = 0x01,
    DataMcNo1 = 0x02,
    DataIcd = 0x09,
    DataEv = 0x0a,
    DataMdr = 0x0c,
    DataCommon = 0x0d,
    DataMdrNo2 = 0x0e,
    Shot = 0x10,
    ShotMcNo1 = 0x12,
    ShotIcd = 0x19,
    ShotEv = 0x1a,
    ShotMdr = 0x1c,
    ShotCommon = 0x1d,
    ShotMdrNo2 = 0x1e,
    LargeDataCommon = 0x2d,
}

impl Display for FrameDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub data_type: FrameDataType,
    pub sequence_number: u8,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub enum FrameParseError {
    InvalidCheckSum { expected: u8, actual: u8 },
    TooSmall,
    InvalidFormat,
    IncorrectLenght,
    InvalidDataType,
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Frame(type: {}, seq: {}) {{ {} }}",
            self.data_type,
            self.sequence_number,
            self.content.format_as_hex()
        )
    }
}

// bytes stream -> Frame stream -> (mdr) packet stream ->

impl TryFrom<&[u8]> for Frame {
    type Error = FrameParseError;

    // without
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 9 {
            return Err(FrameParseError::TooSmall);
        }

        if value[0] != TANDEM_FRAME_START || *value.last().unwrap() != TANDEM_FRAME_END {
            return Err(FrameParseError::InvalidFormat);
        };

        let value = &value[1..(value.len() - 1)];

        let lenght = u32::from_be_bytes(value[2..6].try_into().unwrap());
        let checksum_pos: usize = (lenght + 6) as usize;
        let checksum = value[checksum_pos];
        if value.len() < checksum_pos + 1 {
            return Err(FrameParseError::IncorrectLenght);
        }

        let Ok(data_type) = FrameDataType::try_from(value[0]) else {
            return Err(FrameParseError::InvalidDataType);
        };

        let packet = Frame::new(data_type, value[1], value[6..checksum_pos].into());
        let expected = packet.checksum();
        if checksum != expected {
            return Err(FrameParseError::InvalidCheckSum {
                actual: checksum,
                expected,
            });
        }

        Ok(packet)
    }
}

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        let mut payload: Vec<u8> = vec![self.data_type.into(), self.sequence_number];

        for b in (self.content.len() as u32).to_be_bytes() {
            payload.push(b);
        }

        payload.extend_from_slice(&self.content);
        payload.push(self.checksum());

        let escaped = escape(&payload);
        payload.clear(); // reuse

        payload.push(TANDEM_FRAME_START);
        payload.extend_from_slice(&escaped);
        payload.push(TANDEM_FRAME_END);

        payload
    }
}

impl Frame {
    pub fn new(data_type: FrameDataType, sequence_number: u8, content: &[u8]) -> Frame {
        Frame {
            data_type: data_type,
            sequence_number,
            content: content.into(),
        }
    }

    pub fn new_ack(sequence_number: u8) -> Frame {
        // assert!(sequence_number < 0x02, "seq number must be 0 or 1");
        Frame {
            data_type: FrameDataType::Ack,
            sequence_number: 1 - sequence_number,
            content: vec![],
        }
    }

    pub fn checksum(&self) -> u8 {
        self.content
            .iter()
            .fold(0, |acc: u8, i| acc.wrapping_add(*i))
            .wrapping_add(self.sequence_number)
            .wrapping_add(self.data_type.into())
            .wrapping_add(self.content.len() as u8)
    }

    pub fn from_byte_stream(mut bytes_rx: Receiver<Vec<u8>>) -> Receiver<Frame> {
        let (tx, rx) = tokio::sync::mpsc::channel(512);
        tokio::spawn(async move {
            let mut buffer: Vec<u8> = vec![];
            let mut escape_next = false;
            println!("Waiting ");
            while let Some(bytes) = bytes_rx.recv().await {
                for byte in bytes {
                    // print!("{:02x} ", byte);
                    match byte {
                        TANDEM_FRAME_START => {
                            buffer.clear();
                            buffer.push(TANDEM_FRAME_START);
                        }
                        TANDEM_FRAME_END => {
                            buffer.push(TANDEM_FRAME_END);
                            let Ok(frame) = Frame::try_from(buffer.as_slice()) else {
                                // TODO: close stream
                                break;
                            };
                            tx.send(frame).await.unwrap();
                        }
                        TANDEM_ESCAPE => escape_next = true,
                        _ => buffer.push(if escape_next { unescape(byte) } else { byte }),
                    };
                }
            }
            println!("Done (packet) bytes_rx.is_closed:{}", bytes_rx.is_closed());
        });

        rx
    }

    pub fn to_mdr_bytes_stream(
        mut frame_rx: Receiver<Result<Frame, FrameParseError>>,
    ) -> Receiver<u8> {
        let (tx, rx) = tokio::sync::mpsc::channel(512);
        tokio::spawn(async move {
            while let Some(result) = frame_rx.recv().await {
                if let Ok(frame) = result {
                    for b in frame.content {
                        tx.send(b).await.unwrap();
                    }
                } else {
                    break;
                    // TODO: error handling? close stream?
                }
            }
            println!("Done frame_rx.is_closed:{}", frame_rx.is_closed());
        });

        rx
    }
}
