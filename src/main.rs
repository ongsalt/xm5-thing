// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

use crate::ui::platforms::freya::start;

mod platforms;
mod constant;
mod protocols;
mod ui;

// #[tokio::main]
fn main() {
    // set up bluetooth based on platform?
    
    
    // start ui based on platform?
    start();
}
