use tokio::sync::mpsc::Sender;

use crate::{ActionEvent, ConfigActionEvent};

pub struct TrayIcon {
    _enabled: bool,
    tray_event_tx: Sender<ActionEvent>,
    replay_duration: usize,
    framerate: usize,
}

impl TrayIcon {
    pub fn new(tray_event_tx: Sender<ActionEvent>) -> Self {
        Self {
            tray_event_tx,
            _enabled: true,
            replay_duration: 0,
            framerate: 0,
        }
    }
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
        // let sender_clone1 = self.tray_event_tx.clone();
        let tx_clone = self.tray_event_tx.clone();
        use ksni::menu::*;
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
                submenu: vec![
                    StandardItem {
                        label: "Replay path".into(),
                        icon_name: "inode-directory".into(),
                        activate: Box::new({
                            let tx_clone = tx_clone.clone();
                            move |_| {
                                futures::executor::block_on(async {
                                    tx_clone
                                        .send(ActionEvent::Config(
                                            ConfigActionEvent::ChangeReplayPath,
                                        ))
                                        .await
                                        .unwrap();
                                });
                            }
                        }),
                        ..Default::default()
                    }
                    .into(),
                    SubMenu {
                        label: "Framerate".into(),
                        icon_name: "speedometer".into(),
                        submenu: vec![
                            RadioGroup {
                                selected: self.framerate,
                                select: Box::new({
                                    let tx_clone = tx_clone.clone();
                                    move |this: &mut Self, selection| {
                                        this.framerate = selection;
                                        futures::executor::block_on(async {
                                            tx_clone
                                                .send(if selection == 2 {
                                                    ActionEvent::Config(
                                                        ConfigActionEvent::CustomFramerate,
                                                    )
                                                } else {
                                                    ActionEvent::Config(
                                                        ConfigActionEvent::SetFramerate(
                                                            match selection {
                                                                0 => 30,
                                                                1 => 60,
                                                                _ => 30,
                                                            },
                                                        ),
                                                    )
                                                })
                                                .await
                                                .unwrap();
                                        });
                                    }
                                }),
                                options: vec![
                                    RadioItem {
                                        label: "30".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "60".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "Custom...".into(),
                                        ..Default::default()
                                    },
                                ],
                            }
                            .into(),
                        ],
                        ..Default::default()
                    }
                    .into(),
                    SubMenu {
                        label: "Replay duration".into(),
                        icon_name: "clock".into(),
                        submenu: vec![
                            RadioGroup {
                                selected: self.replay_duration,
                                select: Box::new({
                                    let tx_clone = tx_clone.clone();
                                    move |this: &mut Self, selection| {
                                        this.replay_duration = selection;
                                        futures::executor::block_on(async {
                                            tx_clone
                                                .send(if selection == 4 {
                                                    ActionEvent::Config(
                                                        ConfigActionEvent::CustomReplayDuration,
                                                    )
                                                } else {
                                                    ActionEvent::Config(
                                                        ConfigActionEvent::SetReplayDuration(
                                                            match selection {
                                                                0 => 30,
                                                                1 => 60,
                                                                2 => 180,
                                                                3 => 300,
                                                                _ => 0,
                                                            },
                                                        ),
                                                    )
                                                })
                                                .await
                                                .unwrap();
                                        });
                                    }
                                }),
                                options: vec![
                                    RadioItem {
                                        label: "30s".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "1min".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "3min".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "5min".into(),
                                        ..Default::default()
                                    },
                                    RadioItem {
                                        label: "Custom...".into(),
                                        ..Default::default()
                                    },
                                ],
                            }
                            .into(),
                        ],
                        ..Default::default()
                    }
                    .into(),
                ],
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
