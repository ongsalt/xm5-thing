use super::traits::ServiceHandler;
use crate::constant::SONY_SOME_SERVICE_UUID;
use anyhow::Result;
use windows::{
    core::GUID,
    Devices::{
        Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId},
        Enumeration::DeviceInformation,
    },
    Networking::Sockets::StreamSocket,
    Storage::Streams::{DataReader, InputStreamOptions},
};
use winrt::GuidExtension;

pub mod winrt;

struct WindowsServiceHandler {}

impl WindowsServiceHandler {
    pub async fn new() -> Result<WindowsServiceHandler> {
        let id = RfcommServiceId::FromUuid(GUID::parse(SONY_SOME_SERVICE_UUID)?)?;
        let device_class = RfcommDeviceService::GetDeviceSelector(&id)?;
        let mut services: Vec<DeviceInformation> =
            DeviceInformation::FindAllAsyncAqsFilter(&device_class)?
                .await?
                .into_iter()
                .collect::<Vec<_>>();

        let service = services.pop().unwrap();
        let service = RfcommDeviceService::FromIdAsync(&service.Id()?)?.await?;
        let a = service.Device()?;
        println!("{service:?} {:?}", a.Name()?);

        let socket: StreamSocket = StreamSocket::new()?;
        socket
            .ConnectAsync(
                &service.ConnectionHostName()?,
                &service.ConnectionServiceName()?,
            )?
            .await?;

        let mut buffer = [0u8; 512];
        let input_stream = socket.InputStream()?;

        // binding with async support is really god blessing
        // TODO: handle midway connection closed
        let data_reader = DataReader::CreateDataReader(&input_stream)?;
        data_reader.SetInputStreamOptions(InputStreamOptions::Partial)?;

        //
        while let Ok(size) = data_reader.LoadAsync(64)?.await {
            for i in 0..size {
                buffer[i as usize] = data_reader.ReadByte()?;
            }

            println!("size {size} {:?}", &buffer[0..size as usize]);
        }

        Ok(Self {})
    }
}

impl ServiceHandler for WindowsServiceHandler {
    fn send(&mut self, buffer: &[u8]) -> Result<(), ()> {
        todo!()
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ()> {
        todo!()
    }

    fn close(&mut self) {
        todo!()
    }
}
