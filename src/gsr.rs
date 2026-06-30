use std::{fmt::Display, path::PathBuf, process::Stdio, str::FromStr, sync::Arc};

use log::debug;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{RwLock, mpsc::Sender, oneshot},
    task::JoinHandle,
};

use crate::{ActionEvent, config::Config, utils::process_pattern};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    NixErrno(nix::errno::Errno),
    RecorderNotRunning,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gpu-screen-recorder handler error: {:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(value: nix::errno::Errno) -> Self {
        Self::NixErrno(value)
    }
}

pub struct GpuScreenRecorder {
    pid: Option<u32>,
    config: Arc<RwLock<Config>>,
    app_name: Arc<RwLock<String>>,
    stdout_task_handle: Option<JoinHandle<()>>,
    stderr_task_handle: Option<JoinHandle<()>>,
    watcher_task_handle: Option<JoinHandle<()>>,
    watcher_cancel_tx: Option<oneshot::Sender<()>>,
    action_tx: Sender<ActionEvent>,
}

impl GpuScreenRecorder {
    pub async fn new(
        config: Arc<RwLock<Config>>,
        app_name: Arc<RwLock<String>>,
        action_tx: Sender<ActionEvent>,
    ) -> Result<Self, Error> {
        Ok(Self {
            pid: None,
            config,
            app_name,
            stderr_task_handle: None,
            stdout_task_handle: None,
            watcher_task_handle: None,
            watcher_cancel_tx: None,
            action_tx,
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        let config = self.config.read().await;

        let mut process = Command::new("gpu-screen-recorder")
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
            .arg("-k")
            .arg(&config.codec.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = process.stderr.take().unwrap();
        self.stderr_task_handle = Some(tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                debug!(target: "gpu-screen-recorder stderr", "{}", line);
            }
        }));

        let stdout = process.stdout.take().unwrap();
        let app_name_clone = self.app_name.clone();
        let config_clone = self.config.clone();
        self.stdout_task_handle = Some(tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let config = config_clone.read().await;

                let path = PathBuf::from_str(&line)
                    .expect("gpu-screen-recorder stdout must only contain file paths");

                let app_name = app_name_clone.read().await.clone();

                let target_path = format!(
                    "{}/{}.{}",
                    config.replay_directory.display(),
                    process_pattern(&config.file_name_pattern, &app_name),
                    config.container.to_string()
                );

                let target_path = target_path.split('/').collect::<Vec<&str>>();
                let mut target_path_dir = target_path.clone();
                target_path_dir.pop(); // Remove file name

                std::fs::create_dir_all(target_path_dir.join("/"))
                    .expect("failed to create directories");

                std::fs::rename(path, target_path.join("/")).expect("failed to move replay");
            }
        }));

        let pid = process.id().expect("process should have a PID after spawn");

        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        let action_tx = self.action_tx.clone();
        self.watcher_task_handle = Some(tokio::spawn(async move {
            tokio::select! {
                biased;
                _ = cancel_rx => {}
                result = process.wait() => {
                    if let Ok(status) = result {
                        if !status.success() {
                            let _ = action_tx.send(ActionEvent::GsrCrashed).await;
                        }
                    }
                }
            }
        }));

        self.watcher_cancel_tx = Some(cancel_tx);
        self.pid = Some(pid);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        if let Some(pid) = self.pid.take() {
            if let Some(cancel_tx) = self.watcher_cancel_tx.take() {
                let _ = cancel_tx.send(());
            }

            match signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(()) | Err(nix::errno::Errno::ESRCH) => Ok(()),
                Err(e) => Err(Error::NixErrno(e)),
            }
        } else {
            Err(Error::RecorderNotRunning)
        }
    }

    pub async fn save_replay(&mut self) -> Result<(), Error> {
        if let Some(pid) = self.pid {
            signal::kill(Pid::from_raw(pid as i32), Signal::SIGUSR1)?;
            Ok(())
        } else {
            Err(Error::RecorderNotRunning)
        }
    }

    pub fn is_running(&self) -> bool {
        self.pid.is_some()
    }
}
