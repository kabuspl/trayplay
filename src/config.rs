use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

use crate::ActionEvent;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_bool_true")]
    pub recording_enabled: bool,
    pub screen: String,
    pub container: Container,
    pub audio_tracks: Vec<String>,
    pub framerate: i64,
    pub clear_buffer_on_save: bool,
    pub quality: Quality,
    pub replay_directory: PathBuf,
    pub replay_duration_secs: i64,

    #[serde(skip, default = "Option::default")]
    action_event_tx: Option<Sender<ActionEvent>>,
}

impl Config {
    pub async fn load(action_event_tx: Sender<ActionEvent>) -> Self {
        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        match std::fs::read_to_string(path) {
            Ok(config) => {
                let mut config: Self = toml::from_str(&config).expect("Cannot parse config file");
                config.action_event_tx = Some(action_event_tx);

                config
            }
            Err(_) => Config::default(),
        }
    }

    pub async fn save(&self) {
        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        std::fs::write(path, toml::to_string(&self).unwrap()).expect("Failed to write config file");

        self.action_event_tx
            .as_ref()
            .unwrap()
            .send(ActionEvent::ConfigSaved)
            .await
            .unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        println!("Config missing or broken. Replacing with defaults");

        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        let instance = Self {
            recording_enabled: true,
            screen: "screen".to_string(),
            audio_tracks: vec!["default_output".to_string(), "default_input".to_string()],
            framerate: 60,
            clear_buffer_on_save: true,
            quality: Quality::Ultra,
            replay_directory: dirs::video_dir().unwrap(),
            container: Container::MKV,
            replay_duration_secs: 180,
            action_event_tx: None,
        };

        std::fs::write(path, toml::to_string(&instance).unwrap())
            .expect("Failed to write config file");

        instance
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Medium,
    High,
    VeryHigh,
    Ultra,
}

impl ToString for Quality {
    fn to_string(&self) -> String {
        match self {
            Quality::Medium => "medium",
            Quality::High => "high",
            Quality::VeryHigh => "very_high",
            Quality::Ultra => "ultra",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Container {
    MP4,
    MKV,
    FLV,
    WEBM,
}

impl ToString for Container {
    fn to_string(&self) -> String {
        match self {
            Container::MP4 => "mp4",
            Container::MKV => "mkv",
            Container::FLV => "flv",
            Container::WEBM => "webm",
        }
        .to_string()
    }
}

fn default_bool_true() -> bool {
    true
}
