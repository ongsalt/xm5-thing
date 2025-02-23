use crate::ui::app;
use freya::launch::launch;

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn start(&mut self) {
        self.start_ui();
    }

    #[cfg(target_os = "windows")]
    pub fn start_ui(&self) {
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
    pub fn start_ui(&self) {
        launch(app); // Be aware that this will block the thread
    }
}
