use super::super::traits::{BluetoothAdapter, ServiceHandler};
use super::super::{BluetoothDeviceInfo, Protocol};
use std::mem;
use super::win32_api::{makeword, to_guid};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Networking::WinSock::{WSAStartup, WSADATA};
use windows::Win32::{
    Devices::Bluetooth::*,
    Networking::WinSock::{
        connect, recv, shutdown, socket, WSAGetLastError, SD_BOTH, SEND_RECV_FLAGS, SOCKET,
        SOCKET_ERROR, SOCK_STREAM,
    },
};

pub struct Win32BluetoothAdaptor {}

impl Win32BluetoothAdaptor {
    pub fn new() -> Result<Self, ()> {
        let mut wsa_data = WSADATA::default();
        let result = unsafe { WSAStartup(makeword(2, 2), &mut wsa_data) };

        if result == 0 {
            Ok(Self {})
        } else {
            Err(())
        }
    }
}

impl BluetoothAdapter for Win32BluetoothAdaptor {
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
    socket: SOCKET,
    service_uuid: String,
}

impl WindowsServiceHandler {
    pub fn new(
        info: BluetoothDeviceInfo,
        service_uuid: &str,
    ) -> Result<Self, ()> {
        let result = unsafe { socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32) };

        let socket = match result {
            Ok(socket) => socket,
            Err(_) => return Err(()),
        };

        unsafe {
            let sockaddr = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                serviceClassId: to_guid(service_uuid)?,
                btAddr: (&info.address).into(),
                port: 0,
            };
            let result = connect(
                socket,
                mem::transmute(&sockaddr),
                mem::size_of::<SOCKADDR_BTH>() as i32,
            );
            if result != 0 {
                eprintln!("[connect] {:?}", WSAGetLastError());
                return Err(());
            }
        };

        Ok(Self {
            info,
            socket,
            service_uuid: service_uuid.into(),
        })
    }
}

impl ServiceHandler for WindowsServiceHandler {
    // TODO: handle error
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ()> {
        unsafe {
            let lenght = recv(self.socket, buffer, SEND_RECV_FLAGS::default());
            if lenght == SOCKET_ERROR {
                println!("Error receving. code: {:?}", WSAGetLastError());
                Err(())
            } else {
                Ok(lenght as usize)
            }
        }
    }

    fn close(&mut self) {
        println!("Closing socket: {:?} for {}", self.socket, self.info);
        unsafe { shutdown(self.socket, SD_BOTH) };
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
