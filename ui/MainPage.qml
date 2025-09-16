import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Dialogs
import org.kde.kirigami as Kirigami
import Settings
import "components"

Kirigami.ScrollablePage {
    id: mainPage
    title: "Settings"

    GridLayout {
        width: parent.width
        height: parent.height
        columns: 2

        ConfigLabel {
            text: "Framerate:"
        }

        RowLayout {
            Layout.fillWidth: true

            Controls.SpinBox {
                id: framerate
                Layout.fillWidth: true
                from: 1
                to: 1000
                stepSize: 5
                value: Settings.framerate
            }

            Controls.Label {
                text: "FPS"
            }
        }

        ConfigLabel {
            text: "Duration:"
        }

        RowLayout {
            Layout.fillWidth: true

            Controls.SpinBox {
                id: duration
                Layout.fillWidth: true
                from: 1
                to: 10000
                stepSize: 30
                value: Settings.duration
            }

            Controls.Label {
                text: "secs"
            }
        }

        ConfigLabel {
            text: "Quality:"
        }

        Controls.ComboBox {
            id: quality
            Layout.fillWidth: true
            model: ["Medium", "High", "Very high", "Ultra"]
            currentIndex: Settings.quality
        }

        ConfigLabel {
            text: "Container:"
        }

        Controls.ComboBox {
            id: container
            Layout.fillWidth: true
            model: ["MKV", "MP4", "WEBM", "FLV"]
            currentIndex: Settings.container
        }

        ConfigLabel {
            text: "Codec:"
        }

        Controls.ComboBox {
            id: codec
            Layout.fillWidth: true
            model: ["H264", "MP4", "WEBM", "FLV"]
            currentIndex: Settings.codec
        }

        ConfigLabel {
            text: "Directory:"
        }

        RowLayout {
            Layout.fillWidth: true

            Controls.TextField {
                id: path
                Layout.fillWidth: true
                text: Settings.directory
            }

            Controls.Button {
                icon.name: "system-file-manager-symbolic"
                onClicked: function () {
                    pathChooser.open();
                }
            }

            Dialogs.FolderDialog {
                id: pathChooser
                title: "Choose replay directory"
                currentFolder: "file://" + path.text
                onAccepted: path.text = selectedFolder.toString().replace("file://", "")
            }
        }

        Item {}

        Row {
            Controls.Switch {
                id: clearBuffer
                text: "Clear buffer when saving"
                checked: Settings.clear_buffer
            }
        }

        Item {}

        Row {
            Controls.Switch {
                id: recordReplays
                text: "Record replays"
                checked: Settings.record_replays
            }
        }

        Item {}

        Controls.Button {
            Layout.fillWidth: true
            text: "Edit audio tracks"
            icon.name: "view-media-track"
            onClicked: function () {
                window.pageStack.push(Qt.resolvedUrl("AudioPage.qml"));
            }
        }

        Item {
            Layout.columnSpan: 2
            Layout.fillHeight: true
        }

        Row {
            Layout.columnSpan: 2
            Layout.alignment: Qt.AlignRight

            Controls.Button {
                text: "Apply"
                onClicked: function () {
                    Settings.framerate = framerate.value;
                    Settings.duration = duration.value;
                    Settings.quality = quality.currentIndex;
                    Settings.container = container.currentIndex;
                    Settings.codec = codec.currentIndex;
                    Settings.replay_directory = path.text;
                    Settings.clear_buffer = clearBuffer.checked;
                    Settings.record_replays = recordReplays.checked;
                    Settings.apply_config();
                }
            }
        }
    }
}
