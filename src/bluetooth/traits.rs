use super::BluetoothDeviceInfo;

pub trait BluetoothAdapter {
    fn list_connected_devices(&self) -> Vec<BluetoothDeviceInfo>;
}

// gonna remove this beacuse i just realized that we cant interface with avrcp directly on windows
pub trait ServiceHandler {
    fn send(&mut self, buffer: &[u8]) -> Result<(), ()>;
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ()>;
    fn close(&mut self); // while cant i make drop consuming 
}

