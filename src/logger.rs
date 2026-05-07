use log::{Level, Log};
use tokio::sync::mpsc::Sender;

use crate::ActionEvent;

pub struct CombinedLogger(pub Vec<Box<dyn Log>>);

impl Log for CombinedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.0
            .iter()
            .fold(false, |acc, v| acc || v.enabled(metadata))
    }

    fn log(&self, record: &log::Record) {
        self.0.iter().for_each(|logger| logger.log(record));
    }

    fn flush(&self) {
        self.0.iter().for_each(|logger| logger.flush());
    }
}

pub struct UiLogger {
    pub action_event_tx: Sender<ActionEvent>,
}

impl Log for UiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &log::Record) {
        if record.level() <= Level::Warn {
            match record.level() {
                log::Level::Error => {
                    let _ = self.action_event_tx.try_send(ActionEvent::ShowError(
                        format!(
                            "Error - {} - {}",
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ),
                        format!("{}", record.args()),
                    ));
                }
                log::Level::Warn => {
                    let _ = self.action_event_tx.try_send(ActionEvent::ShowInfo(
                        format!(
                            "Warning - {} - {}",
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ),
                        format!("{}", record.args()),
                    ));
                }
                other => {
                    let _ = self.action_event_tx.try_send(ActionEvent::ShowInfo(
                        format!(
                            "{} - {} - {}",
                            other.as_str(),
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ),
                        format!("{}", record.args()),
                    ));
                }
            }
        }
    }

    fn flush(&self) {}
}
