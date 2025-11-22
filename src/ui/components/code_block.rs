use freya::prelude::*;

use crate::ui::state::Log;


#[component]
pub fn CodeBlock(title: String, code: Vec<Log>) -> Element {
    rsx!(
        rect {
            width: "100%",
            corner_radius: "6",
            border: "1 outer rgb(235, 235, 235)",
            background: "white",

            rect {
                width: "100%",
                background: "rgb(250, 250, 250)",
                padding: "12 18",
                corner_radius: "6 6 0 0",
                border: "0 0 1 0 outer rgb(235, 235, 235)",

                label {
                    font_weight: "medium",

                    "{title}"
                }
            }

            ScrollView {
                width: "100%",
                padding: "6 0",
                // height: "240",

                for line in code {
                    rect {
                        width: "100%",
                        padding: "4 18",
                        direction: "horizontal",
                        spacing: "6",
                        font_size: "14",        

                        label {
                            font_weight: "medium",

                            "{line.timestamp.to_string()}"
                        }
                        label {
                            "{line.message}"
                        }
                    }
                }
            }
        }
    )
}
