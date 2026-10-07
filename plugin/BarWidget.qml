import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import "components"

Item {
    id: root

    implicitWidth: 32
    implicitHeight: 28

    property string helperPath: (typeof Quickshell !== "undefined" && Quickshell.env("DOCKD_BIN")) ? Quickshell.env("DOCKD_BIN") : "dockd"

    Rectangle {
        id: buttonPlate
        anchors.fill: parent
        radius: 6
        color: mouseArea.containsMouse ? Qt.rgba(1, 1, 1, 0.14) : "transparent"

        // Dock icon symbol
        Row {
            anchors.centerIn: parent
            spacing: 3

            Rectangle { width: 3.5; height: 12; radius: 1.5; color: "#38bdf8" }
            Rectangle { width: 3.5; height: 16; radius: 1.5; color: "#ffffff" }
            Rectangle { width: 3.5; height: 12; radius: 1.5; color: "#38bdf8" }
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onClicked: function(mouse) {
            settingsPopup.popup(buttonPlate, 0, buttonPlate.height + 6)
        }
    }

    ToolTip.visible: mouseArea.containsMouse
    ToolTip.text: "Dock Preferences"
    ToolTip.delay: 300

    SettingsPopup {
        id: settingsPopup

        onDockSizeChanged: function(size) {
            rpcProcess.execute(["size", size])
        }
        onMagnificationToggled: function(enabled) {
            rpcProcess.execute(["magnification", enabled ? "on" : "off"])
        }
        onAutoHideToggled: function(enabled) {
            rpcProcess.execute(["autohide", enabled ? "on" : "off"])
        }
        onFilterCurrentMonitorToggled: function(enabled) {
            rpcProcess.execute(["monitor", enabled ? "0" : "all"])
        }
        onResetPinnedTriggered: function() {
            rpcProcess.execute(["reset"])
        }
    }

    Process {
        id: rpcProcess
        function execute(args) {
            command = [root.helperPath].concat(args)
            running = true
        }
    }
}
