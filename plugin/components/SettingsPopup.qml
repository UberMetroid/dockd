import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Popup {
    id: root

    property string currentProfile: "general"
    property bool showFileShortcuts: true
    property bool autoHide: false
    property bool filterCurrentMonitor: false

    signal profileChanged(string profile)
    signal fileShortcutsToggled(bool enabled)
    signal autoHideToggled(bool enabled)
    signal filterCurrentMonitorToggled(bool enabled)

    width: 250
    height: contentColumn.implicitHeight + 32
    padding: 16
    modal: true
    focus: true

    background: Rectangle {
        color: "#1e293b"
        radius: 12
        border.color: "#334155"
        border.width: 1
    }

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        spacing: 12

        Text {
            text: "Dock Preferences"
            color: "#f8fafc"
            font.bold: true
            font.pixelSize: 14
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: "#334155"
        }

        Text {
            text: "Layout Style"
            color: "#94a3b8"
            font.pixelSize: 12
        }

        RowLayout {
            spacing: 8

            Button {
                text: "General"
                highlighted: root.currentProfile === "general"
                onClicked: root.profileChanged("general")
            }
            Button {
                text: "Windows"
                highlighted: root.currentProfile === "windows"
                onClicked: root.profileChanged("windows")
            }
            Button {
                text: "macOS"
                highlighted: root.currentProfile === "mac"
                onClicked: root.profileChanged("mac")
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: "#334155"
        }

        CheckBox {
            text: "File Shortcuts (Home/Trash)"
            checked: root.showFileShortcuts
            onToggled: root.fileShortcutsToggled(checked)
        }

        CheckBox {
            text: "Auto-hide on window overlap"
            checked: root.autoHide
            onToggled: {
                root.autoHide = checked
                root.autoHideToggled(checked)
            }
        }

        CheckBox {
            text: "Filter by current monitor"
            checked: root.filterCurrentMonitor
            onToggled: root.filterCurrentMonitorToggled(checked)
        }
    }
}
