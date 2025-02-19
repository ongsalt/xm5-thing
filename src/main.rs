use bluetooth::{
    traits::{BluetoothAdapter, ServiceHandler}, windows::{WindowsBluetoothAdaptor, WindowsServiceHandler}, BluetoothDeviceInfo, MacAddress, Protocol
};
use constant::{FAST_PAIR_SERVICE_UUID, RANDOM_SERVICE_UUID, SONY_SOME_SERVICE_UUID};

mod bluetooth;
mod constant;
mod protocols;

fn main() -> Result<(), ()> {
    let adapter = WindowsBluetoothAdaptor::new().expect("winsock initialization failed");

    let devices = adapter.list_connected_devices();
    let device = &devices[0];
    println!("{device}");


    let mut service_handler =
        WindowsServiceHandler::new(device.clone(), SONY_SOME_SERVICE_UUID, Protocol::RFCOMM)
            .expect("Service initialization failed");

    let mut buffer = [0u8; 512];
    while let Ok(lenght) = service_handler.receive(&mut buffer) {
        println!("{:?}", &buffer[0..lenght]);
        buffer = [0u8; 512];
    }

    Ok(())
}
