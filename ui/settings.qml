import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    id: window
    title: qsTr("TrayPlay Settings")
    width: 500
    minimumWidth: 500
    height: 560

    pageStack.defaultColumnWidth: 500
    pageStack.initialPage: Qt.resolvedUrl("MainPage.qml")
}
