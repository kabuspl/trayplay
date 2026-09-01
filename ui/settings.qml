import QtQuick
import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.formcard as FormCard
import Qt.labs.platform
import TrayHelper

QtObject {
    property var settingsWindow: Kirigami.ApplicationWindow {
        id: window
        objectName: "window"
        title: i18n("TrayPlay Settings")
        width: 500
        minimumWidth: 500
        height: 620
        visible: false

        pageStack.defaultColumnWidth: 500
        pageStack.initialPage: Qt.resolvedUrl("MainPage.qml")

        onClosing: close => {
            close.accepted = false;
            window.visible = false;
        }
    }

    property var messageBoxWindow: MessageBox {
        id: messageBox
        objectName: "messageBox"
    }

    property var aboutWindow: Kirigami.ApplicationWindow {
        id: aboutWindow
        objectName: "aboutWindow"
        title: i18n("About TrayPlay")
        width: 500
        minimumWidth: 500
        height: 580
        visible: false

        pageStack.defaultColumnWidth: 500
        pageStack.initialPage: FormCard.AboutPage {}

        onClosing: close => {
            close.accepted = false;
            aboutWindow.visible = false;
        }
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
                    window.show();
                    window.raise();
                    window.requestActivate();
                }
            }
            MenuItem {
                text: i18n("About")
                icon.name: "help-about"
                onTriggered: {
                    aboutWindow.show();
                    aboutWindow.raise();
                    aboutWindow.requestActivate();
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
