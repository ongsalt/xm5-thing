use crate::platforms::{traits::DeviceCommunication, utils::U8ArrayExtension};
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
pub struct WindowsDeviceCommunication {
    service: RfcommDeviceService,
    data_reader: DataReader,
    data_writer: DataWriter,
    socket: StreamSocket,
}

impl WindowsDeviceCommunication {
    pub async fn new(service_id: &str) -> Result<WindowsDeviceCommunication> {
        let id = RfcommServiceId::FromUuid(GUID::parse(service_id)?)?;
        let device_class = RfcommDeviceService::GetDeviceSelector(&id)?;
        let mut services: Vec<DeviceInformation> =
            DeviceInformation::FindAllAsyncAqsFilter(&device_class)?
                .await?
                .into_iter()
                .collect();

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

impl DeviceCommunication for WindowsDeviceCommunication {
    fn tx(&self) -> tokio::sync::mpsc::Sender<Vec<u8>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(24);
        let data_writer = self.data_writer.clone();

        tokio::spawn(async move {
            while let Some(value) = rx.recv().await {
                data_writer.WriteBytes(&value).unwrap();
                data_writer.StoreAsync().unwrap().await.unwrap();
                let left = data_writer.UnstoredBufferLength().unwrap();
                println!("left: {left}");
            }
        });

        tx
    }

    fn rx(&self) -> tokio::sync::mpsc::Receiver<Vec<u8>> {
        let (tx, rx) = tokio::sync::mpsc::channel(24);
        let data_reader = self.data_reader.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 512];
            loop {
                let size = data_reader.LoadAsync(512).unwrap().await.unwrap() as usize;
                // println!("Got message size: {size}");
                data_reader.ReadBytes(&mut buffer[0..size]).unwrap();
                tx.send(buffer[0..size].to_vec()).await.unwrap();
                // buffer = [0u8; 512]
            }
        });

        rx
    }

    fn close(&self) {
        self.socket.Close().unwrap();
    }
}
