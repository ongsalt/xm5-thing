use bluetooth::{traits::BluetoothAdapter, windows::WindowsBluetoothAdaptor};

mod constant;
mod bluetooth;
mod protocols;

fn main() -> Result<(), ()> {
    let mut adapter = WindowsBluetoothAdaptor::new();
    // should we do init in new
    // should we move wsa init out of this
    adapter.init().expect("winsock initialization failed");

    let devices = adapter.list_connected_devices();
    println!("{}", devices[0]);
    Ok(())
}

