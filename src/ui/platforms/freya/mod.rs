
use std::{
    ops::Deref,
    time::{self, Instant, SystemTime},
};

use chrono::Local;
use freya::prelude::*;

mod components;
mod spring;

use components::code_block::{CodeBlock, CodeLine};
use futures::StreamExt;

use crate::{
    bluetooth::{traits::ServiceHandler, utils::U8ArrayExtension, windows::WindowsServiceHandler},
    constant::SONY_SOME_SERVICE_UUID,
};

enum ServiceMessage {
    SendAck,
    SendAck2,
}

// wtf did i just wrote
pub fn app() -> Element {
    println!("[root] Rerun");
    let mut log: Signal<Vec<CodeLine>> = use_signal(|| vec![]);
    let mut add_log = move |content: String| {
        log.write()
            .push(CodeLine::new(Local::now().to_rfc2822(), content));
    };

    let mut is_initialized = use_signal(|| false);

    let service: Coroutine<ServiceMessage> = use_coroutine(move |mut rx| async move {
        let mut service = WindowsServiceHandler::new(SONY_SOME_SERVICE_UUID)
            .await
            .unwrap();
        *is_initialized.write() = true;
        add_log("Initiliazed client".into());
        let mut buffer = [0u8; 512];
        let ack = [0x3e, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x3c];
        let ack2 = [0x3e, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3c];

        let service2 = service.clone();
        spawn(async move {
            loop {
                let Ok(lenght) = service2.receive(&mut buffer).await else {
                    add_log("Failed to receive".into());
                    return;
                };
                add_log(format!(
                    "Receive {} bytes: [{}]",
                    lenght,
                    buffer.format_as_hex_with_lenght(lenght)
                ));

                // if let Ok(_) = service2.send(&ack).await {
                //     add_log(format!(
                //         "Receive {} bytes and sent ack: [{}]",
                //         lenght,
                //         buffer.format_as_hex_with_lenght(lenght)
                //     ));
                // } else {
                //     add_log(format!(
                //         "Receive {} bytes and failed to sent ack: [{}]",
                //         lenght,
                //         buffer.format_as_hex_with_lenght(lenght)
                //     ));
                // }
            }
        });

        while let Some(req) = rx.next().await {
            match req {
                ServiceMessage::SendAck => {
                    if let Ok(_) = service.send(&ack).await {
                        add_log("Sent ack".into())
                    } else {
                        add_log("Failed to send ack".into())
                    }
                }
                ServiceMessage::SendAck2 => {
                    if let Ok(_) = service.send(&ack2).await {
                        add_log("Sent ack2".into())
                    } else {
                        add_log("Failed to send ack2".into())
                    }
                }
                _ => {}
            }
        }
    });

    rsx!(
        ScrollView {
            height: "100%",
            width: "100%",
            padding: "16",
            spacing: "16",

            label {
                font_size: "24",
                
                "Some stupid client"
            }

            if *is_initialized.read() {
                rect {
                    direction: "horizontal",
                    spacing: "6",

                    Button {
                        onpress: move |_| service.send(ServiceMessage::SendAck2),

                        label {
                            "Send ack 2"
                        }
                    }

                    Button {
                        onpress: move |_| service.send(ServiceMessage::SendAck),

                        label {
                            "Send ack"
                        }
                    }
                }
            }

            CodeBlock {
                title: "Log",
                code: log.read().clone(), // ??????
            }
        }
    )
}
