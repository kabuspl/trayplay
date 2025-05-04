use tokio::sync::mpsc::Sender;

pub struct TrayIcon {
    _enabled: bool,
    tray_event_tx: Sender<String>,
}

impl TrayIcon {
    pub fn new(tray_event_tx: Sender<String>) -> Self {
        Self {
            tray_event_tx,
            _enabled: true,
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
        let sender_clone2 = self.tray_event_tx.clone();
        let sender_clone3 = self.tray_event_tx.clone();
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
                activate: Box::new(move |_| {
                    futures::executor::block_on(async {
                        sender_clone2.send("save-replay".into()).await.unwrap();
                    });
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "gtk-quit".into(),
                activate: Box::new(move |_| {
                    futures::executor::block_on(async {
                        sender_clone3.send("quit".into()).await.unwrap();
                    });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
