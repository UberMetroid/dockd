import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Popup {
    id: root

    property string dockSize: "medium" // "small", "medium", "large"
    property bool magnification: true
    property bool autoHide: true
    property bool showIndicators: true
    property bool filterCurrentMonitor: false
    property int systemRounding: 12
    property var theme: null

    signal dockSizeChanged(string size)
    signal magnificationToggled(bool enabled)
    signal autoHideToggled(bool enabled)
    signal showIndicatorsToggled(bool enabled)
    signal filterCurrentMonitorToggled(bool enabled)
    signal resetPinnedTriggered()

    width: 380
    height: contentColumn.implicitHeight + 36
    padding: 18
    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    background: Rectangle {
        color: (root.theme && root.theme.background) ? Qt.rgba(0.09, 0.12, 0.18, 0.94) : "#141b27"
        radius: Math.max(10, root.systemRounding)
        border.color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.15)
        border.width: 1
    }

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        spacing: 12

        // Header with title and close button
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Rectangle {
                width: 30
                height: 30
                radius: 8
                color: (root.theme && root.theme.accent) ? Qt.rgba(0.22, 0.74, 0.97, 0.2) : Qt.rgba(1, 1, 1, 0.1)
                border.color: (root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8"
                border.width: 1

                Row {
                    anchors.centerIn: parent
                    spacing: 2
                    Rectangle { width: 3; height: 10; radius: 1.5; color: (root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8" }
                    Rectangle { width: 3; height: 14; radius: 1.5; color: "#ffffff" }
                    Rectangle { width: 3; height: 10; radius: 1.5; color: (root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8" }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 1

                Text {
                    text: "Dock Preferences"
                    color: (root.theme && root.theme.foreground) ? root.theme.foreground : "#f8fafc"
                    font.bold: true
                    font.pixelSize: 14
                }

                Text {
                    text: "macOS-style dock appearance and behavior"
                    color: "#94a3b8"
                    font.pixelSize: 10
                }
            }

            Rectangle {
                width: 22
                height: 22
                radius: 11
                color: closeMouse.containsMouse ? Qt.rgba(1, 1, 1, 0.18) : Qt.rgba(1, 1, 1, 0.08)

                Text {
                    anchors.centerIn: parent
                    text: "✕"
                    color: "#94a3b8"
                    font.pixelSize: 10
                }

                MouseArea {
                    id: closeMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.close()
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Qt.rgba(1, 1, 1, 0.1)
        }

        // Section 1: Dock Size Segmented Control
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: "Dock Size"
                color: "#e2e8f0"
                font.pixelSize: 12
                font.bold: true
            }

            Rectangle {
                Layout.fillWidth: true
                height: 32
                radius: 8
                color: Qt.rgba(0, 0, 0, 0.25)
                border.color: Qt.rgba(1, 1, 1, 0.1)
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 2
                    spacing: 2

                    Repeater {
                        model: [
                            { key: "small", label: "Small" },
                            { key: "medium", label: "Medium" },
                            { key: "large", label: "Large" }
                        ]

                        delegate: Rectangle {
                            required property var modelData
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            radius: 6
                            readonly property bool isSelected: root.dockSize === modelData.key
                            color: isSelected
                                ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                                : (segmentMouse.containsMouse ? Qt.rgba(1, 1, 1, 0.08) : "transparent")
                            Behavior on color { ColorAnimation { duration: 120 } }

                            Text {
                                anchors.centerIn: parent
                                text: parent.modelData.label
                                font.pixelSize: 11
                                font.bold: parent.isSelected
                                color: parent.isSelected ? "#0f172a" : "#cbd5e1"
                            }

                            MouseArea {
                                id: segmentMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.dockSize = parent.modelData.key
                                    root.dockSizeChanged(parent.modelData.key)
                                }
                            }
                        }
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Qt.rgba(1, 1, 1, 0.08)
        }

        // Section 2: Magnification Toggle
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Magnification"
                    color: "#f8fafc"
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    text: "Magnify icons when moving pointer over them"
                    color: "#94a3b8"
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            Rectangle {
                Layout.preferredWidth: 36
                Layout.preferredHeight: 20
                radius: 10
                color: root.magnification
                    ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                    : Qt.rgba(1, 1, 1, 0.2)
                Behavior on color { ColorAnimation { duration: 150 } }

                Rectangle {
                    width: 14
                    height: 14
                    radius: 7
                    anchors.verticalCenter: parent.verticalCenter
                    x: root.magnification ? 19 : 3
                    color: "#ffffff"
                    Behavior on x { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.magnification = !root.magnification
                        root.magnificationToggled(root.magnification)
                    }
                }
            }
        }

        // Section 3: Auto-hide Toggle
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Automatically hide and show the Dock"
                    color: "#f8fafc"
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    text: "Dodge open windows; reveal on bottom screen edge hover"
                    color: "#94a3b8"
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            Rectangle {
                Layout.preferredWidth: 36
                Layout.preferredHeight: 20
                radius: 10
                color: root.autoHide
                    ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                    : Qt.rgba(1, 1, 1, 0.2)
                Behavior on color { ColorAnimation { duration: 150 } }

                Rectangle {
                    width: 14
                    height: 14
                    radius: 7
                    anchors.verticalCenter: parent.verticalCenter
                    x: root.autoHide ? 19 : 3
                    color: "#ffffff"
                    Behavior on x { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.autoHide = !root.autoHide
                        root.autoHideToggled(root.autoHide)
                    }
                }
            }
        }

        // Section 4: Running Application Indicators Toggle
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Show open application indicators"
                    color: "#f8fafc"
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    text: "Display indicator dots beneath running applications"
                    color: "#94a3b8"
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            Rectangle {
                Layout.preferredWidth: 36
                Layout.preferredHeight: 20
                radius: 10
                color: root.showIndicators
                    ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                    : Qt.rgba(1, 1, 1, 0.2)
                Behavior on color { ColorAnimation { duration: 150 } }

                Rectangle {
                    width: 14
                    height: 14
                    radius: 7
                    anchors.verticalCenter: parent.verticalCenter
                    x: root.showIndicators ? 19 : 3
                    color: "#ffffff"
                    Behavior on x { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.showIndicators = !root.showIndicators
                        root.showIndicatorsToggled(root.showIndicators)
                    }
                }
            }
        }

        // Section 5: Filter by Current Monitor Toggle
        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Filter by active display"
                    color: "#f8fafc"
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    text: "Show running indicators only for windows on current screen"
                    color: "#94a3b8"
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }

            Rectangle {
                Layout.preferredWidth: 36
                Layout.preferredHeight: 20
                radius: 10
                color: root.filterCurrentMonitor
                    ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                    : Qt.rgba(1, 1, 1, 0.2)
                Behavior on color { ColorAnimation { duration: 150 } }

                Rectangle {
                    width: 14
                    height: 14
                    radius: 7
                    anchors.verticalCenter: parent.verticalCenter
                    x: root.filterCurrentMonitor ? 19 : 3
                    color: "#ffffff"
                    Behavior on x { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.filterCurrentMonitor = !root.filterCurrentMonitor
                        root.filterCurrentMonitorToggled(root.filterCurrentMonitor)
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Qt.rgba(1, 1, 1, 0.08)
        }

        // Section 6: Reset Pinned Apps Button
        Rectangle {
            Layout.fillWidth: true
            height: 32
            radius: 8
            color: resetMouse.containsMouse ? Qt.rgba(1, 1, 1, 0.12) : Qt.rgba(1, 1, 1, 0.05)
            border.color: resetMouse.containsMouse ? Qt.rgba(1, 1, 1, 0.25) : Qt.rgba(1, 1, 1, 0.1)
            border.width: 1
            Behavior on color { ColorAnimation { duration: 120 } }

            RowLayout {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    text: "↺"
                    color: "#cbd5e1"
                    font.pixelSize: 12
                }

                Text {
                    text: "Reset Pinned Apps to Defaults"
                    color: "#cbd5e1"
                    font.pixelSize: 11
                    font.bold: true
                }
            }

            MouseArea {
                id: resetMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    root.resetPinnedTriggered()
                    resetAnim.restart()
                }
            }

            SequentialAnimation on opacity {
                id: resetAnim
                running: false
                NumberAnimation { to: 0.5; duration: 100 }
                NumberAnimation { to: 1.0; duration: 150 }
            }
        }
    }
}
