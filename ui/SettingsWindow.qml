import "components"

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts

import org.kde.kcmutils as KCMUtils
import org.kde.kirigami as Kirigami
import Settings

Kirigami.ApplicationWindow {
    id: root
    title: i18n("Settings")
    width: 750
    minimumWidth: 200
    height: 580
    visible: false

    property url currentSource
    property bool settingsDirty: false

    function markDirty() {
        settingsDirty = true;
    }

    function applySettings() {
        const page = app.pageStack.currentItem;
        if (page && typeof page.saveChanges === "function") {
            page.saveChanges();
        }
        Settings.apply_config();
        settingsDirty = false;
    }

    function discardSettings() {
        Settings.discard_changes();
        settingsDirty = false;
        if (currentSource) {
            open(currentSource);
        }
    }

    function iclosing() {
        if (applyButton.enabled) {
            messageDialog.item = null;
            messageDialog.open();
            return false;
        }
        return true;
    }

    function pushReplace(item) {
        let page;
        if (app.pageStack.depth === 0) {
            page = app.pageStack.push(item);
        } else {
            page = app.pageStack.replace(item);
        }
        app.currentConfigPage = page;
        if ("settingsWindow" in page) {
            page.settingsWindow = root;
        }
    }

    function open(url) {
        currentSource = url;
        app.isAboutPage = url === "AboutPage.qml";
        pushReplace(Qt.resolvedUrl(url));
    }

    QQC2.ScrollView {
        id: categoriesScroll
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
        }
        width: Kirigami.Units.gridUnit * 7
        contentWidth: availableWidth
        Kirigami.Theme.colorSet: Kirigami.Theme.View
        Kirigami.Theme.inherit: false
        activeFocusOnTab: true
        focus: true
        Accessible.role: Accessible.PageTabList
        background: Rectangle {
            color: Kirigami.Theme.backgroundColor
        }
        ColumnLayout {
            id: categories

            spacing: 0
            width: categoriesScroll.contentWidth
            focus: true

            function openCategory(item) {
                if (root.settingsDirty) {
                    messageDialog.item = item;
                    messageDialog.open();
                    return;
                }
                root.open(item);
            }

            ConfigCategoryDelegate {
                item: "MainPage.qml"
                name: i18n("General")
                iconName: "settings-configure"
                onActivated: categories.openCategory(item)
                highlighted: root.currentSource == item
            }

            ConfigCategoryDelegate {
                item: "VideoPage.qml"
                name: i18n("Video")
                iconName: "video-symbolic"
                onActivated: categories.openCategory(item)
                highlighted: root.currentSource == item
            }

            ConfigCategoryDelegate {
                item: "AudioPage.qml"
                name: i18n("Audio tracks")
                iconName: "audio-headphones-symbolic"
                onActivated: categories.openCategory(item)
                highlighted: root.currentSource == item
            }

            ConfigCategoryDelegate {
                item: "AboutPage.qml"
                name: i18n("About")
                iconName: "help-about"
                onActivated: categories.openCategory(item)
                highlighted: root.currentSource == item
            }
        }
    }

    Kirigami.Separator {
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }
        z: 1
    }
    Kirigami.Separator {
        id: verticalSeparator
        anchors {
            top: parent.top
            left: categoriesScroll.right
            bottom: parent.bottom
        }
        z: 1
    }

    Kirigami.ApplicationItem {
        id: app
        anchors {
            top: parent.top
            left: verticalSeparator.right
            right: parent.right
            bottom: parent.bottom
        }

        pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.Auto
        wideScreen: true
        pageStack.globalToolBar.separatorVisible: bottomSeparator.visible
        pageStack.globalToolBar.colorSet: Kirigami.Theme.Window

        property var currentConfigPage: null
        property bool isAboutPage: false

        Kirigami.PromptDialog {
            id: messageDialog
            property var item
            title: i18n("Apply Settings")
            subtitle: i18n("The current page has unsaved changes. Apply the changes or discard them?")
            standardButtons: Kirigami.Dialog.Apply | Kirigami.Dialog.Discard | Kirigami.Dialog.Cancel
            onApplied: {
                root.applySettings();
                if (item) {
                    root.open(item);
                } else {
                    root.visible = false;
                }
                messageDialog.close();
            }
            onDiscarded: {
                root.discardSettings();
                if (item) {
                    root.open(item);
                } else {
                    root.visible = false;
                }
                messageDialog.close();
            }
        }

        footer: QQC2.Pane {

            padding: Kirigami.Units.largeSpacing

            contentItem: RowLayout {
                id: buttonsRow
                spacing: Kirigami.Units.smallSpacing

                Item {
                    Layout.fillWidth: true
                }

                QQC2.Button {
                    icon.name: "dialog-ok"
                    text: i18n("OK")
                    onClicked: acceptAction.trigger()
                }
                QQC2.Button {
                    id: applyButton
                    enabled: root.settingsDirty
                    icon.name: "dialog-ok-apply"
                    text: i18n("Apply")
                    visible: !app.isAboutPage && app.pageStack.currentItem
                    onClicked: applyAction.trigger()
                }
                QQC2.Button {
                    icon.name: "dialog-cancel"
                    text: i18n("Cancel")
                    onClicked: cancelAction.trigger()
                    visible: !app.isAboutPage
                    KeyNavigation.tab: categories
                }
            }
            background: Item {
                Kirigami.Separator {
                    id: bottomSeparator
                    visible: (app.pageStack.currentItem
                        && app.pageStack.currentItem.flickable
                        && !(app.pageStack.currentItem.flickable instanceof KCMUtils.GridViewKCM)
                        && !(app.pageStack.currentItem.flickable.atYBeginning
                        && app.pageStack.currentItem.flickable.atYEnd)) ?? false
                    anchors {
                        left: parent.left
                        right: parent.right
                        top: parent.top
                    }
                }
            }
        }

        QQC2.Action {
            id: acceptAction
            onTriggered: {
                if (root.settingsDirty) {
                    root.applySettings();
                }
                root.visible = false;
            }
        }

        QQC2.Action {
            id: applyAction
            onTriggered: {
                root.applySettings();
            }
        }

        QQC2.Action {
            id: cancelAction
            onTriggered: {
                if (root.iclosing()) {
                    root.close();
                }
            }
        }

        Keys.onReturnPressed: acceptAction.trigger()
        Keys.onEscapePressed: cancelAction.trigger()
    }

    Component.onCompleted: root.open("MainPage.qml")

    onClosing: close => {
        close.accepted = false;
        if (root.iclosing()) {
            root.visible = false;
        }
    }
}
