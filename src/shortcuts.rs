use futures_util::StreamExt;

use ashpd::desktop::{
    Session,
    global_shortcuts::{GlobalShortcuts, NewShortcut},
};
use lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;

use crate::ActionEvent;

lazy_static! {
    static ref SHORTCUTS: Vec<(&'static str, &'static str, &'static str)> = vec![
        // id, description, trigger
        ("save-replay", "Save replay", "ALT+F10"),
        ("toggle-replay", "Toggle replay", "ALT+SHIFT+F10"),
        ("quit", "Quit program", "ALT+SHIFT+F11")
    ];
}

pub struct GlobalShortcutManager<'a> {
    global_shortcuts_wrapper: GlobalShortcuts<'a>,
    global_shortcuts_session: Session<'a, GlobalShortcuts<'a>>,
    shortcut_tx: Sender<ActionEvent>,
}

impl<'a> GlobalShortcutManager<'a> {
    pub async fn new(shortcut_tx: Sender<ActionEvent>) -> Result<Self, GlobalShortcutManagerError> {
        let wrapper = GlobalShortcuts::new().await?;
        Ok(Self {
            global_shortcuts_session: wrapper.create_session().await?,
            global_shortcuts_wrapper: wrapper,
            shortcut_tx,
        })
    }

    pub async fn register_all(&self) -> Result<(), GlobalShortcutManagerError> {
        let request = self
            .global_shortcuts_wrapper
            .list_shortcuts(&self.global_shortcuts_session)
            .await?;

        let shortcut_ids = request
            .response()?
            .shortcuts()
            .iter()
            .map(|shortcut| shortcut.id().to_string())
            .collect::<Vec<String>>();

        let shortcuts: Vec<NewShortcut> = SHORTCUTS
            .iter()
            .filter(|s| !shortcut_ids.contains(&s.0.to_string()))
            .map(|s| NewShortcut::new(s.0, s.1).preferred_trigger(s.2))
            .collect();

        if !shortcuts.is_empty() {
            let request = self
                .global_shortcuts_wrapper
                .bind_shortcuts(&self.global_shortcuts_session, &shortcuts, None)
                .await;

            // Ignore missing field error for now - looks like a bug in ashpd or in KDE 6.4 beta
            if let Err(error) = &request {
                if let ashpd::Error::Zbus(zbus::Error::Variant(zbus::zvariant::Error::Message(
                    message,
                ))) = error
                {
                    if message != "missing field `shortcuts`" {
                        request?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn run_handler(&self) -> Result<(), GlobalShortcutManagerError> {
        loop {
            if let Ok(mut activated) = self.global_shortcuts_wrapper.receive_activated().await {
                while let Some(activation) = activated.next().await {
                    self.shortcut_tx
                        .send(match activation.shortcut_id() {
                            "save-replay" => ActionEvent::SaveReplay,
                            "quit" => ActionEvent::Quit,
                            "toggle-replay" => ActionEvent::ToggleReplay,
                            _ => ActionEvent::Unknown,
                        })
                        .await?;
                }
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum GlobalShortcutManagerError {
    AshpdError(ashpd::Error),
    MpscError(tokio::sync::mpsc::error::SendError<ActionEvent>),
}

impl From<ashpd::Error> for GlobalShortcutManagerError {
    fn from(value: ashpd::Error) -> Self {
        Self::AshpdError(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<ActionEvent>> for GlobalShortcutManagerError {
    fn from(value: tokio::sync::mpsc::error::SendError<ActionEvent>) -> Self {
        Self::MpscError(value)
    }
}

pub fn setup_global_shortcuts(shortcut_tx: Sender<ActionEvent>) {
    tokio::spawn(async move {
        let global_shortcuts_manager = GlobalShortcutManager::new(shortcut_tx)
            .await
            .expect("Cannot setup global shortcuts");

        global_shortcuts_manager
            .register_all()
            .await
            .expect("Cannot register global shortcuts");

        global_shortcuts_manager
            .run_handler()
            .await
            .expect("Cannot run shortcut handler");
    });
}
