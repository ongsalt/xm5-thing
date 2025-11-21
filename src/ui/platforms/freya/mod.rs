pub mod app;
mod components;
mod spring;

use app::app;
use freya::launch::launch;

pub fn start() {
    start_inner()
}

#[cfg(target_os = "windows")]
fn start_inner() {
    use freya::{launch::launch_cfg, prelude::LaunchConfig};
    use winit::platform::windows::{BackdropType, WindowAttributesExtWindows};

    launch_cfg(
        app,
        LaunchConfig::<()>::new()
            // .with_size(320f64, 480f64)
            .with_background("transparent")
            .with_transparency(true)
            .with_window_attributes(|attr| attr.with_system_backdrop(BackdropType::MainWindow)),
    );

    // launch(app);
}

#[cfg(not(target_os = "windows"))]
fn start_inner() {
    launch(app);
}
