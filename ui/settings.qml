import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    id: window
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
