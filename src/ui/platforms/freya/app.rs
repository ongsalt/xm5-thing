use crate::{
    bluetooth::{traits::ServiceHandler, utils::U8ArrayExtension, windows::WindowsServiceHandler},
    constant::SONY_SOME_SERVICE_UUID,
    protocols::frame::Frame,
    ui::platforms::freya::components::code_block::{CodeBlock, CodeLine},
};
use chrono::Local;
use freya::prelude::*;
use futures::StreamExt;

// wtf did i just wrote
pub fn app() -> Element {
    println!("[root] Rerun");
    let mut log: Signal<Vec<CodeLine>> = use_signal(|| vec![]);
    let mut add_log = move |content: String| {
        log.write()
            .push(CodeLine::new(Local::now().to_string(), content));
    };

    let mut is_initialized = use_signal(|| false);

    let service: Coroutine<()> = use_coroutine(move |mut rx| async move {
        let mut service: WindowsServiceHandler = WindowsServiceHandler::new(SONY_SOME_SERVICE_UUID)
            .await
            .unwrap();
        *is_initialized.write() = true;
        add_log("Initiliazed client".into());

        let service_rx = service.receive_rx().unwrap();

        spawn(async move {
            let mut frames = Frame::from_byte_stream(service_rx);
            while let Some(p) = frames.recv().await {
                let Ok(frame) = p else {
                    continue;
                };

                add_log(format!("{}", frame));
                let ack: Frame = Frame::new_ack(frame.sequence_number);
                let payload: Vec<u8> = (&ack).into();
                add_log(format!("sent {}", payload.format_as_hex()));
                service.send(&payload).await.unwrap();
            }
            println!("Done")
        });

        while let Some(req) = rx.next().await {}
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

                    // Button {
                    //     onpress: move |_| service.send(ServiceMessage::SendAck2),

                    //     label {
                    //         "Send ack 2"
                    //     }
                    // }

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
                code: log.read().clone(), // ??????
            }
        }
    )
}
