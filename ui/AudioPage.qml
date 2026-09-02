import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Settings

ColumnLayout {
    property var settingsWindow
    property var flickable: advancedAudioTracks.visible ? advancedAudioTracks.contentItem : null
    spacing: 0

    Connections {
        target: Settings
        function onChange() {
            if (settingsWindow) {
                settingsWindow.markDirty();
            }
        }
    }

    Kirigami.NavigationTabBar {
        z: 1
        Layout.fillWidth: true
        actions: [
            Kirigami.Action {
                icon.name: "settings-configure"
                text: i18n("Simple")
                checked: simpleAudioTracks.visible
                onTriggered: function () {
                    simpleAudioTracks.visible = true;
                    advancedAudioTracks.visible = false;
                }
            },
            Kirigami.Action {
                icon.name: "code-class"
                text: i18n("Advanced")
                checked: advancedAudioTracks.visible
                onTriggered: function () {
                    simpleAudioTracks.visible = false;
                    advancedAudioTracks.visible = true;
                }
            }
        ]
    }

    Item {
        id: simpleAudioTracks
        visible: true
        Layout.fillWidth: true
        Layout.fillHeight: true
        onVisibleChanged: function () {
            checkSwitches();
        }

        function checkSwitches() {
            recordSystem.checked = Settings.audio_tracks.length > 0 && (Settings.audio_tracks[0].join("|") == "default_output" || Settings.audio_tracks[0].join("|") == "default_output|default_input");
            recordMicrophone.checked = Settings.audio_tracks.length > 0 && (Settings.audio_tracks[0].join("|") == "default_input" || (Settings.audio_tracks[1] && Settings.audio_tracks[1].join("|") == "default_input") || Settings.audio_tracks[0].join("|") == "default_output|default_input");
            mergeTracks.checked = Settings.audio_tracks.length == 1 && Settings.audio_tracks[0].join("|") == "default_output|default_input";
        }

        Column {
            id: simpleAudioTracksColumn
            function isSimpleConfigVisible() {
                if (Settings.audio_tracks.length == 0) {
                    return true;
                }

                if (Settings.audio_tracks.length == 1 && (Settings.audio_tracks[0].join("|") == "default_output" || Settings.audio_tracks[0].join("|") == "default_input" || Settings.audio_tracks[0].join("|") == "default_output|default_input")) {
                    return true;
                }

                if (Settings.audio_tracks.length == 2 && Settings.audio_tracks[0].join("|") == "default_output" && Settings.audio_tracks[1].join("|") == "default_input") {
                    return true;
                }

                return false;
            }
            function toggleTracks() {
                mergeTracks.checked = false;
                while (Settings.audio_tracks[0]) {
                    Settings.remove_audio_track(0);
                }
                var i = 0;
                if (recordSystem.checked) {
                    Settings.add_audio_track();
                    Settings.add_audio_source(i++, "default_output");
                }
                if (recordMicrophone.checked) {
                    Settings.add_audio_track();
                    Settings.add_audio_source(i++, "default_input");
                }
            }
            padding: 15
            spacing: Kirigami.Units.largeSpacing
            anchors.horizontalCenter: parent.horizontalCenter
            visible: isSimpleConfigVisible()

            Controls.Switch {
                id: recordSystem
                text: i18n("Record system sound")
                onToggled: function () {
                    simpleAudioTracksColumn.toggleTracks();
                }
            }

            Controls.Switch {
                id: recordMicrophone
                text: i18n("Record microphone")
                onToggled: function () {
                    simpleAudioTracksColumn.toggleTracks();
                }
            }

            Controls.Switch {
                id: mergeTracks
                text: i18n("Merge system & microphone tracks")
                enabled: recordSystem.checked && recordMicrophone.checked
                onToggled: function () {
                    if (mergeTracks.checked) {
                        while (Settings.audio_tracks[0]) {
                            Settings.remove_audio_track(0);
                        }
                        Settings.add_audio_track();
                        Settings.add_audio_source(0, "default_output");
                        Settings.add_audio_source(0, "default_input");
                    } else {
                        simpleAudioTracksColumn.toggleTracks();
                    }
                }
            }
        }

        Column {
            visible: !simpleAudioTracksColumn.visible
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width

            Kirigami.Icon {
                source: "dialog-information"
                height: 64
                width: 64
                anchors.horizontalCenter: parent.horizontalCenter
            }

            Controls.Label {
                text: i18n("Simple audio settings are not available when custom audio tracks are configured.")
                wrapMode: Text.Wrap
                width: parent.width
                padding: 30
                horizontalAlignment: Text.AlignHCenter
                font.pixelSize: 16
                anchors.horizontalCenter: parent.horizontalCenter
            }

            Controls.Button {
                text: i18n("Reset audio tracks")
                icon.name: "edit-undo"
                anchors.horizontalCenter: parent.horizontalCenter
                onClicked: function () {
                    while (Settings.audio_tracks[0]) {
                        Settings.remove_audio_track(0);
                    }
                    Settings.add_audio_track();
                    Settings.add_audio_track();
                    Settings.add_audio_source(0, "default_output");
                    Settings.add_audio_source(1, "default_input");
                    simpleAudioTracks.checkSwitches();
                }
            }
        }
    }

    Controls.ScrollView {
        id: advancedAudioTracks
        visible: false
        Layout.fillWidth: true
        Layout.fillHeight: true
        Controls.ScrollBar.horizontal.policy: Controls.ScrollBar.AlwaysOff

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
                                        Layout.maximumWidth: sourcesList.width - 16
                                        elide: Text.ElideMiddle
                                        text: switch (modelData) {
                                        case "default_input":
                                            " - " + i18n("Default Microphone");
                                            break;
                                        case "default_output":
                                            " - " + i18n("System Sound");
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
                                text: i18n("Add source")
                                onClicked: function () {
                                    addDialog.trackIndex = index;
                                    addDialog.open();
                                }
                            }
                        }
                    }
                }
            }

            footer: Column {
                anchors {
                    left: parent.left
                    right: parent.right
                }

                Item {
                    width: 1
                    height: Kirigami.Units.largeSpacing * 2
                }

                Controls.Button {
                    icon.name: "list-add"
                    text: i18n("Add track")
                    anchors.horizontalCenter: parent.horizontalCenter

                    onClicked: function () {
                        Settings.add_audio_track();
                    }
                }
            }
        }
    }

    Kirigami.Dialog {
        id: addDialog
        property int trackIndex: 0
        title: i18n("Add audio source")
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
                    track = otherDevice.currentValue;
                } else if (applicationRadio.checked) {
                    track = "app:" + application.currentValue;
                }
                Settings.add_audio_source(addDialog.trackIndex, track);
                addDialog.close();
            }
        }

        ColumnLayout {
            Controls.RadioButton {
                id: defaultMicrophoneRadio
                text: i18n("Default microphone")
                checked: true
            }

            Controls.RadioButton {
                id: systemSoundRadio
                text: i18n("System sound")
            }

            Controls.RadioButton {
                id: applicationRadio
                text: i18n("Application:")
            }

            Controls.ComboBox {
                id: application
                Layout.fillWidth: true
                Layout.maximumWidth: 300
                model: Settings.audio_applications
                enabled: applicationRadio.checked
            }

            Controls.RadioButton {
                id: otherDeviceRadio
                text: i18n("Other device:")
            }

            Controls.ComboBox {
                id: otherDevice
                Layout.fillWidth: true
                Layout.maximumWidth: 300

                model: Settings.audio_devices.map(e => {
                    let split = e.split("|");
                    return {
                        text: split[1],
                        value: split[0]
                    };
                })
                textRole: "text"
                valueRole: "value"
                enabled: otherDeviceRadio.checked
            }
        }
    }
}
