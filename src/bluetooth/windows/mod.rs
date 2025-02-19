use super::traits::{BluetoothAdapter, ServiceHandler};
use super::BluetoothDeviceInfo;
use crate::constant::SERVICE_UUID;
use std::mem;
use winapi::{makeword, to_guid};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Networking::WinSock::{WSAStartup, WSADATA};
use windows::Win32::{
    Devices::Bluetooth::*,
    Networking::WinSock::{
        connect, recv, shutdown, socket, WSAGetLastError, SD_BOTH, SEND_RECV_FLAGS, SOCKET,
        SOCKET_ERROR, SOCK_STREAM,
    },
};

mod winapi;

pub struct WindowsBluetoothAdaptor {}

impl WindowsBluetoothAdaptor {
    pub fn new() -> Self {
        Self {}
    }
}

impl BluetoothAdapter for WindowsBluetoothAdaptor {
    fn init(&mut self) -> Result<(), ()> {
        let mut wsa_data = WSADATA::default();
        unsafe {
            let result = WSAStartup(makeword(2, 2), &mut wsa_data);
            if result == 0 {
                Ok(())
            } else {
                Err(())
            }
        }    
    }
    fn list_connected_devices(&self) -> Vec<BluetoothDeviceInfo> {
        let mut devices = vec![];
        unsafe {
            let search_params: BLUETOOTH_DEVICE_SEARCH_PARAMS = BLUETOOTH_DEVICE_SEARCH_PARAMS {
                dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
                fReturnAuthenticated: false.into(),
                fReturnRemembered: false.into(),
                fReturnUnknown: false.into(),
                fReturnConnected: true.into(),
                fIssueInquiry: true.into(),
                cTimeoutMultiplier: 2,
                hRadio: HANDLE::default(), // Null handle to search all radios
            };

            let mut device_info = BLUETOOTH_DEVICE_INFO {
                dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };

            let h_find = BluetoothFindFirstDevice(&search_params, &mut device_info);

            if h_find.is_err() {
                println!("No Bluetooth devices found.");
                return devices;
            }

            let h_find = h_find.unwrap();

            loop {
                devices.push(BluetoothDeviceInfo::from(&device_info));

                let next_device = BluetoothFindNextDevice(h_find, &mut device_info);
                if next_device.is_err() {
                    // println!("stopping {}", next_device.unwrap_err());
                    break;
                }
            }

            BluetoothFindDeviceClose(h_find).expect("It should close gracefully");
        }

        devices
    }
}


pub struct WindowsServiceHandler {
    pub info: BluetoothDeviceInfo,
    socket: Option<SOCKET>,
}

impl WindowsServiceHandler {
    pub fn new(info: BluetoothDeviceInfo) -> Self {
        Self {
            info: info,
            socket: None,
        }
    }
}

impl ServiceHandler for WindowsServiceHandler {
    fn init(&mut self) -> Result<(), ()> {
        if self.initialized() {
            return Err(());
        }
        let result = unsafe { socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32) };
        if let Ok(socket) = result {
            self.socket = Some(socket);
            Ok(())
        } else {
            Err(())
        }
    }

    fn initialized(&self) -> bool {
        self.socket.is_some()
    }

    fn connect(&mut self) -> Result<(), ()> {
        if !self.initialized() {
            return Err(());
        }

        let service_uuid = to_guid(&SERVICE_UUID);

        unsafe {
            let sockaddr = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                serviceClassId: service_uuid,
                btAddr: (&self.info.address).into(),
                port: 0,
            };
            let result = connect(
                self.socket.unwrap(),
                mem::transmute(&sockaddr),
                mem::size_of::<SOCKADDR_BTH>() as i32,
            );
            if result != 0 {
                eprintln!("[b_connect] {:?}", WSAGetLastError());
                return Err(());
            }
        };

        Ok(())
    }

    // TODO: handle error
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ()> {
        if self.socket.is_none() {
            return Err(());
        }
        let socket = self.socket.unwrap();
        unsafe {
            let lenght = recv(socket, buffer, SEND_RECV_FLAGS::default());
            if lenght == SOCKET_ERROR {
                println!("Error receving. code: {:?}", WSAGetLastError());
                Err(())
            } else {
                Ok(lenght as usize)
            }
        }
    }

    fn close(&mut self) {
        if let Some(socket) = self.socket {
            println!("Closing socket: {:?} for {}", socket, self.info);
            unsafe { shutdown(socket, SD_BOTH) };
        }
    }

    fn send(&mut self, buffer: &[u8]) -> Result<(), ()> {
        todo!()
    }
}

impl Drop for WindowsServiceHandler {
    fn drop(&mut self) {
        self.close();
    }
}
