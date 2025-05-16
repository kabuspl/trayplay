use log::info;
use zbus::{Connection, proxy};

use crate::utils::get_script_path;

#[proxy(
    interface = "org.kde.kwin.Scripting",
    default_service = "org.kde.KWin",
    default_path = "/Scripting"
)]
trait KWinScripting {
    #[zbus(name = "loadScript")]
    fn load_script(&self, file_path: &str, plugin_name: &str) -> zbus::Result<i32>;

    #[zbus(name = "unloadScript")]
    fn unload_script(&self, plugin_name: &str) -> zbus::Result<bool>;

    #[zbus(name = "start")]
    fn start(&self) -> zbus::Result<()>;
}

pub struct KWinScriptManager<'a> {
    _dbus_connection: Connection,
    kwin_scripting_proxy: KWinScriptingProxy<'a>,
}

impl<'a> KWinScriptManager<'a> {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let dbus_connection = zbus::connection::Connection::session().await?;

        Ok(Self {
            kwin_scripting_proxy: KWinScriptingProxy::new(&dbus_connection).await?,
            _dbus_connection: dbus_connection,
        })
    }

    pub async fn load(&self) {
        info!("Loading KWin script");

        self.kwin_scripting_proxy
            .load_script(
                get_script_path()
                    .expect("Cannot find KWin script")
                    .to_str()
                    .unwrap(),
                "trayplay",
            )
            .await
            .expect("Failed to load KWin script");

        self.kwin_scripting_proxy
            .start()
            .await
            .expect("Failed to start KWin script");
    }

    pub async fn unload(&self) {
        info!("Unloading KWin script");

        self.kwin_scripting_proxy
            .unload_script("trayplay")
            .await
            .expect("Failed to unload KWin script");
    }
}
