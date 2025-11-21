// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

use crate::ui::platforms::freya::start;

mod bluetooth;
mod constant;
mod protocols;
mod ui;

// #[tokio::main]
fn main() {
    start();
}
