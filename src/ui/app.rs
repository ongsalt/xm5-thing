use crate::ui::{components::code_block::CodeBlock, state::use_app_state};
use freya::prelude::*;

// wtf did i just wrote
pub fn app() -> Element {
    let (app_state, coroutine) = use_app_state();

    let is_initialized = use_memo(move || (app_state.read().connection).is_some());

    rsx!(
        ScrollView {
            height: "100%",
            width: "100%",
            padding: "16",
            spacing: "8",

            label {
                font_size: "24",

                "Some text"
            }

            if is_initialized() {
                rect {
                    direction: "horizontal",
                    spacing: "6",

                    label {
                        "Connected"
                    }

                    // Button {
                    //     onpress: move |_| service.send(ServiceMessage::SendAck),

                    //     label {
                    //         "Send ack"
                    //     }
                    // }
                }
            }

            CodeBlock {
                title: "Log",
                code: app_state.read().log.clone(), // ??????
            }
        }
    )
}
