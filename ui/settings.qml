import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import QtQuick.Dialogs as Dialogs
import org.kde.kirigami as Kirigami
import Settings

Kirigami.ApplicationWindow {
    id: window
    title: "TrayPlay Settings"
    width: 400
    maximumWidth: 400
    minimumWidth: 400
    height: mainPage.height - 4
    Component.onCompleted: function () {
        window.minimumHeight = mainPage.height;
    }

    component ConfigLabel: Controls.Label {
        Layout.alignment: Qt.AlignVCenter | Qt.AlignRight
    }

    pageStack.initialPage: Kirigami.Page {
        id: mainPage
        title: "Settings"

        GridLayout {
            width: parent.width
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
                    window.pageStack.push(audioPage);
                }
            }

            Row {
                Layout.columnSpan: 2
                Layout.alignment: Qt.AlignRight
                Layout.topMargin: 20

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

    Component {
        id: audioPage
        Kirigami.ScrollablePage {
            title: "Audio Tracks"
            actions: [
                Kirigami.Action {
                    icon.name: "list-add"
                    text: "Add"
                    onTriggered: function () {
                        Settings.add_audio_track();
                    }
                }
            ]

            Kirigami.CardsListView {
                model: Settings.audio_tracks

                delegate: Kirigami.AbstractCard {
                    contentItem: Item {
                        implicitWidth: delegateLayout.implicitWidth
                        implicitHeight: delegateLayout.implicitHeight

                        RowLayout {
                            id: delegateLayout
                            anchors {
                                left: parent.left
                                top: parent.top
                                right: parent.right
                            }

                            Controls.Label {
                                font.pixelSize: 32
                                text: index + 1
                                leftPadding: 8
                                rightPadding: 8
                            }

                            ColumnLayout {
                                spacing: 0
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignTop

                                ListView {
                                    id: sourcesList
                                    property int trackIndex: index
                                    Layout.fillWidth: true
                                    Layout.alignment: Qt.AlignVCenter
                                    Layout.preferredHeight: count * 18
                                    model: Settings.audio_tracks[index]
                                    delegate: RowLayout {
                                        id: sourceDelegateLayout

                                        spacing: 0
                                        Controls.Label {
                                            Layout.alignment: Qt.AlignVCenter
                                            text: switch (modelData) {
                                            case "default_input":
                                                " - Default Microphone";
                                                break;
                                            case "default_output":
                                                " - System Sound";
                                                break;
                                            default:
                                                " - " + modelData;
                                                break;
                                            }
                                        }
                                        Controls.ToolButton {
                                            Layout.alignment: Qt.AlignVCenter
                                            Layout.preferredWidth: 16
                                            Layout.preferredHeight: 16
                                            icon.name: "remove"
                                            icon.height: 12

                                            onClicked: function () {
                                                Settings.remove_audio_source(sourcesList.trackIndex, index);
                                            }
                                        }
                                    }
                                }
                            }

                            GridLayout {
                                columns: 3
                                Layout.alignment: Qt.AlignTop

                                Controls.Button {
                                    icon.name: "arrow-up"
                                    enabled: index != 0
                                    onClicked: function () {
                                        Settings.move_audio_track(index, index - 1);
                                    }
                                }

                                Controls.Button {
                                    icon.name: "arrow-down"
                                    enabled: index + 1 != Settings.audio_tracks.length
                                    onClicked: function () {
                                        Settings.move_audio_track(index, index + 1);
                                    }
                                }

                                Controls.Button {
                                    icon.name: "delete"
                                    onClicked: function () {
                                        Settings.remove_audio_track(index);
                                    }
                                }

                                Controls.Button {
                                    Layout.columnSpan: 3
                                    Layout.fillWidth: true
                                    icon.name: "list-add"
                                    text: "Add source"
                                    onClicked: function () {
                                        addDialog.trackIndex = index;
                                        addDialog.open();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Kirigami.Dialog {
                id: addDialog
                property int trackIndex: 0
                title: "Add audio source"
                padding: Kirigami.Units.largeSpacing
                standardButtons: Kirigami.Dialog.NoButton
                customFooterActions: Kirigami.Action {
                    text: "Add"
                    icon.name: "list-add"
                    onTriggered: function () {
                        var track = "";
                        if (defaultMicrophoneRadio.checked) {
                            track = "default_input";
                        } else if (systemSoundRadio.checked) {
                            track = "default_output";
                        } else if (otherDeviceRadio.checked) {
                            track = otherDevice.currentText;
                        } else if (applicationRadio.checked) {
                            track = "app:" + application.currentText;
                        }
                        Settings.add_audio_source(addDialog.trackIndex, track);
                        addDialog.close();
                    }
                }

                ColumnLayout {
                    Controls.RadioButton {
                        id: defaultMicrophoneRadio
                        text: "Default microhpone"
                        checked: true
                    }

                    Controls.RadioButton {
                        id: systemSoundRadio
                        text: "System sound"
                    }
                }
            }
        }
    }
}
