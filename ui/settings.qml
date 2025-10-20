import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    id: window
    title: "TrayPlay Settings"
    width: 500
    minimumWidth: 500
    height: 560

    pageStack.initialPage: Qt.resolvedUrl("MainPage.qml")
}
