// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

use crate::ui::start;

mod platforms;
mod constant;
mod protocols;
mod ui;

// #[tokio::main]
fn main() {
    start();
}

/* 
Expected interface
- start ui and init bt
- ui: select device
- query device info, we gonna have loading state (tanstack query lets goooo)
*/