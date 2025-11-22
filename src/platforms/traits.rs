use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::BluetoothDeviceInfo;

pub trait BluetoothAdapter {
    fn list_devices(&self) -> Vec<BluetoothDeviceInfo>;
}

pub trait DeviceCommunication {
    fn tx(&self) -> Sender<Vec<u8>>;
    fn rx(&self) -> Receiver<Vec<u8>>;
    fn close(&self);
}

pub trait ServiceHandler {
    async fn send(&self, buffer: &[u8]) -> Result<()>;
    fn receive_rx(&self) -> Result<Receiver<u8>>;
    async fn close(&mut self);
}
