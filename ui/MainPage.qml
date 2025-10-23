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
    title: "Settings"
    actions: [
        Kirigami.Action {
            id: recordReplays
            text: "Record replays"
            checkable: true

            displayComponent: Controls.Switch {
                action: recordReplays
            }

            Component.onCompleted: function () {
                recordReplays.checked = Settings.record_replays;
            }
        }
    ]

    GridLayout {
        width: parent.width
        height: parent.height
        columns: 2
        rowSpacing: Kirigami.Units.largeSpacing

        ConfigLabel {
            text: "Video source:"
        }

        Controls.ComboBox {
            id: video_source
            Layout.fillWidth: true
            model: [
                {
                    text: `Default (${Settings.video_sources[0].split("|")[0]})`,
                    value: "screen"
                },
                ...Settings.video_sources.map(e => {
                    var split = e.split("|");
                    if (split.length > 1) {
                        return {
                            text: `${split[0]} (${split[1]})`,
                            value: split[0]
                        };
                    } else if (split[0] == "portal") {
                        return {
                            text: "XDG Desktop Portal",
                            value: "portal"
                        };
                    } else {
                        return {
                            text: split[0],
                            value: split[0]
                        };
                    }
                })]
            textRole: "text"
            valueRole: "value"
            currentValue: Settings.video_source_choice
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

        Controls.Label {
            text: "Save videos:"
            Layout.alignment: Qt.AlignTop | Qt.AlignRight
            Layout.topMargin: 2
        }

        ColumnLayout {
            Controls.RadioButton {
                id: separateDirsRadio
                text: "In directories named after the current full-screen app"
                checked: Settings.file_name_pattern == "%app%/%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%"
            }

            Controls.RadioButton {
                id: rootDirRadio
                text: "Directly in the directory selected above"
                checked: Settings.file_name_pattern == "%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%"
            }

            Controls.RadioButton {
                id: customDirRadio
                text: "Using custom pattern"
            }

            RowLayout {
                visible: customDirRadio.checked

                Controls.TextField {
                    id: customDirField
                    text: Settings.file_name_pattern
                    Layout.fillWidth: true
                }

                Controls.ToolButton {
                    icon.name: "info"

                    Controls.ToolTip.visible: hovered
                    Controls.ToolTip.text: `Available variables:
%app% - title of the current full-screen window or unknown
%year% - current year
%month% - current month
%day% - current day
%hour% - current hour
%minute% - current minute
%second% - current second`
                }
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
            model: ["H.264", "H.265 (HEVC)", "H.265 (HEVC) HDR", "H.265 (HEVC) 10-bit", "AV1", "AV1 HDR", "AV1 10-bit", "VP8", "VP9"]
            currentIndex: Settings.codec
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

        Item {}

        Row {
            Controls.Switch {
                id: clearBuffer
                text: "Clear buffer when saving"
                checked: Settings.clear_buffer
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
                    if (separateDirsRadio.checked) {
                        Settings.file_name_pattern = "%app%/%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
                    } else if (rootDirRadio.checked) {
                        Settings.file_name_pattern = "%app%_replay_%year%-%month%-%day%_%hour%-%minute%-%second%";
                    } else if (customDirRadio.checked) {
                        Settings.file_name_pattern = customDirField.text;
                    }
                    Settings.framerate = framerate.value;
                    Settings.duration = duration.value;
                    Settings.quality = quality.currentIndex;
                    Settings.container = container.currentIndex;
                    Settings.codec = codec.currentIndex;
                    Settings.replay_directory = path.text;
                    Settings.clear_buffer = clearBuffer.checked;
                    Settings.record_replays = recordReplays.checked;
                    Settings.video_source_choice = video_source.currentValue;
                    Settings.apply_config();
                }
            }
        }
    }
}
