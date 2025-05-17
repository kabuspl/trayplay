use std::{
    fmt::Display,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    str::FromStr,
    sync::Arc,
};

use log::debug;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::config::Config;

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
    process: Option<Child>,
    config: Arc<RwLock<Config>>,
    app_name: Arc<RwLock<String>>,
    stdout_task_handle: Option<JoinHandle<()>>,
    stderr_task_handle: Option<JoinHandle<()>>,
}

impl GpuScreenRecorder {
    pub async fn new(
        config: Arc<RwLock<Config>>,
        app_name: Arc<RwLock<String>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            process: None,
            config,
            app_name,
            stderr_task_handle: None,
            stdout_task_handle: None,
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
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = process.stderr.take().unwrap();
        self.stderr_task_handle = Some(tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            for line in reader.lines().filter_map(|line| line.ok()) {
                debug!(target: "gpu-screen-recorder stderr", "{}", line);
            }
        }));

        let stdout = process.stdout.take().unwrap();
        let app_name_clone = self.app_name.clone();
        let config_clone = self.config.clone();
        self.stdout_task_handle = Some(tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            for line in reader.lines().filter_map(|line| line.ok()) {
                let path = PathBuf::from_str(&line)
                    .expect("gpu-screen-recorder stdout must only contain file paths");

                let mut target_path = config_clone.read().await.replay_directory.clone();
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
        }));

        self.process = Some(process);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        if let Some(process) = &self.process {
            signal::kill(Pid::from_raw(process.id() as i32), Signal::SIGTERM)?;

            Ok(())
        } else {
            Err(Error::RecorderNotRunning)
        }
    }

    pub async fn save_replay(&mut self) -> Result<(), Error> {
        // info!("Saving replay from {}", self.app_name.read().await);
        if let Some(process) = &self.process {
            signal::kill(Pid::from_raw(process.id() as i32), Signal::SIGUSR1)?;
            Ok(())
        } else {
            Err(Error::RecorderNotRunning)
        }
    }
}
