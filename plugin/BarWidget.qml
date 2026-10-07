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

    property string profile: "general"
    property string helperPath: (typeof Quickshell !== "undefined" && Quickshell.env("DOCKD_BIN")) ? Quickshell.env("DOCKD_BIN") : "dockd"

    Rectangle {
        id: buttonPlate
        anchors.fill: parent
        radius: 6
        color: mouseArea.containsMouse ? Qt.rgba(1, 1, 1, 0.12) : "transparent"

        // Dock icon symbol
        Row {
            anchors.centerIn: parent
            spacing: 3

            Rectangle { width: 4; height: 14; radius: 2; color: "#38bdf8" }
            Rectangle { width: 4; height: 18; radius: 2; color: "#f8fafc" }
            Rectangle { width: 4; height: 14; radius: 2; color: "#38bdf8" }
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onClicked: function(mouse) {
            if (mouse.button === Qt.LeftButton || mouse.button === Qt.RightButton) {
                quickMenu.popup(root, mouse.x, mouse.y)
            }
        }
    }

    Menu {
        id: quickMenu
        title: "dockd"

        MenuItem {
            text: "Layout: General"
            onTriggered: rpcProcess.execute(["profile", "general"])
        }
        MenuItem {
            text: "Layout: Windows Taskbar"
            onTriggered: rpcProcess.execute(["profile", "windows"])
        }
        MenuItem {
            text: "Layout: macOS Style"
            onTriggered: rpcProcess.execute(["profile", "mac"])
        }
        MenuSeparator {}
        MenuItem {
            text: "Refresh State"
            onTriggered: rpcProcess.execute(["state"])
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
