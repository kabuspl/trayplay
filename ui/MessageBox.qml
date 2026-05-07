import QtQuick
import org.kde.kirigami as Kirigami
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: messageBox
    title: "TrayPlay"
    property alias text: messageBoxLabel.text
    property alias icon: messageBoxIcon.source
    minimumWidth: 400
    maximumWidth: 400
    minimumHeight: mainLayout.implicitHeight + 20
    maximumHeight: mainLayout.implicitHeight + 20

    color: active ? palette.active.window : palette.inactive.window

    Kirigami.Separator {
        id: topBorder
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 1
        z: 999
    }

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Kirigami.Units.largeSpacing

            Kirigami.Icon {
                id: messageBoxIcon
                source: "dialog-information"
                Layout.preferredHeight: 64
                Layout.preferredWidth: 64
                Layout.alignment: Qt.AlignTop
            }

            TextEdit {
                id: messageBoxLabel
                Layout.fillWidth: true
                Layout.fillHeight: true
                wrapMode: Text.WordWrap
                selectByMouse: true
                readOnly: true
                color: Kirigami.Theme.textColor
                text: ""
            }
        }

        Button {
            id: messageBoxButton
            Layout.alignment: Qt.AlignRight
            text: "OK"
            onClicked: messageBox.visible = false
            highlighted: true
            icon.name: "dialog-ok"
            focus: true
            activeFocusOnTab: true

            Keys.onReturnPressed: messageBoxButton.click()
            Keys.onEscapePressed: messageBoxButton.click()
        }
    }
}
