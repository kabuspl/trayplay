use paste::paste;
use std::sync::Arc;

use cstr::cstr;
use qmetaobject::{
    QSingletonInit, QString, QmlEngine, prelude::QObject, qml_register_singleton_instance,
    qt_base_class, qt_method, qt_property, qt_signal,
};
use tokio::sync::{RwLock, mpsc::Sender};

use crate::{ActionEvent, config::Config};

macro_rules! getter {
    ($key: ident, $type: expr) => {
        paste! {
            fn [<get_ $key>](&self) -> $type {
                self.$key
            }
        }
    };

    ($key: ident, $type: expr, cloned) => {
        paste! {
            fn [<get_ $key>](&self) -> $type {
                self.$key.clone()
            }
        }
    };
}

macro_rules! setter {
    ($key: ident, $type: expr) => {
        paste! {
            fn [<set_ $key>](&mut self, value: $type) {
                self.$key = value;
            }
        }
    };
}

macro_rules! property_impl {
    ($key: ident, $type: expr) => {
        getter!($key, $type);
        setter!($key, $type);
    };

    ($key: ident, $type: expr, cloned) => {
        getter!($key, $type, cloned);
        setter!($key, $type);
    };
}

#[derive(QObject, Default)]
pub struct Settings {
    config: Arc<RwLock<Config>>,
    action_event_tx: Option<Sender<ActionEvent>>,
    base: qt_base_class!(trait QObject),
    framerate: qt_property!(u32; READ get_framerate WRITE set_framerate),
    duration: qt_property!(u32; READ get_duration WRITE set_duration),
    quality: qt_property!(usize; READ get_quality WRITE set_quality),
    container: qt_property!(usize; READ get_container WRITE set_container),
    codec: qt_property!(usize; READ get_codec WRITE set_codec),
    directory: qt_property!(QString; READ get_directory WRITE set_directory),
    clear_buffer: qt_property!(bool; READ get_clear_buffer WRITE set_clear_buffer),
    record_replays: qt_property!(bool; READ get_record_replays WRITE set_record_replays),
    apply_config: qt_method!(fn(&self)),
    change: qt_signal!(),
}

impl Settings {
    property_impl!(framerate, u32);
    property_impl!(duration, u32);
    property_impl!(quality, usize);
    property_impl!(container, usize);
    property_impl!(codec, usize);
    property_impl!(directory, QString, cloned);
    property_impl!(clear_buffer, bool);
    property_impl!(record_replays, bool);

    fn apply_config(&self) {
        let mut config = futures::executor::block_on(async { self.config.write().await });
        config.framerate = self.framerate;
        config.clear_buffer_on_save = self.clear_buffer;
        config.replay_directory = self.directory.to_string().into();
        config.replay_duration_secs = self.duration as i64;
        if config.recording_enabled != self.record_replays {
            futures::executor::block_on(async {
                self.action_event_tx
                    .as_ref()
                    .unwrap()
                    .send(ActionEvent::ToggleReplay)
                    .await
                    .unwrap();
            })
        }
        config.recording_enabled = self.record_replays;
        config.codec = self.codec.try_into().unwrap();
        config.container = self.container.try_into().unwrap();
        config.quality = self.quality.try_into().unwrap();
        futures::executor::block_on(async { config.save().await });
    }

    pub async fn new(config: Arc<RwLock<Config>>, action_event_tx: Sender<ActionEvent>) -> Self {
        let config_values = config.read().await;

        Self {
            base: Default::default(),
            change: Default::default(),
            framerate: config_values.framerate,
            duration: config_values.replay_duration_secs as u32,
            quality: config_values.quality as usize,
            container: config_values.container as usize,
            codec: config_values.codec as usize,
            directory: config_values.replay_directory.display().to_string().into(),
            clear_buffer: config_values.clear_buffer_on_save,
            record_replays: config_values.recording_enabled,
            apply_config: Default::default(),
            config: config.clone(),
            action_event_tx: Some(action_event_tx),
        }
    }
}

impl QSingletonInit for Settings {
    fn init(&mut self) {}
}

pub fn open_settings(action_event_tx: Sender<ActionEvent>, config: Arc<RwLock<Config>>) {
    tokio::spawn(async move {
        let mut engine = QmlEngine::new();

        let settings = Settings::new(config, action_event_tx).await;

        qml_register_singleton_instance(cstr!("Settings"), 1, 0, cstr!("Settings"), settings);

        engine.load_data(include_str!("../ui/settings.qml").into());

        engine.exec();
    });
}
