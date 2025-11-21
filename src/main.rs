// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

mod bluetooth;
mod constant;
mod protocols;
mod ui;

use ui::platforms::freya::app::App;

// #[tokio::main]
fn main() {
    let mut app = App::new();
    app.start();
}
