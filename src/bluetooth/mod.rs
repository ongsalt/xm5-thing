use std::fmt::Display;

use ::windows::Win32::Devices::Bluetooth::{BTHPROTO_L2CAP, BTHPROTO_RFCOMM};

pub mod traits;
pub mod windows;

#[derive(Clone, Copy, Debug)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn new(value: &[u8; 6]) -> MacAddress {
        MacAddress(value.clone())
    }
}

impl Into<u64> for &MacAddress {
    fn into(self) -> u64 {
        u64::from_le_bytes([
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], 0, 0,
        ])
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x?}::{:02x?}::{:02x?}::{:02x?}::{:02x?}::{:02x?}",
            self.0[5], self.0[4], self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

#[derive(Clone, Debug)]
pub struct BluetoothDeviceInfo {
    pub name: String,
    pub address: MacAddress,
}

impl Display for BluetoothDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.address)
    }
}


pub enum Protocol {
    RFCOMM,
    L2CAP
}

impl Protocol {
    fn to_windows_enum(&self) -> u32 {
        match self {
            Protocol::RFCOMM => BTHPROTO_RFCOMM,
            Protocol::L2CAP => BTHPROTO_L2CAP,
        }
    }
}