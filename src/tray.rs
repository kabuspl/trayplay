use log::error;
use std::{iter::once, sync::Arc};

use ksni::{
    MenuItem,
    menu::{RadioGroup, RadioItem, StandardItem, SubMenu},
};
use tokio::sync::{RwLock, mpsc::Sender};

use crate::{ActionEvent, config::Config, utils::ask_custom_number};

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

#[derive(Clone)]
enum TrayMultipleOption {
    String(String, String),
    Number(String, i64),
}

impl Into<RadioItem> for &TrayMultipleOption {
    fn into(self) -> RadioItem {
        RadioItem {
            label: match self {
                TrayMultipleOption::String(label, _) | TrayMultipleOption::Number(label, _) => {
                    label.to_string()
                }
            },
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct NotNumberOptionError;

impl TryInto<i64> for &TrayMultipleOption {
    type Error = NotNumberOptionError;

    fn try_into(self) -> Result<i64, Self::Error> {
        if let TrayMultipleOption::Number(_, number) = self {
            Ok(*number)
        } else {
            Err(NotNumberOptionError)
        }
    }
}
impl TryInto<i64> for TrayMultipleOption {
    type Error = NotNumberOptionError;

    fn try_into(self) -> Result<i64, Self::Error> {
        (&self).try_into()
    }
}

#[derive(Debug)]
struct NotStringOptionError;

impl TryInto<String> for &TrayMultipleOption {
    type Error = NotStringOptionError;

    fn try_into(self) -> Result<String, Self::Error> {
        if let TrayMultipleOption::String(_, string) = self {
            Ok(string.to_string())
        } else {
            Err(NotStringOptionError)
        }
    }
}
impl TryInto<String> for TrayMultipleOption {
    type Error = NotStringOptionError;

    fn try_into(self) -> Result<String, Self::Error> {
        (&self).try_into()
    }
}

enum TrayConfigItem<T>
where
    T: ksni::Tray + CommunicationProvider,
{
    Multiple {
        label: String,
        icon: String,
        options: Vec<TrayMultipleOption>,
        initial_state: usize,
        action: Box<dyn Fn(&mut T, usize) + Send + 'static>,
    },
    Toggle {
        label: String,
        icon: String,
        action: Box<dyn Fn(&mut T) + Send + 'static>,
    },
    Custom {
        label: String,
        icon: String,
        action: Box<dyn Fn(&mut T) + Send + 'static>,
    },
}

impl<T> Into<MenuItem<T>> for TrayConfigItem<T>
where
    T: ksni::Tray + CommunicationProvider,
{
    fn into(self) -> MenuItem<T> {
        match self {
            TrayConfigItem::Multiple {
                label,
                icon,
                options,
                action,
                initial_state,
            } => SubMenu {
                label,
                icon_name: icon,
                submenu: vec![
                    RadioGroup {
                        selected: initial_state,
                        select: action,
                        options: options
                            .iter()
                            .map(|option| option.into())
                            .chain(once(RadioItem {
                                label: "Custom...".into(),
                                ..Default::default()
                            }))
                            .collect(),
                    }
                    .into(),
                ],
                ..Default::default()
            }
            .into(),
            TrayConfigItem::Toggle {
                label,
                icon,
                action,
            } => todo!("Implement toggle config menu item type"),
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

macro_rules! tray_config_item_radio {
    ($config_key:ident, $config:expr, $label:expr, $icon:expr, $values:expr) => {{
        let config = $config;

        TrayConfigItem::Multiple::<TrayIcon> {
            label: $label.into(),
            icon: $icon.into(),
            options: $values,
            initial_state: $values
                .iter()
                .position(|element| {
                    let a: i64 = element.try_into().unwrap();
                    a == config.$config_key
                })
                .unwrap_or($values.len()),
            action: Box::new(|item, selection| {
                futures::executor::block_on(async {
                    let config = item.get_config();
                    let mut config = config.write().await;
                    if selection >= $values.len() {
                        match ask_custom_number("Instant Replay Settings", $label, 0) {
                            Ok(number) => {
                                if let Some(number) = number {
                                    config.$config_key = number;
                                    config.save();
                                }
                            }
                            Err(err) => {
                                error!("Error when asking for custom config value: {}", err);
                            }
                        }
                    } else {
                        config.$config_key = $values[selection].clone().try_into().unwrap();
                        config.save();
                    }
                });
            }),
        }
    }};
}

macro_rules! tray_config_item_custom {
    ($config_key:ident, $label:expr, $icon:expr, $action:expr) => {
        TrayConfigItem::Custom::<TrayIcon> {
            label: $label.into(),
            icon: $icon.into(),
            action: Box::new(|item| {
                futures::executor::block_on(async {
                    $action(item.get_config(), item.get_action_event_tx()).await;
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
        "Instant Replay".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let tx_clone = self.tray_event_tx.clone();
        use ksni::menu::*;

        let config = futures::executor::block_on(async { self.config.read().await });

        let settings_menu = vec![
            tray_config_item_radio!(
                framerate,
                &config,
                "Framerate",
                "speedometer",
                vec![
                    TrayMultipleOption::Number("30".into(), 30),
                    TrayMultipleOption::Number("60".into(), 60)
                ]
            )
            .into(),
            tray_config_item_radio!(
                replay_duration_secs,
                &config,
                "Duration",
                "clock",
                vec![
                    TrayMultipleOption::Number("30s".into(), 30),
                    TrayMultipleOption::Number("1min".into(), 60),
                    TrayMultipleOption::Number("2min".into(), 120),
                    TrayMultipleOption::Number("3min".into(), 180),
                    TrayMultipleOption::Number("5min".into(), 300),
                ]
            )
            .into(),
            tray_config_item_custom!(
                replay_directory,
                "Path",
                "inode-directory",
                async move |_, action_event_tx: Sender<ActionEvent>| {
                    // Need to send message to main thread because for some reason portal file picker request
                    // is not being sent when directly called here...
                    action_event_tx
                        .send(ActionEvent::ChangeReplayPath)
                        .await
                        .unwrap();
                }
            )
            .into(),
        ];

        vec![
            // TODO: implement toggling replays on and off
            // CheckmarkItem {
            //     label: "Record replays".into(),
            //     checked: self.enabled,
            //     icon_name: "media-skip-backward".into(),
            //     activate: Box::new(move |this: &mut Self| {
            //         this.enabled = !this.enabled;
            //         futures::executor::block_on(async {
            //             sender_clone1.send("toggle-replay".into()).await.unwrap();
            //         });
            //     }),
            //     ..Default::default()
            // }
            // .into(),
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
            SubMenu {
                label: "Settings".into(),
                icon_name: "configure".into(),
                submenu: settings_menu,
                ..Default::default()
            }
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

impl CommunicationProvider for TrayIcon {
    fn get_config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }

    fn get_action_event_tx(&self) -> Sender<ActionEvent> {
        self.tray_event_tx.clone()
    }
}

trait CommunicationProvider {
    fn get_config(&self) -> Arc<RwLock<Config>>;
    fn get_action_event_tx(&self) -> Sender<ActionEvent>;
}
