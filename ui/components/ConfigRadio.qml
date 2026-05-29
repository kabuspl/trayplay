import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts

Controls.RadioButton {
    id: radio
    Layout.fillWidth: true

    contentItem: Controls.Label {
        text: radio.text
        wrapMode: Text.Wrap
        verticalAlignment: Text.AlignVCenter
        leftPadding: radio.indicator && !radio.mirrored ? radio.indicator.width + radio.spacing : 0
        rightPadding: radio.indicator && radio.mirrored ? radio.indicator.width + radio.spacing : 0
    }
}
