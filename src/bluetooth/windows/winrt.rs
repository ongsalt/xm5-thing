use windows::{
    core::GUID,
    Devices::{
        Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId},
        Enumeration::DeviceInformation,
    },
    Networking::Sockets::StreamSocket,
    Storage::Streams::{Buffer, DataReader},
};

use crate::constant::SONY_SOME_SERVICE_UUID;

// too much unwrap
pub async fn shit() -> Result<(), ()> {
    let id = RfcommServiceId::FromUuid(GUID::parse(SONY_SOME_SERVICE_UUID).unwrap()).unwrap();
    let device_class = RfcommDeviceService::GetDeviceSelector(&id).unwrap();
    let mut services: Vec<DeviceInformation> =
        DeviceInformation::FindAllAsyncAqsFilter(&device_class)
            .unwrap()
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

    let service = services.pop().unwrap();
    let service = RfcommDeviceService::FromIdAsync(&service.Id().unwrap())
        .unwrap()
        .await
        .unwrap();
    let a = service.Device().unwrap();
    println!("{service:?} {:?}", a.Name().unwrap());

    let mut socket = StreamSocket::new().unwrap();
    socket
        .ConnectAsync(
            &service.ConnectionHostName().unwrap(),
            &service.ConnectionServiceName().unwrap(),
        )
        .expect("fuck this i will use anyhow")
        .await
        .unwrap();

    let mut buffer = [0u8; 512];
    let input_stream = socket.InputStream().unwrap();

    // TODO: handle midway connection closed 
    let data_reader = DataReader::CreateDataReader(&input_stream).unwrap();
    let size = data_reader.LoadAsync(64).unwrap().await.unwrap();
    println!("size: {size}");
    for i in 0..size {
        buffer[i as usize] = data_reader.ReadByte().unwrap();
    }

    println!("{:?}", &buffer[0..size as usize]);
    Ok(())
}

#[derive(Debug)]
enum GuidParsingError {
    ParseInt,
    Lenght,
}

trait GuidExtension {
    fn parse(s: &str) -> Result<GUID, GuidParsingError>;
}

impl GuidExtension for GUID {
    fn parse(s: &str) -> Result<GUID, GuidParsingError> {
        let mut bytes = [0u8; 16];
        let mut index = 0;
        let parts = s.split('-');
        for part in parts {
            for i in (0..part.len()).step_by(2) {
                if let Ok(digit) = u8::from_str_radix(&part[i..i + 2], 16) {
                    bytes[index] = digit;
                    index += 1;
                } else {
                    return Err(GuidParsingError::ParseInt);
                }
            }
        }

        Ok(GUID::from_values(
            u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
            u16::from_be_bytes(bytes[6..8].try_into().unwrap()),
            bytes[8..16].try_into().unwrap(),
        ))
    }
}
