use chrono::{DateTime, Utc};
use freya::prelude::*;

use crate::{
    platforms::{
        traits::DeviceCommunication, windows::WindowsDeviceCommunication,
    },
    protocols::{
        connection::{HeadphoneAppCommand, HeadphoneConnection},
    },
};

pub struct AppState<D: DeviceCommunication> {
    connection: Option<HeadphoneConnection<D>>,
    log: Vec<Log>,
}

impl<D: DeviceCommunication> AppState<D> {
    fn new() -> Self {
        Self {
            connection: None,
            log: vec![],
        }
    }
}

pub struct Log {
    timestamp: DateTime<Utc>,
    message: String,
}

pub fn use_app_state() -> (
    Signal<AppState<WindowsDeviceCommunication>>,
    Coroutine<HeadphoneAppCommand>,
) {
    let mut app_state = use_signal(|| AppState::<WindowsDeviceCommunication>::new());

    // actor model as its finest,
    // this pretty much look like elm pattern tho
    let c: Coroutine<HeadphoneAppCommand> = use_coroutine(move |command_rx| async move {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(512);

        tokio::spawn(async move {
            command_rx;
            // put this in a callback and be done
            tx.send(AppState::<WindowsDeviceCommunication>::new())
                .await
                .unwrap();
        });

        while let Some(value) = rx.recv().await {
            *app_state.write() = value;
        }
    });

    // TODO: make this readonly
    (app_state, c)
}

pub fn start_actors() {}
