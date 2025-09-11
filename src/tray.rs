use log::error;
use std::{iter::once, process::Command, sync::Arc};

use ksni::{
    MenuItem,
    menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem, SubMenu},
};
use tokio::sync::{RwLock, mpsc::Sender};

use crate::{ActionEvent, config::Config, kdialog::MessageBox};

pub struct TrayIcon {
    _enabled: bool,
    tray_event_tx: Sender<ActionEvent>,
    config: Arc<RwLock<Config>>,
}

impl TrayIcon {
    pub async fn new(tray_event_tx: Sender<ActionEvent>, config: &Arc<RwLock<Config>>) -> Self {
        Self {
            tray_event_tx,
            _enabled: true,
            config: config.clone(),
        }
    }
}

struct TrayMultipleOption<T>(String, T);

impl<T> Into<RadioItem> for &TrayMultipleOption<T> {
    fn into(self) -> RadioItem {
        RadioItem {
            label: self.0.clone(),
            ..Default::default()
        }
    }
}

#[allow(dead_code)]
enum TrayConfigItem<T, O>
where
    T: ksni::Tray,
{
    Multiple {
        label: String,
        icon: String,
        options: Vec<TrayMultipleOption<O>>,
        initial_state: usize,
        show_custom: bool,
        action: Box<dyn Fn(&mut T, usize) + Send + 'static>,
    },
    Toggle {
        label: String,
        icon: String,
        initial_state: bool,
        action: Box<dyn Fn(&mut T) + Send + 'static>,
    },
    Custom {
        label: String,
        icon: String,
        action: Box<dyn Fn(&mut T) + Send + 'static>,
    },
}

impl<T, O> Into<MenuItem<T>> for TrayConfigItem<T, O>
where
    T: ksni::Tray,
{
    fn into(self) -> MenuItem<T> {
        match self {
            TrayConfigItem::Multiple {
                label,
                icon,
                options,
                action,
                initial_state,
                show_custom,
            } => SubMenu {
                label,
                icon_name: icon,
                submenu: vec![
                    RadioGroup {
                        selected: initial_state,
                        select: action,
                        options: {
                            let options = options.iter().map(|option| option.into());

                            if show_custom {
                                options
                                    .chain(once(RadioItem {
                                        label: "Custom...".into(),
                                        ..Default::default()
                                    }))
                                    .collect()
                            } else {
                                options.collect()
                            }
                        },
                    }
                    .into(),
                ],
                ..Default::default()
            }
            .into(),
            TrayConfigItem::Toggle {
                label,
                icon,
                initial_state,
                action,
            } => CheckmarkItem {
                label,
                icon_name: icon,
                activate: action,
                checked: initial_state,
                ..Default::default()
            }
            .into(),
            TrayConfigItem::Custom {
                label,
                icon,
                action,
            } => StandardItem {
                label,
                icon_name: icon,
                activate: action,
                ..Default::default()
            }
            .into(),
        }
    }
}

macro_rules! tray_config_item_custom {
    ($label:expr, $icon:expr, $action:expr) => {
        TrayConfigItem::Custom::<TrayIcon, u8> {
            label: $label.into(),
            icon: $icon.into(),
            action: Box::new(|item| {
                futures::executor::block_on(async {
                    $action(item.config.clone(), item.tray_event_tx.clone()).await;
                });
            }),
        }
    };
}

macro_rules! tray_config_item_toggle {
    ($label:expr, $icon:expr, $initial_state:expr, $action:expr) => {
        TrayConfigItem::Toggle::<TrayIcon, u8> {
            label: $label.into(),
            icon: $icon.into(),
            initial_state: $initial_state,
            action: Box::new(|item| {
                futures::executor::block_on(async {
                    $action(item.config.clone(), item.tray_event_tx.clone()).await;
                });
            }),
        }
    };
}

impl ksni::Tray for TrayIcon {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn icon_name(&self) -> String {
        "media-skip-backward".into()
    }

    fn title(&self) -> String {
        "TrayPlay".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let tx_clone = self.tray_event_tx.clone();
        use ksni::menu::*;

        let config = futures::executor::block_on(async { self.config.read().await });

        vec![
            tray_config_item_toggle!("Record replays", "media-record", config.recording_enabled, async |_, action_event_tx: Sender<ActionEvent>| {
                action_event_tx.send(ActionEvent::ToggleReplay).await.unwrap();
            }).into(),
            StandardItem {
                label: "Save replay".into(),
                icon_name: "document-save".into(),
                activate: Box::new({
                    let tx_clone = tx_clone.clone();
                    move |_| {
                        futures::executor::block_on(async {
                            tx_clone.send(ActionEvent::SaveReplay).await.unwrap();
                        });
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Settings".into(),
                icon_name: "configure".into(),
                activate: Box::new({
                    let tx_clone = tx_clone.clone();
                    move |_| {
                        futures::executor::block_on(async {
                            tx_clone.send(ActionEvent::OpenSettings).await.unwrap();
                        });
                    }
                }),
                ..Default::default()
            }
            .into(),
            tray_config_item_custom!("About", "help-about", async move |_, _| {
                let gsr_version = Command::new("gpu-screen-recorder")
                    .arg("--version")
                    .output()
                    .unwrap();
                MessageBox::new(format!(
                    "TrayPlay version: {}\ngpu-screen-recorder version: {}\nReport issues at: https://github.com/kabuspl/trayplay/issues\nLicense: MIT\n© 2025 kabuspl",
                    env!("CARGO_PKG_VERSION"),
                    String::from_utf8(gsr_version.stdout).unwrap()
                ))
                .title("About TrayPlay")
                .show()
                .unwrap();
            })
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "gtk-quit".into(),
                activate: Box::new({
                    let tx_clone = tx_clone.clone();
                    move |_| {
                        futures::executor::block_on(async {
                            tx_clone.send(ActionEvent::Quit).await.unwrap();
                        });
                    }
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
