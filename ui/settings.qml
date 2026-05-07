import QtQuick
import org.kde.kirigami as Kirigami

QtObject {
    property var settingsWindow: Kirigami.ApplicationWindow {
        id: window
        objectName: "window"
        title: qsTr("TrayPlay Settings")
        width: 500
        minimumWidth: 500
        height: 580
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
}
