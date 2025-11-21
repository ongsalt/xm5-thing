use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use super::BluetoothDeviceInfo;

pub trait BluetoothAdapter {
    fn list_connected_devices(&self) -> Vec<BluetoothDeviceInfo>;
}

// we need to seperate tx and rx
pub trait ServiceHandler {
    async fn send(&self, buffer: &[u8]) -> Result<()>;
    fn receive_rx(&self) -> Result<Receiver<u8>>;
    async fn close(&mut self); // while cant i make drop consuming
}
