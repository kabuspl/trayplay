use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

use crate::ActionEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_bool_true")]
    pub recording_enabled: bool,
    pub screen: String,
    pub container: Container,
    #[serde(default = "Codec::default")]
    pub codec: Codec,
    pub audio_tracks: Vec<String>,
    pub framerate: u32,
    pub clear_buffer_on_save: bool,
    pub quality: Quality,
    pub replay_directory: PathBuf,
    pub replay_duration_secs: i64,
    #[serde(default = "default_file_name_pattern")]
    pub file_name_pattern: String,

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
            Err(err) => {
                println!("{}", err);
                Config::default()
            }
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
            codec: Codec::H264,
            replay_duration_secs: 180,
            file_name_pattern: default_file_name_pattern(),
            action_event_tx: None,
        };

        std::fs::write(path, toml::to_string(&instance).unwrap())
            .expect("Failed to write config file");

        instance
    }
}

#[repr(usize)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Default, IntEnum)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Medium,
    High,
    #[default]
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

#[repr(usize)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, IntEnum)]
#[serde(rename_all = "lowercase")]
pub enum Container {
    MKV,
    MP4,
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

#[repr(usize)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, IntEnum)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    HEVC,
    HEVCHDR,
    HEVC10Bit,
    AV1,
    AV1HDR,
    AV110Bit,
    VP8,
    VP9,
}

impl ToString for Codec {
    fn to_string(&self) -> String {
        match self {
            Codec::H264 => "h264",
            Codec::HEVC => "hevc",
            Codec::HEVCHDR => "hevc_hdr",
            Codec::HEVC10Bit => "hevc_10bit",
            Codec::AV1 => "av1",
            Codec::AV1HDR => "av1_hdr",
            Codec::AV110Bit => "av1_10bit",
            Codec::VP8 => "vp8",
            Codec::VP9 => "vp9",
        }
        .to_string()
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::H264
    }
}

fn default_bool_true() -> bool {
    true
}

fn default_file_name_pattern() -> String {
    "%app%/%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%".to_string()
}
