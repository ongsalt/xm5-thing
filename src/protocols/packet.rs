use std::{sync::mpsc::Receiver, thread};

// https://github.com/AndreasOlofsson/mdr-protocol
const TANDEM_FRAME_START: u8 = 0x3e; // <
const TANDEM_FRAME_END: u8 = 0x3c; // >
const TANDEM_ESCAPE: u8 = 0x3d; // =
const TANDEM_ESCAPE_MASK: u8 = 0b11101111;

#[derive(Debug)]
pub struct Packet {
    packet_type: u8,
    sequence_number: u8,
    content: Vec<u8>,
}

fn escape(byte: u8) -> u8 {
    byte | (!TANDEM_ESCAPE_MASK & 0xFF)
}

pub enum PacketParseError {
    InvalidCheckSum,
    InvalidFormat,
}

// bytes stream -> Packet stream -> packet stream ->

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    // without
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 7 {
            return Err(PacketParseError::InvalidFormat);
        }

        let lenght = u32::from_be_bytes(value[2..6].try_into().unwrap());
        let checksum_pos = (lenght + 6) as usize;
        if value.len() < checksum_pos + 1 {
            return Err(PacketParseError::InvalidFormat);
        }

        let checksum = value[checksum_pos];
        let packet = Packet::new(value[0], value[1], value[6..checksum_pos].into());
        if checksum != packet.checksum() {
            return Err(PacketParseError::InvalidCheckSum);
        }

        Ok(packet)
    }
}

impl Packet {
    pub fn new(packet_type: u8, sequence_number: u8, content: &[u8]) -> Packet {
        Packet {
            packet_type,
            sequence_number,
            content: content.into(),
        }
    }

    pub fn checksum(&self) -> u8 {
        self.content.iter().fold(0, |acc, i| acc + i)
    }

    pub fn from_byte_stream(bytes_rx: Receiver<u8>) -> Receiver<Result<Packet, PacketParseError>> {
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let mut buffer: Vec<u8> = vec![];
            let mut escape_next = false;
            for byte in bytes_rx {
                match byte {
                    TANDEM_FRAME_START => buffer.clear(),
                    TANDEM_FRAME_END => tx.send(Packet::try_from(buffer.as_slice())).unwrap(),
                    TANDEM_ESCAPE => escape_next = true,
                    _ => buffer.push(if escape_next { escape(byte) } else { byte }),
                };
            }
        });
        rx
    }
}
