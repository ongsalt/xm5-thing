use tokio::sync::mpsc::Receiver;

// from https://github.com/AndreasOlofsson/mdr-protocol

const TANDEM_FRAME_START: u8 = 0x3e; // <
const TANDEM_FRAME_END: u8 = 0x3c; // >
const TANDEM_ESCAPE: u8 = 0x3d; // =
const TANDEM_ESCAPE_MASK: u8 = 0b11101111;

#[derive(Debug)]
pub struct Frame {
    pub packet_type: u8,
    pub sequence_number: u8,
    pub content: Vec<u8>,
}

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

#[derive(Debug)]
pub enum FrameParseError {
    InvalidCheckSum { expected: u8, actual: u8 },
    TooSmall,
    InvalidFormat,
    IncorrectLenght,
}

// bytes stream -> Packet stream -> packet stream ->

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
        if value.len() < checksum_pos + 1 {
            return Err(FrameParseError::IncorrectLenght);
        }

        let checksum = value[checksum_pos];
        let packet = Frame::new(value[0], value[1], value[6..checksum_pos].into());
        let c = packet.checksum();
        if checksum != c {
            return Err(FrameParseError::InvalidCheckSum {
                actual: checksum,
                expected: c,
            });
        }

        Ok(packet)
    }
}

impl Into<Vec<u8>> for &Frame {
    fn into(self) -> Vec<u8> {
        let mut payload = vec![self.packet_type, self.sequence_number];

        for b in (self.content.len() as u32).to_be_bytes() {
            payload.push(b);
        }

        payload.extend_from_slice(&self.content);
        payload.push(self.checksum());

        let escaped = escape(&payload);
        payload.clear();

        payload.push(TANDEM_FRAME_START);
        payload.extend_from_slice(&escaped);
        payload.push(TANDEM_FRAME_END);

        payload
    }
}

impl Frame {
    pub fn new(packet_type: u8, sequence_number: u8, content: &[u8]) -> Frame {
        Frame {
            packet_type,
            sequence_number,
            content: content.into(),
        }
    }

    pub fn new_ack(sequence_number: u8) -> Frame {
        assert!(sequence_number < 0x02, "seq number must be 0 or 1");
        Frame {
            packet_type: 0x01,
            sequence_number: 1 - sequence_number,
            content: vec![],
        }
    }

    pub fn checksum(&self) -> u8 {
        self.content
            .iter()
            .fold(0, |acc: u8, i| acc.wrapping_add(*i))
            .wrapping_add(self.sequence_number)
            .wrapping_add(self.packet_type)
            .wrapping_add(self.content.len() as u8)
    }

    pub fn from_byte_stream(
        mut bytes_rx: Receiver<u8>,
    ) -> Receiver<Result<Frame, FrameParseError>> {
        let (tx, rx) = tokio::sync::mpsc::channel(512);
        tokio::spawn(async move {
            let mut buffer: Vec<u8> = vec![];
            let mut escape_next = false;
            println!("Waiting ");
            while let Some(byte) = bytes_rx.recv().await {
                // print!("{:02x} ", byte);
                match byte {
                    TANDEM_FRAME_START => {
                        buffer.clear();
                        buffer.push(TANDEM_FRAME_START);
                    }
                    TANDEM_FRAME_END => {
                        buffer.push(TANDEM_FRAME_END);
                        tx.send(Frame::try_from(buffer.as_slice())).await.unwrap();
                    }
                    TANDEM_ESCAPE => escape_next = true,
                    _ => buffer.push(if escape_next { unescape(byte) } else { byte }),
                };
            }
            println!("Done (packet) bytes_rx.is_closed:{}", bytes_rx.is_closed());
        });

        rx
    }
}
