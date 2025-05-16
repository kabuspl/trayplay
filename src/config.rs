use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub screen: String,
    pub container: Container,
    pub audio_tracks: Vec<String>,
    pub framerate: i64,
    pub clear_buffer_on_save: bool,
    pub quality: Quality,
    pub replay_directory: PathBuf,
    pub replay_duration_secs: i64,
}

impl Config {
    pub fn load() -> Self {
        // println!("{:#?}", USER_CONFIGURABLE.iter());
        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        match std::fs::read_to_string(path) {
            Ok(config) => toml::from_str(&config).expect("Cannot parse config file"),
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) {
        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        std::fs::write(path, toml::to_string(&self).unwrap()).expect("Failed to write config file");
    }
}

impl Default for Config {
    fn default() -> Self {
        println!("Config missing or broken. Replacing with defaults");

        let mut path = dirs::config_dir().unwrap();
        path.push("trayplay.toml");

        let instance = Self {
            screen: "screen".to_string(),
            audio_tracks: vec!["default_output".to_string(), "default_input".to_string()],
            framerate: 60,
            clear_buffer_on_save: true,
            quality: Quality::Ultra,
            replay_directory: dirs::video_dir().unwrap(),
            container: Container::MKV,
            replay_duration_secs: 180,
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
