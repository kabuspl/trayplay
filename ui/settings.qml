import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.ApplicationWindow {
    id: window
    title: "TrayPlay Settings"
    width: 400
    minimumWidth: 400
    height: pageStack.initialPage.height

    pageStack.initialPage: Qt.resolvedUrl("MainPage.qml")
}
