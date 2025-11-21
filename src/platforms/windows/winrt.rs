use anyhow::{bail, Result};
use windows::core::GUID;

pub trait GuidExtension {
    fn parse(s: &str) -> Result<GUID>;
}

impl GuidExtension for GUID {
    fn parse(s: &str) -> Result<GUID> {
        let mut bytes = [0u8; 16];
        let mut index = 0;
        let parts = s.split('-');
        for part in parts {
            for i in (0..part.len()).step_by(2) {
                let Ok(digit) = u8::from_str_radix(&part[i..i + 2], 16) else {
                    bail!("invalid hex")
                };
                bytes[index] = digit;
                index += 1;
            }
        }

        Ok(GUID::from_values(
            u32::from_be_bytes(bytes[0..4].try_into()?),
            u16::from_be_bytes(bytes[4..6].try_into()?),
            u16::from_be_bytes(bytes[6..8].try_into()?),
            bytes[8..16].try_into()?,
        ))
    }
}
