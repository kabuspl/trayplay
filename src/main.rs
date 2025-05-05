use std::{
    error::Error,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    sync::Arc,
};

use ashpd::desktop::{file_chooser::OpenFileRequest, registry::Registry};
use config::Config;
use kdialog::{InputBox, MessageBox};
use ksni::TrayMethods;
use kwin::KWinScriptManager;
use log::{debug, error, info};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::sync::{RwLock, mpsc};
use tray::TrayIcon;
use zbus::{Connection, names::BusName, proxy};

mod active_window;
mod config;
mod kdialog;
mod kwin;
mod shortcuts;
mod tray;
mod utils;

#[derive(Debug)]
pub enum ActionEvent {
    SaveReplay,
    Quit,
    Config(ConfigActionEvent),
    Unknown,
}

#[derive(Debug)]
pub enum ConfigActionEvent {
    ChangeReplayPath,
    SetFramerate(u16),
    CustomFramerate,
    SetReplayDuration(u16),
    CustomReplayDuration,
}

#[proxy(
    interface = "org.kde.osdService",
    default_service = "org.kde.plasmashell",
    default_path = "/org/kde/osdService"
)]
trait OsdService {
    #[zbus(name = "showText")]
    fn show_text(&self, icon: &str, text: &str) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut config = Config::load();

    let connection = Connection::session().await?;
    let service_name = "ovh.kabus.instantreplay";
    let proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    let exists = proxy
        .name_has_owner(BusName::try_from(service_name)?)
        .await?;

    if exists {
        error!("Cannot start more than one instance of Instant Replay!");
        MessageBox::new("Cannot start more than one instance of Instant Replay!")
            .title("Error")
            .show()?;
        std::process::exit(1);
    }

    let kwin_script_manager = KWinScriptManager::new().await?;
    kwin_script_manager.load().await;

    // Let xdg portal know what desktop file are we
    Registry::default()
        .register("ovh.kabus.instantreplay")
        .await?;

    let (action_tx, mut action_rx) = mpsc::channel(8);
    let tray = TrayIcon::new(action_tx.clone());
    let _tray_handle = tray.spawn().await.unwrap();
    shortcuts::setup_global_shortcuts(action_tx);

    let app_name = Arc::new(RwLock::new("unknown".to_string()));
    active_window::setup_active_window_manager(app_name.clone()).await?;

    let mut gpu_screen_recorder = Command::new("gpu-screen-recorder")
        .arg("-w")
        .arg(&config.screen)
        .arg("-c")
        .arg(config.container.to_string())
        .arg("-f")
        .arg(config.framerate.to_string())
        .arg("-r")
        .arg(config.replay_duration_secs.to_string())
        .arg("-restart-replay-on-save")
        .arg(if config.clear_buffer_on_save {
            "yes"
        } else {
            "no"
        })
        .arg("-bm")
        .arg("qp")
        .arg("-q")
        .arg(config.quality.to_string())
        .args(config.audio_tracks.iter().flat_map(|track| ["-a", track]))
        .arg("-o")
        .arg(&config.replay_directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stderr = gpu_screen_recorder.stderr.take().unwrap();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines().filter_map(|line| line.ok()) {
            debug!(target: "gpu-screen-recorder stderr", "{}", line);
        }
    });

    let stdout = gpu_screen_recorder.stdout.take().unwrap();

    let app_name_clone = app_name.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        for line in reader.lines().filter_map(|line| line.ok()) {
            let path = PathBuf::from_str(&line)
                .expect("gpu-screen-recorder stdout must only contain file paths");

            let mut target_path = PathBuf::from_str("/media/HDD2/Powtórki/").unwrap();
            target_path.push(app_name_clone.read().await.clone());
            if !std::fs::exists(&target_path).unwrap() {
                std::fs::create_dir(&target_path).unwrap()
            }
            target_path.push(
                path.file_name()
                    .map(|e| e.to_str().unwrap().to_string())
                    .unwrap()
                    .replace(
                        "Replay",
                        &(app_name_clone.read().await.to_string() + "_replay"),
                    ),
            );

            std::fs::rename(path, target_path).expect("failed to move replay");
        }
    });

    let conn = Connection::session().await?;

    loop {
        if let Some(action) = action_rx.recv().await {
            match action {
                ActionEvent::SaveReplay => {
                    info!("Saving replay from {}", app_name.read().await);
                    signal::kill(
                        Pid::from_raw(gpu_screen_recorder.id() as i32),
                        Signal::SIGUSR1,
                    )?;
                    OsdServiceProxy::new(&conn)
                        .await?
                        .show_text(
                            "media-record",
                            &format!("Replay from \"{}\" saved!", app_name.read().await),
                        )
                        .await?;
                }
                ActionEvent::Quit => {
                    kwin_script_manager.unload().await;
                    signal::kill(
                        Pid::from_raw(gpu_screen_recorder.id() as i32),
                        Signal::SIGTERM,
                    )?;
                    std::process::exit(0);
                }
                ActionEvent::Config(config_event) => match config_event {
                    ConfigActionEvent::ChangeReplayPath => {
                        let request = OpenFileRequest::default()
                            .directory(true)
                            .current_folder(&config.replay_directory)?
                            .send()
                            .await
                            .and_then(|r| r.response());

                        match request {
                            Ok(directory) => {
                                let directory = directory.uris()[0].to_file_path().unwrap();
                                DialogBuilder::message()
                                    .set_text(directory.display().to_string())
                                    .alert()
                                    .show()
                                    .unwrap();
                            }
                            Err(_) => todo!(),
                        }
                    }
                    ConfigActionEvent::CustomReplayDuration => {
                        let result = InputBox::new(
                            "Replay duration (in seconds):",
                            kdialog::InputBoxType::Text,
                        )
                        .initial(config.replay_duration_secs.to_string())
                        .title("Instant Replay Settings")
                        .show()?;

                        if let Some(result) = result {
                            let number = result.replace("\n", "").parse::<u16>();
                            if let Ok(number) = number {
                                config.replay_duration_secs = number;
                                config.save();
                            } else {
                                MessageBox::new("You need to input an integer.")
                                    .title("Error")
                                    .show()?;
                            }
                        }
                    }
                    ConfigActionEvent::SetReplayDuration(duration) => {
                        config.replay_duration_secs = duration;
                        config.save();
                    }
                    ConfigActionEvent::CustomFramerate => {
                        let result = InputBox::new(
                            "Framerate (in frames per seconds):",
                            kdialog::InputBoxType::Text,
                        )
                        .initial(config.framerate.to_string())
                        .title("Instant Replay Settings")
                        .show()?;

                        if let Some(result) = result {
                            let number = result.replace("\n", "").parse::<u16>();
                            if let Ok(number) = number {
                                config.framerate = number;
                                config.save();
                            } else {
                                MessageBox::new("You need to input an integer.")
                                    .title("Error")
                                    .show()?;
                            }
                        }
                    }
                    ConfigActionEvent::SetFramerate(framerate) => {
                        config.framerate = framerate;
                        config.save();
                    }
                },
                other => todo!("Unhandled action event: {:?}", other),
            }
        }
    }
}
