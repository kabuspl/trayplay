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
    title: i18n("Video")
    property var settingsWindow

    function markDirty() {
        if (settingsWindow) {
            settingsWindow.markDirty();
        }
    }

    function saveChanges() {
        Settings.framerate = framerate.value;
        Settings.quality = quality.currentIndex;
        Settings.codec = codec.currentIndex;
        Settings.video_source_choice = video_source.currentValue;
    }

    Kirigami.FormLayout {
        width: parent.width
        height: parent.height

        Controls.ComboBox {
            id: video_source
            Kirigami.FormData.label: i18n("Video source:")
            Layout.fillWidth: true
            model: [
                {
                    text: i18n("Default (%0)").arg(Settings.video_sources[0].split("|")[0]),
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
            onActivated: mainPage.markDirty()
        }

        Controls.ComboBox {
            id: codec
            Kirigami.FormData.label: i18n("Codec:")
            Layout.fillWidth: true
            model: ["H.264", "H.265 (HEVC)", "H.265 (HEVC) HDR", "H.265 (HEVC) 10-bit", "AV1", "AV1 HDR", "AV1 10-bit", "VP8", "VP9"]
            currentIndex: Settings.codec
            onActivated: mainPage.markDirty()
        }

        Controls.ComboBox {
            id: quality
            Kirigami.FormData.label: i18n("Quality:")
            Layout.fillWidth: true
            model: ["Medium", "High", "Very high", "Ultra"]
            currentIndex: Settings.quality
            onActivated: mainPage.markDirty()
        }

        RowLayout {
            Layout.fillWidth: true
            Kirigami.FormData.label: i18n("Framerate:")

            Controls.SpinBox {
                id: framerate
                Layout.fillWidth: true
                from: 1
                to: 1000
                stepSize: 5
                value: Settings.framerate
                onValueModified: mainPage.markDirty()
            }

            Controls.Label {
                text: "FPS"
            }
        }
    }
}
