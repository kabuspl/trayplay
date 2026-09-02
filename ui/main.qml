import QtQuick
import Qt.labs.platform
import TrayHelper

QtObject {
    property var settingsWindow: SettingsWindow {
        id: window
        objectName: "window"
    }

    property var messageBoxWindow: MessageBox {
        id: messageBox
        objectName: "messageBox"
    }

    property var trayIcon: SystemTrayIcon {
        visible: true
        icon.name: "ovh.kabus.TrayPlay"
        tooltip: "TrayPlay"

        onActivated: {
            window.show();
            window.raise();
            window.requestActivate();
        }

        menu: Menu {
            onAboutToShow: {
                recordReplays.checked = TrayHelper.record_replays;
            }

            MenuItem {
                id: recordReplays
                text: i18n("Record replays")
                icon.name: "media-record"
                checked: TrayHelper.record_replays
                onTriggered: {
                    TrayHelper.record_replays = checked;
                }
                checkable: true
            }
            MenuItem {
                text: i18n("Save replay")
                icon.name: "document-save"
                onTriggered: TrayHelper.save_replay()
            }
            MenuItem {
                text: i18n("Open replay directory")
                icon.name: "inode-directory"
                onTriggered: TrayHelper.open_replay_directory()
            }
            MenuSeparator {}
            MenuItem {
                text: i18n("Settings")
                icon.name: "settings-configure"
                onTriggered: {
                    window.open("MainPage.qml");
                    window.show();
                    window.raise();
                    window.requestActivate();
                }
            }
            MenuItem {
                text: i18n("About")
                icon.name: "help-about"
                onTriggered: {
                    window.open("AboutPage.qml");
                    window.show();
                    window.raise();
                    window.requestActivate();
                }
            }
            MenuSeparator {}
            MenuItem {
                text: i18n("Quit")
                icon.name: "application-exit"
                onTriggered: TrayHelper.quit()
            }
        }
    }
}
