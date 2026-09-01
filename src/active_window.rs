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

use crate::{
    config::{self, Config},
    utils,
};

struct ActiveWindowManager {
    tx: mpsc::Sender<(String, String, bool, i32)>,
}

#[interface(name = "ovh.kabus.TrayPlay.ActiveWindowManager")]
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
    config: Arc<RwLock<Config>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (app_name_tx, mut app_name_rx) = mpsc::channel(8);

    let active_window_manager = ActiveWindowManager { tx: app_name_tx };

    let _conn = zbus::connection::Builder::session()?
        .name("ovh.kabus.TrayPlay")?
        .serve_at("/ovh/kabus/TrayPlay", active_window_manager)?
        .build()
        .await?;

    tokio::spawn(async move {
        // Move connection inside tokio task so it doesn't get dropped immediately
        let _conn = _conn;

        loop {
            if let Some((desktop_file, title, fullscreen, pid)) = app_name_rx.recv().await {
                if config.read().await.use_steam_game_names
                    && let Some(appid) = get_steam_appid(pid).await
                    && let Some(appmanifest_path) = get_steam_appmanifest_path(appid)
                    && let Some(steam_app_name) = get_steam_app_name(appmanifest_path).await
                {
                    info!("Current app is now {}", steam_app_name);
                    *app_name.write().await = steam_app_name;
                } else if fullscreen {
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

pub async fn get_steam_appid(pid: i32) -> Option<u32> {
    fs::read_to_string(format!("/proc/{}/environ", pid))
        .await
        .ok()
        .and_then(|environ| {
            environ
                .split('\0')
                .filter_map(|entry| entry.split_once('='))
                .find_map(|(key, value)| {
                    (key == "SteamAppId")
                        .then(|| value.parse::<u32>().ok())
                        .flatten()
                })
        })
}

pub fn get_steam_appmanifest_path(appid: u32) -> Option<PathBuf> {
    let libraryfolders = [
        dirs::data_dir()?.join("Steam/steamapps/libraryfolders.vdf"),
        dirs::home_dir()?.join(
            ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/libraryfolders.vdf",
        ),
    ]
    .into_iter()
    .find_map(|path| std::fs::read_to_string(path).ok())?;

    let vdf = keyvalues_parser::parse(&libraryfolders).ok()?;
    let root = vdf.value.get_obj()?;

    root.values().find_map(|values| {
        let folder = values.first()?.get_obj()?;
        let apps = folder.get("apps")?.first()?.get_obj()?;

        if !apps
            .keys()
            .any(|key| key.parse::<u32>().ok() == Some(appid))
        {
            return None;
        }

        let library_path = folder.get("path")?.first()?.get_str()?;

        Some(
            PathBuf::from(library_path)
                .join("steamapps")
                .join(format!("appmanifest_{appid}.acf")),
        )
    })
}

pub async fn get_steam_app_name(appmanifest_path: impl AsRef<Path>) -> Option<String> {
    let appmanifest = fs::read_to_string(appmanifest_path).await.ok()?;
    let vdf = keyvalues_parser::parse(&appmanifest).ok()?;
    let root = vdf.value.get_obj()?;

    root.get("name")?
        .first()?
        .get_str()
        .map(|val| val.to_string())
}
