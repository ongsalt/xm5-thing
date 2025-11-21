use std::{
    ops::Deref,
    time::{self, Instant, SystemTime},
};

use chrono::Local;
use freya::prelude::*;

pub mod app;
mod components;
mod spring;

use components::code_block::{CodeBlock, CodeLine};
use futures::StreamExt;

use crate::{
    bluetooth::{traits::ServiceHandler, utils::U8ArrayExtension, windows::WindowsServiceHandler},
    constant::SONY_SOME_SERVICE_UUID,
    protocols::frame::{self, Frame},
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
        let mut service: WindowsServiceHandler = WindowsServiceHandler::new(SONY_SOME_SERVICE_UUID)
            .await
            .unwrap();
        *is_initialized.write() = true;
        add_log("Initiliazed client".into());

        let mut service_rx = service.receive_rx().unwrap();

        spawn(async move {
            let mut packets = Frame::from_byte_stream(service_rx);
            while let Some(p) = packets.recv().await {
                let Ok(packet) = p else {
                    continue;
                };
                
                add_log(format!("{:.?}", packet));
                let ack= Frame::new_ack(packet.sequence_number);
                let payload: Vec<u8> = (&ack).into();
                add_log(format!("send {:.?}", payload.format_as_hex()));
                service.send(&payload).await.unwrap();
            }
            println!("Done")
        });

        while let Some(req) = rx.next().await {
            match req {
                ServiceMessage::SendAck => {}
                ServiceMessage::SendAck2 => {}
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
