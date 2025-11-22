use chrono::{DateTime, Local, Utc};
use freya::prelude::*;
use futures::StreamExt;

use crate::{
    constant::SONY_SOME_SERVICE_UUID,
    platforms::{traits::DeviceCommunication, windows::WindowsDeviceCommunication, MacAddress},
    protocols::connection::{HeadphoneAppCommand, HeadphoneConnection},
};

#[derive(Debug)]
pub struct AppState<D: DeviceCommunication> {
    pub connection: Option<HeadphoneConnection<D>>,
    pub log: Vec<Log>,
}

impl<D: DeviceCommunication> AppState<D> {
    fn new() -> Self {
        Self {
            connection: None,
            log: vec![],
        }
    }

    // fn connect(&mut self, mac_address: MacAddress) {
    async fn connect(&mut self, communication: D) {
        self.connection = Some(HeadphoneConnection::new(communication).await)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

pub fn use_app_state() -> (
    Signal<AppState<WindowsDeviceCommunication>>,
    Coroutine<HeadphoneAppCommand>,
) {
    let mut app_state = use_signal(|| AppState::<WindowsDeviceCommunication>::new());
    let mut add_log = move |m: String| {
        println!("[log] {m}");
        app_state.write().log.push(Log {
            timestamp: Local::now().into(),
            message: m,
        });
    };

    // actor model as its finest,
    // this pretty much look like elm pattern tho
    let c: Coroutine<HeadphoneAppCommand> = use_coroutine(move |mut command_rx| async move {
        let communication = WindowsDeviceCommunication::new(SONY_SOME_SERVICE_UUID)
            .await
            .unwrap();

        app_state.write().connect(communication).await;
        add_log("Initialized".into());

        tokio::spawn(async move {
            command_rx;

        });

        let mut rx = app_state.peek().connection.as_ref().unwrap().properties_rx();
        while let Some(value) = rx.recv().await {
            // *app_state.write() = value;
            add_log(format!("{value:.?}"));
        }
    });

    // TODO: make this readonly
    (app_state, c)
}
