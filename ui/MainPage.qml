pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Dialogs
import org.kde.kirigami as Kirigami
import Settings
import "components"

Kirigami.ScrollablePage {
    id: mainPage
    title: i18n("General")
    property var settingsWindow

    function markDirty() {
        if (settingsWindow) {
            settingsWindow.markDirty();
        }
    }

    function loadChanges() {
        path.text = Settings.real_directory;
        separateDirsRadio.checked = Settings.file_name_pattern == "%app%/%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
        rootDirRadio.checked = Settings.file_name_pattern == "%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
        customDirRadio.checked = !separateDirsRadio.checked && !rootDirRadio.checked;
        customDirField.text = Settings.file_name_pattern;
        duration.value = Settings.duration;
        container.currentIndex = Settings.container;
        clearBuffer.checked = Settings.clear_buffer;
        recordReplays.checked = Settings.record_replays;
        useSteamGameNames.checked = Settings.use_steam_game_names;
        detectOnlyFullscreenApps.checked = Settings.detect_only_fullscreen_apps;
    }

    function saveChanges() {
        if (separateDirsRadio.checked) {
            Settings.file_name_pattern = "%app%/%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
        } else if (rootDirRadio.checked) {
            Settings.file_name_pattern = "%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
        } else if (customDirRadio.checked) {
            Settings.file_name_pattern = customDirField.text;
        }
        if (!isFlatpak) {
            Settings.directory = path.text;
            Settings.update_real_path();
        }
        Settings.duration = duration.value;
        Settings.container = container.currentIndex;
        Settings.clear_buffer = clearBuffer.checked;
        Settings.record_replays = recordReplays.checked;
        Settings.use_steam_game_names = useSteamGameNames.checked;
        Settings.detect_only_fullscreen_apps = detectOnlyFullscreenApps.checked;
    }

    actions: [
        Kirigami.Action {
            id: recordReplays
            text: i18n("Record replays")
            checkable: true

            displayComponent: Controls.Switch {
                action: recordReplays
            }

            onTriggered: mainPage.markDirty()
        }
    ]

    Kirigami.FormLayout {
        width: parent.width
        height: parent.height

        RowLayout {
            Layout.fillWidth: true
            Kirigami.FormData.label: i18n("Directory:")

            Controls.TextField {
                id: path
                Layout.fillWidth: true
                readOnly: isFlatpak
                onTextEdited: mainPage.markDirty()

                Controls.ToolTip.visible: isFlatpak && (hovered || activeFocus)
                Controls.ToolTip.text: i18n("Manual path editing is not supported under Flatpak - please use the file picker")
            }

            Controls.Button {
                icon.name: "system-file-manager-symbolic"
                onClicked: function () {
                    pathChooser.open();
                }
            }

            Dialogs.FolderDialog {
                id: pathChooser
                title: i18n("Choose replay directory")
                currentFolder: "file://" + path.text
                onAccepted: {
                    Settings.directory = selectedFolder.toString().replace("file://", "");
                    Settings.update_real_path();
                    path.text = Settings.real_directory;
                    mainPage.markDirty();
                }
            }
        }

        ColumnLayout {
            Kirigami.FormData.label: i18n("Save videos:")
            Kirigami.FormData.labelAlignment: Qt.AlignTop

            ConfigRadio {
                id: separateDirsRadio
                Layout.fillWidth: true
                text: i18n("In directories named after the current full-screen app")
                onClicked: mainPage.markDirty()
            }

            ConfigRadio {
                id: rootDirRadio
                Layout.fillWidth: true
                text: i18n("Directly in the directory selected above")
                onClicked: mainPage.markDirty()
            }

            ConfigRadio {
                id: customDirRadio
                Layout.fillWidth: true
                text: i18n("Using custom pattern: ")
                onClicked: mainPage.markDirty()
            }

            RowLayout {
                // visible: customDirRadio.checked

                Controls.TextField {
                    id: customDirField
                    Layout.fillWidth: true
                    enabled: customDirRadio.checked
                    onTextEdited: mainPage.markDirty()
                }

                Controls.ToolButton {
                    icon.name: "info"
                    enabled: customDirRadio.checked

                    Controls.ToolTip.visible: hovered
                    Controls.ToolTip.text: i18n(
                        "Available variables:\n" +
                        "%app% - title of the current full-screen window or unknown\n" +
                        "%year% - current year\n" +
                        "%month% - current month\n" +
                        "%day% - current day\n" +
                        "%hour% - current hour\n" +
                        "%minute% - current minute\n" +
                        "%second% - current second"
                    )
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Kirigami.FormData.label: i18n("Duration:")

            Controls.SpinBox {
                id: duration
                Layout.fillWidth: true
                from: 1
                to: 10000
                stepSize: 30
                onValueModified: mainPage.markDirty()
            }

            Controls.Label {
                text: i18n("secs")
            }
        }

        Controls.ComboBox {
            id: container
            Kirigami.FormData.label: i18n("Container:")
            Layout.fillWidth: true
            model: ["MKV", "MP4", "WEBM", "FLV"]
            onActivated: mainPage.markDirty()
        }

        Controls.Switch {
            id: clearBuffer
            Kirigami.FormData.label: i18n("Clear buffer when saving:")
            onToggled: mainPage.markDirty()
        }

        Controls.Switch {
            id: detectOnlyFullscreenApps
            Kirigami.FormData.label: i18n("Detect only fullscreen apps:")
            onToggled: mainPage.markDirty()
        }

        RowLayout {
            Kirigami.FormData.label: i18n("Use Steam game names:")

            Controls.Switch {
                id: useSteamGameNames
                onToggled: mainPage.markDirty()
            }

            Controls.ToolButton {
                icon.name: "info"
                Layout.preferredWidth: 18
                Layout.preferredHeight: 18

                Controls.ToolTip.visible: hovered
                Controls.ToolTip.text: i18n("Uses app name from Steam instead of window title for file " +
                    "naming when recorded app is launched through Steam.")
            }
        }

    }

    Component.onCompleted: loadChanges()
}
