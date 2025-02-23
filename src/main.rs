// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

use app::App;
use bluetooth::{
    traits::ServiceHandler,
    windows::{winrt, WindowsServiceHandler},
};
use constant::SONY_SOME_SERVICE_UUID;

mod app;
mod bluetooth;
mod constant;
mod protocols;
mod ui;

// #[tokio::main]
fn main() {
    let mut app = App::new();
    app.start();
}
