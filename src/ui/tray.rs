use std::{process::Command, sync::Arc};

use qmetaobject::{prelude::QObject, qt_base_class, qt_method, qt_property};
use tokio::sync::{RwLock, mpsc::Sender};

use crate::{ActionEvent, config::Config, utils::get_real_directory};

#[derive(QObject, Default)]
pub struct TrayHelper {
    base: qt_base_class!(trait QObject),
    tray_event_tx: Option<Sender<ActionEvent>>,
    config: Arc<RwLock<Config>>,
    record_replays: qt_property!(bool; READ get_record_replays WRITE set_record_replays),
    save_replay: qt_method!(fn(&self)),
    open_replay_directory: qt_method!(fn(&self)),
    quit: qt_method!(fn(&self)),
}

impl TrayHelper {
    pub async fn new(config: Arc<RwLock<Config>>, tray_event_tx: Sender<ActionEvent>) -> Self {
        let record_replays = config.read().await.recording_enabled;
        Self {
            base: Default::default(),
            tray_event_tx: Some(tray_event_tx),
            config: config.clone(),
            record_replays,
            save_replay: Default::default(),
            open_replay_directory: Default::default(),
            quit: Default::default(),
        }
    }

    fn get_record_replays(&self) -> bool {
        let config = self.config.clone();
        futures::executor::block_on(async move { config.read().await.recording_enabled })
    }

    fn set_record_replays(&self, value: bool) {
        let config = self.config.clone();
        futures::executor::block_on(async move { config.write().await.recording_enabled = value });
        self.tray_event_tx
            .as_ref()
            .unwrap()
            .try_send(ActionEvent::ToggleReplay)
            .unwrap();
    }

    fn save_replay(&self) {
        self.tray_event_tx
            .as_ref()
            .unwrap()
            .try_send(ActionEvent::SaveReplay)
            .unwrap();
    }

    fn open_replay_directory(&self) {
        let config = self.config.clone();
        futures::executor::block_on(async move {
            Command::new("xdg-open")
                .arg(get_real_directory(
                    &config.read().await.replay_directory.to_string_lossy(),
                ))
                .spawn();
        });
    }

    fn quit(&self) {
        self.tray_event_tx
            .as_ref()
            .unwrap()
            .try_send(ActionEvent::Quit)
            .unwrap();
    }
}
