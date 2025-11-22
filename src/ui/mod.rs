pub mod app;
mod components;
mod query;
mod spring;
mod state;
mod terminal;

use app::app;
use freya::launch::launch;
use freya::{launch::launch_cfg, prelude::LaunchConfig};

use crate::ui::terminal::start_ratatui;

pub fn start() {
    // start_ratatui();
    start_inner()
}

#[cfg(target_os = "windows")]
fn start_inner() {
    use winit::platform::windows::{BackdropType, WindowAttributesExtWindows};

    launch_cfg(
        app,
        LaunchConfig::<()>::new()
            // .with_size(320f64, 480f64)
            .with_background("transparent")
            .with_transparency(true)
            .with_window_attributes(|attr| attr.with_system_backdrop(BackdropType::MainWindow)),
    );
}

#[cfg(not(target_os = "windows"))]
fn start_inner() {
    launch(app);
}
