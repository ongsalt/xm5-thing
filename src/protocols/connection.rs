use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    platforms::{traits::DeviceCommunication, BluetoothDeviceInfo, MacAddress},
    protocols::{
        frame::Frame,
        mdr::{packet, MDRPacket},
        properties::{self, HeadphoneProperties},
    },
};

#[derive(Debug, Clone)]
pub struct HeadphoneConnection<D: DeviceCommunication> {
    // device_info: BluetoothDeviceInfo,
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
    pub async fn new(mut communication: D) -> Self {
        // pub async fn new(device_info: BluetoothDeviceInfo, mut communication: D) -> Self {
        let properties = HeadphoneProperties::new(&mut communication).await;

        Self {
            // device_info,
            properties,
            communication,
        }
    }

    pub async fn send(&mut self, command: HeadphoneAppCommand) {
        let value = vec![];
        self.communication.tx().send(value).await.unwrap();
    }

    // TODO: make this callable only once
    pub fn properties_rx(&self) -> Receiver<HeadphoneProperties> {
        let (tx, rx) = tokio::sync::mpsc::channel(24);
        // this shit again...
        let byte_rx = self.communication.rx();
        let mut frame_rx = Frame::from_byte_stream(byte_rx);
        let communication_tx = self.communication.tx();

        let ack = async move |seq| {
            let frame = Frame::new_ack(seq);
            communication_tx.send(frame.into()).await.unwrap();
        };

        let p: HeadphoneProperties = self.properties.clone();

        tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                println!(" 𐘀 {:.?}", frame);
                ack(frame.sequence_number).await;
                let packets = MDRPacket::from_frame(frame);
                for packet in packets {
                    println!("   𐘀 {:.?}", packet);
                    tx.send(p).await.unwrap();
                }
            }
        });

        rx
    }
}
