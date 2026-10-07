import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root

    property string desktopId: ""
    property string name: ""
    property string iconPath: ""
    property bool running: false
    property bool focused: false
    property bool urgent: false
    property var windows: []
    property var theme: null
    property int activeCount: 0
    property int badgeCount: 0
    property string profile: "mac" // "general", "windows", "mac"
    property real slotSize: 48
    property real iconSize: 36
    property bool magnification: true
    property bool showIndicators: true

    signal clicked()
    signal middleClicked()
    signal rightClicked(real mouseX, real mouseY)
    signal focusWindow(string address)
    signal closeWindow(string address)

    width: slotSize
    height: slotSize

    readonly property bool isHovered: mouseArea.containsMouse || previewPopup.hovered
    readonly property real scaleFactor: {
        if (mouseArea.pressed) return 0.92
        if (root.magnification && isHovered) return 1.25
        if (isHovered) return 1.08
        return 1.0
    }

    Behavior on scale {
        NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
    }
    scale: scaleFactor

    transform: Translate {
        y: (root.magnification && root.isHovered && !mouseArea.pressed) ? -6 : 0
        Behavior on y {
            NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
        }
    }

    // Urgency Pulse Aura
    Rectangle {
        id: urgentAura
        visible: root.urgent
        anchors.centerIn: parent
        width: 44
        height: 44
        radius: 22
        color: "transparent"
        border.color: (root.theme && root.theme.urgent) ? root.theme.urgent : "#ef4444"
        border.width: 2
        z: -1

        SequentialAnimation on opacity {
            running: root.urgent
            loops: Animation.Infinite
            NumberAnimation { from: 0.2; to: 1.0; duration: 600; easing.type: Easing.InOutQuad }
            NumberAnimation { from: 1.0; to: 0.2; duration: 600; easing.type: Easing.InOutQuad }
        }

        SequentialAnimation on scale {
            running: root.urgent
            loops: Animation.Infinite
            NumberAnimation { from: 0.95; to: 1.15; duration: 600; easing.type: Easing.InOutQuad }
            NumberAnimation { from: 1.15; to: 0.95; duration: 600; easing.type: Easing.InOutQuad }
        }
    }

    // Background selection indicator for Windows profile
    Rectangle {
        id: winPlate
        visible: root.profile === "windows" && (root.running || root.isHovered)
        anchors.fill: parent
        anchors.margins: 2
        radius: 4
        color: root.focused ? Qt.rgba(1, 1, 1, 0.18) : (root.isHovered ? Qt.rgba(1, 1, 1, 0.10) : Qt.rgba(1, 1, 1, 0.04))
        border.color: root.focused ? Qt.rgba(1, 1, 1, 0.3) : "transparent"
        border.width: 1
    }

    // App Icon Image
    Image {
        id: iconImg
        anchors.centerIn: parent
        width: root.iconSize
        height: root.iconSize
        source: {
            if (!root.iconPath || root.iconPath.length === 0) return ""
            if (root.iconPath.indexOf("://") !== -1) return root.iconPath
            if (root.iconPath.charAt(0) === "/") return "file://" + root.iconPath
            return "image://icon/" + root.iconPath
        }
        sourceSize.width: Math.max(64, root.iconSize * 2)
        sourceSize.height: Math.max(64, root.iconSize * 2)
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        smooth: true
        mipmap: true

        // Fallback letter glyph if icon missing
        Rectangle {
            anchors.fill: parent
            visible: iconImg.status !== Image.Ready
            color: (root.theme && root.theme.card) ? root.theme.card : "#334155"
            radius: 8
            Text {
                anchors.centerIn: parent
                text: root.name.length > 0 ? root.name.charAt(0).toUpperCase() : "?"
                color: (root.theme && root.theme.foreground) ? root.theme.foreground : "#f8fafc"
                font.bold: true
                font.pixelSize: Math.round(root.iconSize * 0.45)
            }
        }
    }

    // Running indicator dot / pill
    Rectangle {
        id: runningDot
        visible: root.showIndicators && root.running
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 2
        width: root.focused ? 12 : 4
        height: 3
        radius: 1.5
        color: root.urgent
            ? ((root.theme && root.theme.urgent) ? root.theme.urgent : "#ef4444")
            : (root.focused
                ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                : Qt.rgba(1, 1, 1, 0.65))

        Behavior on width {
            NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
        }
        Behavior on color {
            ColorAnimation { duration: 150 }
        }
    }

    // Notification Badge
    Rectangle {
        id: badge
        visible: root.badgeCount > 0
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: -2
        anchors.rightMargin: -2
        width: Math.max(16, badgeText.implicitWidth + 6)
        height: 16
        radius: 8
        color: (root.theme && root.theme.urgent) ? root.theme.urgent : "#ef4444"
        border.color: (root.theme && root.theme.background) ? root.theme.background : "#0f172a"
        border.width: 1.5

        Text {
            id: badgeText
            anchors.centerIn: parent
            text: root.badgeCount > 99 ? "99+" : root.badgeCount.toString()
            color: "#ffffff"
            font.pixelSize: 9
            font.bold: true
        }
    }

    // Single-Window Tooltip
    ToolTip {
        visible: root.isHovered && root.windows.length <= 1 && root.name.length > 0
        text: root.name + (root.activeCount > 0 ? " (running)" : "")
        delay: 400
        timeout: 4000
    }

    // Multi-Window Hover Preview Card
    Popup {
        id: previewPopup
        readonly property bool hovered: previewMouse.containsMouse
        visible: root.isHovered && root.windows.length > 1
        x: (root.width - width) / 2
        y: -height - 12
        width: 220
        height: Math.min(260, previewList.implicitHeight + 20)
        padding: 8
        closePolicy: Popup.NoAutoClose

        background: Rectangle {
            color: (root.theme && root.theme.card) ? root.theme.card : "#1e293b"
            radius: 8
            border.color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.15)
            border.width: 1
        }

        MouseArea {
            id: previewMouse
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }

        ColumnLayout {
            id: previewList
            anchors.fill: parent
            spacing: 4

            Text {
                text: root.name + " (" + root.windows.length + " windows)"
                color: (root.theme && root.theme.foreground) ? root.theme.foreground : "#f8fafc"
                font.bold: true
                font.pixelSize: 11
                Layout.fillWidth: true
                elide: Text.ElideRight
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: Qt.rgba(1, 1, 1, 0.1)
            }

            Repeater {
                model: root.windows

                delegate: Rectangle {
                    Layout.fillWidth: true
                    height: 26
                    radius: 4
                    color: winRowMouse.containsMouse ? Qt.rgba(1, 1, 1, 0.12) : (modelData.focused ? Qt.rgba(1, 1, 1, 0.06) : "transparent")

                    MouseArea {
                        id: winRowMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: root.focusWindow(modelData.address)
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 4
                        spacing: 6

                        Rectangle {
                            width: 6
                            height: 6
                            radius: 3
                            color: modelData.focused ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8") : (modelData.urgent ? ((root.theme && root.theme.urgent) ? root.theme.urgent : "#ef4444") : Qt.rgba(1, 1, 1, 0.4))
                        }

                        Text {
                            text: modelData.title || "Window"
                            color: (root.theme && root.theme.foreground) ? root.theme.foreground : "#f8fafc"
                            font.pixelSize: 11
                            Layout.fillWidth: true
                            elide: Text.ElideRight
                        }

                        Text {
                            text: "WS " + (modelData.workspace_name || modelData.workspace_id)
                            color: (root.theme && root.theme.muted) ? root.theme.muted : "#94a3b8"
                            font.pixelSize: 9
                        }

                        Rectangle {
                            width: 16
                            height: 16
                            radius: 8
                            color: winCloseMouse.containsMouse ? Qt.rgba(1, 0, 0, 0.2) : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "✕"
                                color: winCloseMouse.containsMouse ? "#ef4444" : ((root.theme && root.theme.muted) ? root.theme.muted : "#94a3b8")
                                font.pixelSize: 10
                            }

                            MouseArea {
                                id: winCloseMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: root.closeWindow(modelData.address)
                            }
                        }
                    }
                }
            }
        }
    }

    // Mouse Area
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton

        onClicked: function(mouse) {
            if (mouse.button === Qt.LeftButton) {
                root.clicked()
            } else if (mouse.button === Qt.MiddleButton) {
                root.middleClicked()
            } else if (mouse.button === Qt.RightButton) {
                root.rightClicked(mouse.x, mouse.y)
            }
        }
    }
}
