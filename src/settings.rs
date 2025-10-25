use cpp::cpp;
use paste::paste;
use std::sync::Arc;

use cstr::cstr;
use qmetaobject::{
    QSingletonInit, QString, QStringList, QUrl, QVariantList, QmlEngine, prelude::QObject,
    qml_register_singleton_instance, qrc, qt_base_class, qt_method, qt_property, qt_signal,
};
use tokio::sync::{RwLock, mpsc::Sender};

use crate::{ActionEvent, config::Config, utils::get_command_output};

cpp! {{
    #include <QTranslator>
    #include <QQmlEngine>
    #include <QCoreApplication>
}}

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
    file_name_pattern: qt_property!(QString; READ get_file_name_pattern WRITE set_file_name_pattern),
    audio_applications: qt_property!(QStringList; READ get_audio_applications WRITE set_audio_applications),
    audio_devices: qt_property!(QStringList; READ get_audio_devices WRITE set_audio_devices),
    video_sources: qt_property!(QStringList; READ get_video_sources WRITE set_video_sources),
    video_source_choice: qt_property!(QString; READ get_video_source_choice WRITE set_video_source_choice),
    audio_tracks_inner: Vec<Vec<String>>,
    audio_tracks: qt_property!(QVariantList; READ get_audio_tracks NOTIFY change),
    apply_config: qt_method!(fn(&self)),
    remove_audio_source: qt_method!(fn(&mut self, track: usize, source: usize)),
    add_audio_source: qt_method!(fn(&mut self, track: usize, source: QString)),
    remove_audio_track: qt_method!(fn(&mut self, track: usize)),
    add_audio_track: qt_method!(fn(&mut self)),
    move_audio_track: qt_method!(fn(&mut self, track: usize, target_index: usize)),
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
    property_impl!(file_name_pattern, QString, cloned);
    property_impl!(audio_applications, QStringList, cloned);
    property_impl!(audio_devices, QStringList, cloned);
    property_impl!(video_sources, QStringList, cloned);
    property_impl!(video_source_choice, QString, cloned);

    fn get_audio_tracks(&self) -> QVariantList {
        self.audio_tracks_inner
            .iter()
            .map(|track| QStringList::from(track.clone()))
            .collect()
    }

    fn remove_audio_source(&mut self, track: usize, source: usize) {
        self.audio_tracks_inner[track].remove(source);
        self.change();
    }

    fn add_audio_source(&mut self, track: usize, source: QString) {
        self.audio_tracks_inner[track].push(source.into());
        self.change();
    }

    fn remove_audio_track(&mut self, track: usize) {
        self.audio_tracks_inner.remove(track);
        self.change();
    }

    fn add_audio_track(&mut self) {
        self.audio_tracks_inner.push(vec![]);
        self.change();
    }

    fn move_audio_track(&mut self, track_index: usize, target_index: usize) {
        if target_index >= self.audio_tracks_inner.len() {
            let track = self.audio_tracks_inner[track_index].clone();
            self.audio_tracks_inner.remove(track_index);
            self.audio_tracks_inner.push(track);
        } else {
            self.audio_tracks_inner.swap(track_index, target_index);
        }
        self.change();
    }

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
        config.audio_tracks = self
            .audio_tracks_inner
            .iter()
            .map(|track| track.join("|"))
            .collect();
        config.file_name_pattern = self.file_name_pattern.to_string();
        config.screen = self.video_source_choice.to_string();
        futures::executor::block_on(async { config.save().await });
        self.change();
    }

    pub async fn new(config: Arc<RwLock<Config>>, action_event_tx: Sender<ActionEvent>) -> Self {
        let config_values = config.read().await;

        let audio_applications =
            get_command_output("gpu-screen-recorder", &["--list-application-audio"])
                .unwrap()
                .split('\n')
                .filter(|e| !e.is_empty())
                .collect::<QStringList>()
                .clone();

        let audio_devices = get_command_output("gpu-screen-recorder", &["--list-audio-devices"])
            .unwrap()
            .split('\n')
            .filter(|e| {
                // we don't need default devices here, they heave separate redio buttons
                !e.is_empty() && !e.contains("default_output") && !e.contains("default_input")
            })
            .collect::<QStringList>()
            .clone();

        let video_sources = get_command_output("gpu-screen-recorder", &["--list-capture-options"])
            .unwrap()
            .split('\n')
            .filter(|e| !e.is_empty() && *e != "region") // region selection is not supported right now
            .collect::<QStringList>()
            .clone();

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
            audio_applications,
            audio_devices,
            video_sources,
            video_source_choice: QString::from(config_values.screen.clone()),
            audio_tracks_inner: config_values
                .audio_tracks
                .iter()
                .map(|track| track.split("|").map(|s| s.to_string()).collect())
                .collect(),
            audio_tracks: config_values
                .audio_tracks
                .iter()
                .map(|track| QStringList::from(track.split("|").collect::<Vec<&str>>()))
                .collect(),
            file_name_pattern: config_values.file_name_pattern.clone().into(),
            apply_config: Default::default(),
            remove_audio_source: Default::default(),
            add_audio_source: Default::default(),
            remove_audio_track: Default::default(),
            add_audio_track: Default::default(),
            move_audio_track: Default::default(),
            config: config.clone(),
            action_event_tx: Some(action_event_tx),
        }
    }
}

impl QSingletonInit for Settings {
    fn init(&mut self) {}
}

qrc!(settings_ui, "ui" as "ui" {
    "settings.qml",
    "AudioPage.qml",
    "MainPage.qml",
    "components/ConfigLabel.qml",
    "lang/pl_PL.qm",
    "lang/de_DE.qm",
    "lang/fr_FR.qm",
    "lang/es_ES.qm"
});

pub fn open_settings(action_event_tx: Sender<ActionEvent>, config: Arc<RwLock<Config>>) {
    tokio::spawn(async move {
        let mut engine = QmlEngine::new();

        settings_ui(); // Load qrc

        {
            let engine_ptr = engine.cpp_ptr();
            cpp!(unsafe [engine_ptr as "QQmlEngine *"] {
                static QTranslator translator;
                QCoreApplication::removeTranslator(&translator);

                QString lang_id = QLocale::system().name();

                if (lang_id != "en") {
                    if (translator.load(":/ui/lang/" + lang_id + ".qm")) {
                        QCoreApplication::installTranslator(&translator);
                    }
                }

                engine_ptr->retranslate();
            });
        }

        let settings = Settings::new(config, action_event_tx).await;

        qml_register_singleton_instance(cstr!("Settings"), 1, 0, cstr!("Settings"), settings);

        engine.load_url(QUrl::from_user_input("qrc:/ui/settings.qml".into()));

        engine.exec();
    });
}
