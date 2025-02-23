// #![cfg_attr(
//     all(not(debug_assertions), target_os = "windows"),
//     windows_subsystem = "windows"
// )]

use app::App;
use bluetooth::windows::winrt;

mod bluetooth;
mod constant;
mod protocols;
mod ui;
mod app;

#[tokio::main]
async fn main() { 
    // let mut app = App::new();
    // app.start();
    winrt::shit().await.expect("bruh");
}
