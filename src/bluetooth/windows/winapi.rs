use windows::{
    core::{GUID, PCSTR},
    Win32::{
        Devices::Bluetooth::BLUETOOTH_DEVICE_INFO,
        System::Rpc::UuidFromStringA,
    },
};

use crate::bluetooth::{BluetoothDeviceInfo, MacAddress};

// todo: cache this
pub fn to_guid(guid_str: &'static str) -> GUID {
    let mut guid = GUID {
        ..Default::default()
    };
    unsafe {
        let _ = UuidFromStringA(PCSTR::from_raw(guid_str.as_ptr()), &mut guid);
    }
    guid
}

pub fn makeword(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

impl From<&BLUETOOTH_DEVICE_INFO> for BluetoothDeviceInfo {
    fn from(value: &BLUETOOTH_DEVICE_INFO) -> Self {
        unsafe {
            Self {
                name: String::from_utf16_lossy(&value.szName),
                address: MacAddress::new(&value.Address.Anonymous.rgBytes),
            }
        }
    }
}
