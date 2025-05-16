use std::sync::Arc;

use log::info;
use tokio::sync::{RwLock, mpsc};
use zbus::interface;

use crate::utils;

struct ActiveWindowManager {
    tx: mpsc::Sender<(String, String, bool)>,
}

#[interface(name = "ovh.kabus.trayplay.ActiveWindowManager")]
impl ActiveWindowManager {
    async fn set_active_window(&self, desktop_file: &str, title: &str, fullscreen: bool) {
        self.tx
            .send((desktop_file.to_string(), title.to_string(), fullscreen))
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
            if let Some((desktop_file, title, fullscreen)) = app_name_rx.recv().await {
                if fullscreen {
                    let app_name_new = utils::get_app_name(&desktop_file).unwrap().unwrap_or(title);
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
