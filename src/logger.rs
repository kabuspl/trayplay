use log::{Level, Log};

use crate::kdialog::{InfoBox, MessageBox};

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

pub struct KDialogLogger;

impl Log for KDialogLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &log::Record) {
        if record.level() <= Level::Warn {
            match record.level() {
                log::Level::Error => {
                    InfoBox::error(record.args().as_str().unwrap_or("No information available"))
                        .title(format!(
                            "Error - {} - {}",
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ))
                        .show()
                        .unwrap();
                }
                log::Level::Warn => {
                    InfoBox::warning(record.args().as_str().unwrap_or("No information available"))
                        .title(format!(
                            "Warning - {} - {}",
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ))
                        .show()
                        .unwrap();
                }
                other => {
                    MessageBox::new(record.args().as_str().unwrap_or("No information available"))
                        .title(format!(
                            "{} - {} - {}",
                            other.as_str(),
                            record.module_path().unwrap_or("unknown module"),
                            record.file().unwrap_or("unknown file")
                        ))
                        .show()
                        .unwrap();
                }
            }
        }
    }

    fn flush(&self) {}
}
