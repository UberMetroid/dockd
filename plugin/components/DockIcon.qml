import QtQuick
import QtQuick.Controls

Item {
    id: root

    property string desktopId: ""
    property string name: ""
    property string iconPath: ""
    property bool running: false
    property bool focused: false
    property int activeCount: 0
    property int badgeCount: 0
    property string profile: "general" // "general", "windows", "mac"

    signal clicked()
    signal middleClicked()
    signal rightClicked(real mouseX, real mouseY)

    width: 48
    height: 48

    readonly property bool isHovered: mouseArea.containsMouse
    readonly property real scaleFactor: {
        if (profile === "mac" && isHovered) return 1.25
        if (isHovered) return 1.10
        return 1.0
    }

    Behavior on scale {
        NumberAnimation { duration: 120; easing.type: Easing.OutQuad }
    }
    scale: scaleFactor

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
        width: 36
        height: 36
        source: root.iconPath.length > 0 ? "file://" + root.iconPath : ""
        sourceSize.width: 48
        sourceSize.height: 48
        fillMode: Image.PreserveAspectFit
        asynchronous: true

        // Fallback letter glyph if icon missing
        Rectangle {
            anchors.fill: parent
            visible: iconImg.status !== Image.Ready
            color: "#334155"
            radius: 8
            Text {
                anchors.centerIn: parent
                text: root.name.length > 0 ? root.name.charAt(0).toUpperCase() : "?"
                color: "#f8fafc"
                font.bold: true
                font.pixelSize: 18
            }
        }
    }

    // Running indicator dot / pill
    Rectangle {
        id: runningDot
        visible: root.running
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 2
        width: root.profile === "windows" ? (root.focused ? 18 : 8) : (root.activeCount > 1 ? 8 : 5)
        height: root.profile === "windows" ? 3 : 5
        radius: 2.5
        color: root.focused ? "#38bdf8" : Qt.rgba(1, 1, 1, 0.7)

        Behavior on width {
            NumberAnimation { duration: 150; easing.type: Easing.OutQuad }
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
        color: "#ef4444"
        border.color: "#0f172a"
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

    // Tooltip
    ToolTip {
        visible: root.isHovered && root.name.length > 0
        text: root.name + (root.activeCount > 1 ? " (" + root.activeCount + ")" : "")
        delay: 400
        timeout: 4000
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
