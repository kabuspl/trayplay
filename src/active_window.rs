use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use log::info;
use tokio::{
    fs,
    sync::{RwLock, mpsc},
};
use zbus::interface;

use crate::utils;

struct ActiveWindowManager {
    tx: mpsc::Sender<(String, String, bool, i32)>,
}

#[interface(name = "ovh.kabus.trayplay.ActiveWindowManager")]
impl ActiveWindowManager {
    async fn set_active_window(&self, desktop_file: &str, title: &str, fullscreen: bool, pid: i32) {
        self.tx
            .send((desktop_file.to_string(), title.to_string(), fullscreen, pid))
            .await
            .unwrap();
    }
}

pub async fn setup_active_window_manager(
    app_name: Arc<RwLock<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (app_name_tx, mut app_name_rx) = mpsc::channel(8);

    let active_window_manager = ActiveWindowManager { tx: app_name_tx };

    let _conn = zbus::connection::Builder::session()?
        .name("ovh.kabus.trayplay")?
        .serve_at("/ovh/kabus/trayplay", active_window_manager)?
        .build()
        .await?;

    tokio::spawn(async move {
        // Move connection inside tokio task so it doesn't get dropped immediately
        let _conn = _conn;

        loop {
            if let Some((desktop_file, title, fullscreen, pid)) = app_name_rx.recv().await {
                if fullscreen {
                    let mut app_name_new =
                        utils::get_app_name(&desktop_file).unwrap().unwrap_or(title);
                    if app_name_new.len() > 100 {
                        // app name too long - let's find executable name
                        if let Ok(path) = fs::read_link(format!("/proc/{}/exe", pid)).await {
                            app_name_new = path.file_name().unwrap().display().to_string();
                            if app_name_new == "wine-preloader" {
                                // try to find wine exe name
                                if let Ok(cmdline) =
                                    fs::read_to_string(format!("/proc/{}/cmdline", pid)).await
                                {
                                    app_name_new = cmdline
                                        .split('\0')
                                        .next()
                                        .unwrap()
                                        .split('\\')
                                        .last()
                                        .unwrap()
                                        .replace(".exe", "");
                                    // TODO: Structure this part better
                                }
                            }
                        } else {
                            // process died?
                            app_name_new = "unknown".to_string();
                        }
                    }
                    info!("Current app is now {}", app_name_new);
                    *app_name.write().await = app_name_new;
                } else if *app_name.read().await != "unknown" {
                    info!("Current app is unknown");
                    *app_name.write().await = "unknown".to_string();
                }
            }
        }
    });

    Ok(())
}
