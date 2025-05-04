use futures_util::StreamExt;

use ashpd::desktop::{
    Session,
    global_shortcuts::{GlobalShortcuts, NewShortcut},
};
use lazy_static::lazy_static;
use log::info;
use tokio::sync::mpsc::Sender;

lazy_static! {
    static ref SHORTCUTS: Vec<(&'static str, &'static str, &'static str)> = vec![
        // id, description, trigger
        ("save-replay", "Save replay", "ALT+F10"),
        // ("toggle-replay", "Toggle replay", "ALT+SHIFT+F10"), // TODO: implement toggling replays on and off
        ("quit", "Quit program", "ALT+SHIFT+F11")
    ];
}

pub struct GlobalShortcutManager<'a> {
    global_shortcuts_wrapper: GlobalShortcuts<'a>,
    global_shortcuts_session: Session<'a, GlobalShortcuts<'a>>,
    shortcut_tx: Sender<String>,
}

impl<'a> GlobalShortcutManager<'a> {
    pub async fn new(shortcut_tx: Sender<String>) -> Result<Self, GlobalShortcutManagerError> {
        let wrapper = GlobalShortcuts::new().await?;
        Ok(Self {
            global_shortcuts_session: wrapper.create_session().await?,
            global_shortcuts_wrapper: wrapper,
            shortcut_tx,
        })
    }

    async fn register(
        &self,
        id: &str,
        description: &str,
        trigger: &str,
    ) -> Result<(), GlobalShortcutManagerError> {
        info!("Registering new shortcut with id {}", id);

        let shortcut = NewShortcut::new(id, description).preferred_trigger(trigger);
        let shortcuts = [shortcut];
        let request = self
            .global_shortcuts_wrapper
            .bind_shortcuts(&self.global_shortcuts_session, &shortcuts, None)
            .await?;
        request.response()?;

        Ok(())
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

        for shortcut in SHORTCUTS.iter() {
            if !shortcut_ids.contains(&shortcut.0.to_string()) {
                self.register(shortcut.0, shortcut.1, shortcut.2).await?;
            }
        }

        Ok(())
    }

    pub async fn run_handler(&self) -> Result<(), GlobalShortcutManagerError> {
        loop {
            if let Ok(mut activated) = self.global_shortcuts_wrapper.receive_activated().await {
                while let Some(activation) = activated.next().await {
                    self.shortcut_tx
                        .send(activation.shortcut_id().to_string())
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
    MpscError(tokio::sync::mpsc::error::SendError<String>),
}

impl From<ashpd::Error> for GlobalShortcutManagerError {
    fn from(value: ashpd::Error) -> Self {
        Self::AshpdError(value)
    }
}

impl From<tokio::sync::mpsc::error::SendError<String>> for GlobalShortcutManagerError {
    fn from(value: tokio::sync::mpsc::error::SendError<String>) -> Self {
        Self::MpscError(value)
    }
}

pub fn setup_global_shortcuts(shortcut_tx: Sender<String>) {
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
