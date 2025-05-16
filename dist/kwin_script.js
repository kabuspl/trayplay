function send(window) {
    callDBus(
        "ovh.kabus.trayplay",
        "/ovh/kabus/trayplay",
        "ovh.kabus.trayplay.ActiveWindowManager",
        "SetActiveWindow",
        window.desktopFileName,
        window.caption,
        window.fullScreen,
    );
}

workspace.windowActivated.connect(send);
