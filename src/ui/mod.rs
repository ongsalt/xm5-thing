use freya::prelude::*;

mod spring;

pub fn app() -> Element {
    rsx!(
        rect {
            height: "100%",
            width: "100%",
            main_align: "center",
            cross_align: "center",

            label {
                "Hello, World!"
            }
        }
    )
}
