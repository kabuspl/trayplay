function send(window) {
    callDBus(
        "ovh.kabus.instantreplay",
        "/ovh/kabus/instantreplay",
        "ovh.kabus.instantreplay.ActiveWindowManager",
        "SetActiveWindow",
        window.desktopFileName,
        window.caption,
        window.fullScreen,
    );
}

workspace.windowActivated.connect(send);
