use std::{error::Error, sync::Arc};

use ashpd::desktop::registry::Registry;
use config::Config;
use gsr::GpuScreenRecorder;
use ksni::TrayMethods;
use kwin::KWinScriptManager;
use log::{error, info, warn};
use logger::{CombinedLogger, KDialogLogger};
use tokio::sync::{RwLock, mpsc};
use tray::TrayIcon;
use utils::ask_path;
use zbus::{Connection, names::BusName, proxy};

mod active_window;
mod config;
mod gsr;
mod kdialog;
mod kwin;
mod logger;
mod shortcuts;
mod tray;
mod utils;

#[derive(Debug)]
pub enum ActionEvent {
    SaveReplay,
    Quit,
    Unknown,
    ChangeReplayPath,
    ConfigSaved,
    ToggleReplay,
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
    let env_logger = env_logger::builder()
        .parse_env(env_logger::Env::default().default_filter_or("warn"))
        .build();
    let kdialog_logger = KDialogLogger {};

    log::set_max_level(env_logger.filter());
    log::set_boxed_logger(Box::new(CombinedLogger(vec![
        Box::new(env_logger),
        Box::new(kdialog_logger),
    ])))?;

    let (action_tx, mut action_rx) = mpsc::channel(8);

    let config = Arc::new(RwLock::new(Config::load(action_tx.clone()).await));

    let connection = Connection::session().await?;
    let service_name = "ovh.kabus.trayplay";
    let proxy = zbus::fdo::DBusProxy::new(&connection).await?;
    let exists = proxy
        .name_has_owner(BusName::try_from(service_name)?)
        .await?;

    if exists {
        error!("Cannot start more than one instance of TrayPlay!");
        std::process::exit(1);
    }

    let kwin_script_manager = KWinScriptManager::new().await?;
    kwin_script_manager.load().await;

    // Let xdg portal know what desktop file are we
    Registry::default().register("ovh.kabus.trayplay").await?;

    let tray = TrayIcon::new(action_tx.clone(), &config).await;
    // let tray = TrayIconClean::new(action_tx.clone(), &config);
    let tray_handle = tray.spawn().await.unwrap();
    shortcuts::setup_global_shortcuts(action_tx);

    let app_name = Arc::new(RwLock::new("unknown".to_string()));
    active_window::setup_active_window_manager(app_name.clone()).await?;

    let mut gpu_screen_recorder = GpuScreenRecorder::new(config.clone(), app_name.clone()).await?;
    if config.read().await.recording_enabled {
        handle_gsr_start_result(gpu_screen_recorder.start().await);
    }

    let conn = Connection::session().await?;
    let osd_service = OsdServiceProxy::new(&conn).await?;

    loop {
        if let Some(action) = action_rx.recv().await {
            match action {
                ActionEvent::SaveReplay => {
                    info!("Saving replay from {}", app_name.read().await);
                    match gpu_screen_recorder.save_replay().await {
                        Ok(_) => {
                            osd_service
                                .show_text(
                                    "media-record",
                                    &format!("Replay from \"{}\" saved!", app_name.read().await),
                                )
                                .await?;
                        }
                        Err(err) => match err {
                            gsr::Error::RecorderNotRunning => {
                                error!("Replay recording is either turned off or has crashed.")
                            }
                            err => {
                                error!("Failed to save replay: {}", err);
                            }
                        },
                    }
                }
                ActionEvent::Quit => {
                    kwin_script_manager.unload().await;
                    gpu_screen_recorder.stop().await?;
                    std::process::exit(0);
                }
                ActionEvent::ChangeReplayPath => {
                    let mut config = config.write().await;
                    match ask_path(true, &config.replay_directory).await {
                        Ok(directory) => {
                            if let Some(directory) = directory {
                                config.replay_directory = directory;
                                config.save().await;
                            }
                        }
                        Err(err) => {
                            error!("Error when asking for replay directory: {}", err);
                        }
                    };
                }
                ActionEvent::ConfigSaved => {
                    if gpu_screen_recorder.is_running() {
                        gpu_screen_recorder.stop().await?;
                        handle_gsr_start_result(gpu_screen_recorder.start().await);
                    }
                }
                ActionEvent::ToggleReplay => {
                    if gpu_screen_recorder.is_running() {
                        gpu_screen_recorder.stop().await?;
                        osd_service
                            .show_text("media-playback-stopped", "Replay recording stopped")
                            .await?;
                        let mut config = config.write().await;
                        config.recording_enabled = false;
                        config.save().await;
                    } else {
                        gpu_screen_recorder.start().await?;
                        osd_service
                            .show_text("media-playback-playing", "Replay recording started")
                            .await?;
                        let mut config = config.write().await;
                        config.recording_enabled = true;
                        config.save().await;
                    }
                    tray_handle.update(|_| {}).await;
                }
                other => {
                    warn!("Unhandled action event: {:?}", other)
                }
            }
        }
    }
}

fn handle_gsr_start_result(result: Result<(), gsr::Error>) {
    match result {
        Ok(gsr) => gsr,
        Err(err) => match err {
            gsr::Error::IoError(err) => match err.kind() {
                std::io::ErrorKind::NotFound => error!("gpu-screen-recorder is not installed!"),
                err => error!("Error while starting gpu-screen-recorder: {}", err),
            },
            err => error!("Error while starting gpu-screen-recorder: {}", err),
        },
    }
}
