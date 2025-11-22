use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    platforms::{traits::DeviceCommunication, BluetoothDeviceInfo, MacAddress},
    protocols::{
        frame::Frame,
        properties::{self, HeadphoneProperties},
    },
};

pub struct HeadphoneConnection<D: DeviceCommunication> {
    device_info: BluetoothDeviceInfo,
    properties: HeadphoneProperties,
    communication: D,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum HeadphoneAppCommand {
    SwitchDevice(MacAddress),
    EnablePinning(bool),
}

// we should have 1 actor to deal with Actual stuff
// bytes - frame - packet - Connenction - ui
// exposed event on_property_change to ui

impl<D: DeviceCommunication> HeadphoneConnection<D> {
    async fn new(device_info: BluetoothDeviceInfo, mut communication: D) -> Self {
        let properties = HeadphoneProperties::new(&mut communication).await;

        Self {
            device_info,
            properties,
            communication,
        }
    }

    async fn send(&mut self, command: HeadphoneAppCommand) {
        let value = vec![];
        self.communication.tx().send(value).await.unwrap();
    }

    // TODO: make this callable only once
    fn properties_rx(&self) -> Receiver<HeadphoneProperties> {
        let (tx, rx) = tokio::sync::mpsc::channel(24);
        // this shit again...
        let byte_rx = self.communication.rx();
        let mut frame_rx = Frame::from_byte_stream(byte_rx);
        // let mut mdr_rx = Frame::to_mdr_bytes_stream(frame_rx);
        let mut packet_rx = ();

        let mut p: HeadphoneProperties = self.properties.clone();

        tokio::spawn(async move {
            while let Some(result) = frame_rx.recv().await {
                if let Ok(frame) = result {
                    // p.update(packet);
                    println!("{}", frame);
                    tx.send(p).await.unwrap();
                }
            }
        });

        rx
    }
}
