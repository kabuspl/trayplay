use std::default;

use qmetaobject::{prelude::QObject, qt_base_class, qt_method};
use tokio::sync::mpsc::Sender;

pub enum MessageBoxResult {
    OK,
}

#[derive(QObject)]
pub struct MessageBoxHelper {
    base: qt_base_class!(trait QObject),
    result_tx: Option<Sender<MessageBoxResult>>,
    send_result: qt_method!(fn(&self, result: u8)),
}

impl MessageBoxHelper {
    pub fn new(result_tx: Sender<MessageBoxResult>) -> Self {
        Self {
            base: Default::default(),
            result_tx: Some(result_tx),
            send_result: Default::default(),
        }
    }

    fn send_result(&self, result: u8) {
        self.result_tx
            .as_ref()
            .unwrap()
            .try_send(match result {
                _ => MessageBoxResult::OK,
            })
            .unwrap();
    }
}

impl Default for MessageBoxHelper {
    fn default() -> Self {
        Self {
            base: Default::default(),
            result_tx: None,
            send_result: Default::default(),
        }
    }
}
