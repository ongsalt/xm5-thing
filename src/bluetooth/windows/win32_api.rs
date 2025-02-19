use std::ffi::{CStr, CString};

use windows::{
    core::{GUID, PCSTR},
    Win32::{
        Devices::Bluetooth::BLUETOOTH_DEVICE_INFO,
        System::Rpc::{UuidFromStringA, RPC_S_INVALID_STRING_UUID},
    },
};

use crate::bluetooth::{BluetoothDeviceInfo, MacAddress};

// why tf is this failing
pub fn to_guid(guid_str: &str) -> Result<GUID, ()> {
    let mut guid = GUID {
        ..Default::default()
    };
    let c_string = CString::new(guid_str).expect("guid should not contain null");
    let ptr = PCSTR::from_raw(c_string.as_ptr() as *const u8);

    unsafe {
        let rpc_status = UuidFromStringA(ptr, &mut guid);
        if rpc_status == RPC_S_INVALID_STRING_UUID {
            eprintln!("invalid guid_str {guid_str}, {guid:?}, {c_string:?}");
            return Err(());
        }
    }

    Ok(guid)
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
