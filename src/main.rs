use std::{
    error::Error,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    sync::Arc,
};

use ashpd::desktop::registry::Registry;
use config::Config;
use ksni::TrayMethods;
use kwin::KWinScriptManager;
use log::{debug, info};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::sync::{RwLock, mpsc};
use tray::TrayIcon;
use zbus::{Connection, proxy};

mod active_window;
mod config;
mod kwin;
mod shortcuts;
mod tray;
mod utils;

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

    let config = Config::load();

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
        .arg(config.screen)
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
        .arg("-o")
        .arg(config.replay_directory)
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
            match action.as_str() {
                "save-replay" => {
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
                "quit" => {
                    kwin_script_manager.unload().await;
                    signal::kill(
                        Pid::from_raw(gpu_screen_recorder.id() as i32),
                        Signal::SIGTERM,
                    )?;
                    std::process::exit(0);
                }
                other => {
                    info!("Received unknown action event: {}", other);
                }
            }
        }
    }
}
