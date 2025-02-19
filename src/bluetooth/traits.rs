use super::BluetoothDeviceInfo;

pub trait BluetoothAdapter {
    fn init(&mut self) -> Result<(), ()>;
    fn list_connected_devices(&self) -> Vec<BluetoothDeviceInfo>;
}

pub trait ServiceHandler {
    fn init(&mut self) -> Result<(), ()>;
    fn initialized(&self) -> bool;
    fn connect(&mut self) -> Result<(), ()>; // todo: error handling
    fn send(&mut self, buffer: &[u8]) -> Result<(), ()>;
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, ()>;
    fn close(&mut self); 
}

