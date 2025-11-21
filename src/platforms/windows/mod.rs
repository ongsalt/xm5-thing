use super::traits::ServiceHandler;
use crate::{platforms::utils::U8ArrayExtension, constant::SONY_SOME_SERVICE_UUID};
use anyhow::{bail, Ok, Result};
use windows::{
    core::GUID,
    Devices::{
        Bluetooth::Rfcomm::{RfcommDeviceService, RfcommServiceId},
        Enumeration::DeviceInformation,
    },
    Networking::Sockets::StreamSocket,
    Storage::Streams::{Buffer, DataReader, DataWriter, InputStreamOptions},
};
use winrt::GuidExtension;

pub mod winrt;

#[derive(Debug, Clone)]
pub struct WindowsServiceHandler {
    pub service: RfcommDeviceService,
    pub data_reader: DataReader,
    data_writer: DataWriter,
    socket: StreamSocket,
}

impl WindowsServiceHandler {
    pub async fn new(service_id: &str) -> Result<WindowsServiceHandler> {
        let id = RfcommServiceId::FromUuid(GUID::parse(service_id)?)?;
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

        let input_stream = socket.InputStream()?;
        let output_stream = socket.OutputStream()?;

        let data_writer = DataWriter::CreateDataWriter(&output_stream)?;
        let data_reader = DataReader::CreateDataReader(&input_stream)?;
        data_reader.SetInputStreamOptions(InputStreamOptions::Partial)?;

        Ok(Self {
            data_reader,
            data_writer,
            service,
            socket,
        })
    }
}

impl ServiceHandler for WindowsServiceHandler {
    async fn send(&self, buffer: &[u8]) -> Result<()> {
        self.data_writer.WriteBytes(buffer)?;
        self.data_writer.StoreAsync()?;

        println!("sent {}: [{}]", buffer.len(), buffer.format_as_hex());
        Ok(())
    }

    fn receive_rx(&self) -> Result<tokio::sync::mpsc::Receiver<u8>> {
        // should this block???
        println!("Receive rx");
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        let data_reader = self.data_reader.clone();

        tokio::spawn(async move {
            loop {
                let size = data_reader.LoadAsync(512).unwrap().await.unwrap() as usize;
                // println!("Got message size: {size}");
                for _ in 0..size {
                    tx.send(data_reader.ReadByte().unwrap()).await.unwrap();
                }
            }
        });

        Ok(rx)
    }

    async fn close(&mut self) {
        todo!()
    }
}
