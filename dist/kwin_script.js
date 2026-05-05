var active_window = null;

function send(window) {
    if (window.active) {
        callDBus(
            "ovh.kabus.trayplay",
            "/ovh/kabus/trayplay",
            "ovh.kabus.trayplay.ActiveWindowManager",
            "SetActiveWindow",
            window.desktopFileName,
            window.caption,
            window.fullScreen,
            window.pid,
        );
        if (active_window != null) {
            active_window.fullScreenChanged.disconnect(fullScreenChanged);
        }
        active_window = window;
        active_window.fullScreenChanged.connect(fullScreenChanged);
    }
}

function fullScreenChanged() {
    callDBus(
        "ovh.kabus.trayplay",
        "/ovh/kabus/trayplay",
        "ovh.kabus.trayplay.ActiveWindowManager",
        "SetActiveWindow",
        active_window.desktopFileName,
        active_window.caption,
        active_window.fullScreen,
        active_window.pid,
    );
}

workspace.windowActivated.connect(send);
workspace.windowAdded.connect(send);
