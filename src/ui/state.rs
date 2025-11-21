use freya::prelude::*;

pub fn use_device_state() -> Coroutine<HeadphoneUICommand> {
    let mut device_state = use_signal(|| HeadphoneProperties::new());

    // actor model as its finest,
    // this pretty much look like elm pattern tho
    let c: Coroutine<HeadphoneUICommand> = use_coroutine(move |rx| async move {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(512);

        tokio::spawn(async move {
            // put this in a callback and be done
            tx.send(HeadphoneProperties::new()).await.unwrap();
        });

        while let Some(value) = rx.recv().await {
            *device_state.write() = value;
        }
    });

    c
}

pub struct HeadphoneUICommand {}

pub struct HeadphoneProperties {}

impl HeadphoneProperties {
    fn new() -> Self {
        Self {}
    }
}

fn update_it_somehow(headphone_properties: &mut HeadphoneProperties) {}
