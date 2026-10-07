import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "components"

Item {
    id: root

    // Host properties injected by Omarchy Shell host
    property var shell: null
    property var manifest: null

    // Default pinned items: ONLY File Explorer, Default Web Browser, Default Terminal
    readonly property var defaultDockItems: [
        {
            desktop_id: "nautilus.desktop",
            name: "Files",
            icon_path: "/usr/share/icons/hicolor/scalable/apps/org.gnome.Nautilus.svg",
            running: false,
            focused: false,
            urgent: false,
            windows: [],
            active_count: 0
        },
        {
            desktop_id: "firefox.desktop",
            name: "Web Browser",
            icon_path: "/usr/share/icons/hicolor/scalable/apps/firefox.svg",
            running: false,
            focused: false,
            urgent: false,
            windows: [],
            active_count: 0
        },
        {
            desktop_id: "org.wezfurlong.wezterm.desktop",
            name: "Terminal",
            icon_path: "/usr/share/icons/hicolor/scalable/apps/org.wezfurlong.wezterm.svg",
            running: false,
            focused: false,
            urgent: false,
            windows: [],
            active_count: 0
        }
    ]

    // Dock configuration state
    property string profile: "mac"
    property string dockSize: "medium" // "small", "medium", "large"
    property bool magnification: true
    property bool showIndicators: true
    property bool autoHide: true
    property bool proximityActive: false
    property var dockItems: defaultDockItems
    property string activeAddress: ""
    property bool overlap: false
    property bool hasWindows: false
    property bool filterMonitor: false
    property var theme: null

    // Dynamic system window corner radius (matches decoration:rounding from Hyprland)
    property int systemRounding: 12

    Process {
        id: roundingProc
        command: ["hyprctl", "-j", "getoption", "decoration:rounding"]
        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: {
                try {
                    var parsed = JSON.parse(text || "{}")
                    var n = Number(parsed.int)
                    if (isFinite(n) && n >= 0) {
                        root.systemRounding = n
                    }
                } catch(e) {}
            }
        }
    }

    // Geometry calculations based on dockSize
    readonly property real dockHeight: dockSize === "small" ? 48 : (dockSize === "large" ? 68 : 56)
    readonly property real slotDimension: dockSize === "small" ? 40 : (dockSize === "large" ? 58 : 48)
    readonly property real iconDimension: dockSize === "small" ? 28 : (dockSize === "large" ? 44 : 36)

    // Intellihide: hides when windows exist on screen, reveals on mouse proximity or empty desktop
    readonly property bool isDockHidden: {
        if (!root.autoHide) return false
        if (root.proximityActive || (dockPanelWindow && (dockPanelWindow.isHovered || dockPanelWindow.isProximityTriggered || dockPanelWindow.isPopupOpen))) return false
        return (root.overlap || root.hasWindows)
    }

    // Auto-hide hysteresis timer to prevent abrupt snapping
    Timer {
        id: autohideTimer
        interval: 400
        repeat: false
        onTriggered: {
            if (!dockPanelWindow.isHovered && !dockPanelWindow.isProximityTriggered && !dockPanelWindow.isPopupOpen) {
                root.proximityActive = false
            }
        }
    }

    // 1. Edge Trigger Layer Window: full-width strip at bottom edge to summon dock on hover
    PanelWindow {
        id: edgeTriggerWindow
        visible: root.isDockHidden

        WlrLayershell.namespace: "dockd-edge"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
        exclusionMode: ExclusionMode.Ignore
        color: "transparent"

        anchors {
            bottom: true
            left: true
            right: true
        }
        margins {
            bottom: 0
            left: 0
            right: 0
        }
        implicitHeight: 4

        Loader {
            id: edgeTriggerLoader
            anchors.fill: parent
            active: edgeTriggerWindow.visible

            sourceComponent: Component {
                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                    onEntered: {
                        autohideTimer.stop()
                        root.proximityActive = true
                    }
                }
            }
        }
    }

    // 2. The Persistent Layer-Shell Dock Window
    PanelWindow {
        id: dockPanelWindow

        readonly property bool isHovered: dockHoverArea.containsMouse
        readonly property bool isProximityTriggered: proximityTrigger.containsMouse
        readonly property bool isPopupOpen: (settingsPopup && settingsPopup.visible) || (appMenu && appMenu.visible)

        anchors {
            bottom: true
        }
        margins {
            bottom: 0
        }
        exclusiveZone: -1

        WlrLayershell.layer: root.autoHide ? WlrLayer.Overlay : WlrLayer.Top
        WlrLayershell.namespace: "dockd"

        implicitWidth: dockBar.implicitWidth + 48
        implicitHeight: root.dockHeight + 16

        color: "transparent"

        // Screen edge proximity trigger spanning the bottom area of the dock window
        MouseArea {
            id: proximityTrigger
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
            z: 1
            onEntered: {
                autohideTimer.stop()
                root.proximityActive = true
            }
            onExited: {
                autohideTimer.restart()
            }
        }

        // Background macOS frosted glass pill
        Rectangle {
            id: dockBar
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 8
            height: root.dockHeight
            implicitWidth: contentRow.implicitWidth + 18
            width: implicitWidth
            z: 2

            radius: root.systemRounding
            Behavior on radius {
                NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }

            color: (root.theme && root.theme.background)
                ? Qt.rgba(0.08, 0.11, 0.16, 0.78)
                : Qt.rgba(0.08, 0.11, 0.16, 0.78)
            border.color: (root.theme && root.theme.border)
                ? root.theme.border
                : Qt.rgba(1, 1, 1, 0.15)
            border.width: 1

            opacity: root.isDockHidden ? 0.75 : 1.0
            transform: Translate {
                y: root.isDockHidden ? (root.dockHeight - 4) : 0
                Behavior on y {
                    NumberAnimation { duration: 240; easing.type: Easing.OutCubic }
                }
            }
            Behavior on opacity {
                NumberAnimation { duration: 180 }
            }

            MouseArea {
                id: dockHoverArea
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.RightButton
                onEntered: {
                    autohideTimer.stop()
                    root.proximityActive = true
                }
                onExited: {
                    autohideTimer.restart()
                }
                onClicked: function(mouse) {
                    if (mouse.button === Qt.RightButton) {
                        settingsPopup.popup(dockBar, mouse.x, mouse.y)
                    }
                }
            }

            RowLayout {
                id: contentRow
                anchors.centerIn: parent
                spacing: 6
                opacity: root.isDockHidden ? 0.0 : 1.0
                Behavior on opacity {
                    NumberAnimation { duration: 160 }
                }

                // 1. App Launcher / Launchpad Button
                Item {
                    id: launcherBtn
                    width: root.slotDimension
                    height: root.slotDimension
                    Layout.alignment: Qt.AlignVCenter

                    Rectangle {
                        id: launcherPlate
                        anchors.centerIn: parent
                        width: root.slotDimension - 8
                        height: root.slotDimension - 8
                        radius: Math.max(4, root.systemRounding - 4)
                        color: launcherMouse.containsMouse
                            ? ((root.theme && root.theme.accent) ? Qt.rgba(0.22, 0.74, 0.97, 0.25) : Qt.rgba(1, 1, 1, 0.18))
                            : Qt.rgba(1, 1, 1, 0.08)
                        border.color: launcherMouse.containsMouse
                            ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                            : Qt.rgba(1, 1, 1, 0.12)
                        border.width: 1

                        scale: launcherMouse.pressed ? 0.92 : (launcherMouse.containsMouse && root.magnification ? 1.18 : 1.0)
                        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }

                        transform: Translate {
                            y: (root.magnification && launcherMouse.containsMouse && !launcherMouse.pressed) ? -4 : 0
                            Behavior on y { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                        }

                        // 9-dot silver grid icon (macOS Launchpad style)
                        Grid {
                            anchors.centerIn: parent
                            columns: 3
                            spacing: 3

                            Repeater {
                                model: 9
                                Rectangle {
                                    width: 3.5
                                    height: 3.5
                                    radius: 1.75
                                    color: launcherMouse.containsMouse
                                        ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                                        : "#ffffff"
                                }
                            }
                        }
                    }

                    MouseArea {
                        id: launcherMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: function(mouse) {
                            if (mouse.button === Qt.LeftButton) {
                                dockRpc.launchAppDrawer()
                            } else {
                                settingsPopup.popup(launcherBtn, mouse.x, mouse.y)
                            }
                        }
                    }

                    ToolTip.visible: launcherMouse.containsMouse
                    ToolTip.text: "Applications (Launchpad)"
                    ToolTip.delay: 300
                }

                // Separator after App Launcher
                Rectangle {
                    width: 1
                    height: Math.round(root.dockHeight * 0.45)
                    color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.15)
                    Layout.alignment: Qt.AlignVCenter
                    Layout.leftMargin: 2
                    Layout.rightMargin: 2
                }

                // 2. Application launcher & running window icons
                Repeater {
                    model: root.dockItems

                    delegate: DockIcon {
                        desktopId: modelData.desktop_id || ""
                        name: modelData.name || ""
                        iconPath: modelData.icon_path || ""
                        running: modelData.running || false
                        focused: modelData.focused || false
                        urgent: modelData.urgent || false
                        windows: modelData.windows || []
                        theme: root.theme
                        activeCount: modelData.active_count || 0
                        badgeCount: modelData.badge_count || 0
                        profile: root.profile
                        slotSize: root.slotDimension
                        iconSize: root.iconDimension
                        magnification: root.magnification
                        showIndicators: root.showIndicators

                        onClicked: {
                            dockRpc.sendAction("toggle", { desktop_id: modelData.desktop_id })
                        }
                        onMiddleClicked: {
                            dockRpc.sendAction("launch", { desktop_id: modelData.desktop_id })
                        }
                        onRightClicked: function(mx, my) {
                            appMenu.itemData = modelData
                            appMenu.popup(this, mx, my)
                        }
                        onFocusWindow: function(addr) {
                            dockRpc.sendAction("focus", { address: addr })
                        }
                        onCloseWindow: function(addr) {
                            dockRpc.sendAction("close", { address: addr })
                        }
                    }
                }

                // Separator before Quick Settings
                Rectangle {
                    width: 1
                    height: Math.round(root.dockHeight * 0.45)
                    color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.15)
                    Layout.alignment: Qt.AlignVCenter
                    Layout.leftMargin: 2
                    Layout.rightMargin: 2
                }

                // 3. Quick Settings Button
                Item {
                    id: settingsBtn
                    width: root.slotDimension
                    height: root.slotDimension
                    Layout.alignment: Qt.AlignVCenter

                    Rectangle {
                        id: settingsPlate
                        anchors.centerIn: parent
                        width: root.slotDimension - 8
                        height: root.slotDimension - 8
                        radius: Math.max(4, root.systemRounding - 4)
                        color: settingsMouse.containsMouse
                            ? ((root.theme && root.theme.accent) ? Qt.rgba(0.22, 0.74, 0.97, 0.25) : Qt.rgba(1, 1, 1, 0.18))
                            : Qt.rgba(1, 1, 1, 0.08)
                        border.color: settingsMouse.containsMouse
                            ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                            : Qt.rgba(1, 1, 1, 0.12)
                        border.width: 1

                        scale: settingsMouse.pressed ? 0.92 : (settingsMouse.containsMouse && root.magnification ? 1.18 : 1.0)
                        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }

                        transform: Translate {
                            y: (root.magnification && settingsMouse.containsMouse && !settingsMouse.pressed) ? -4 : 0
                            Behavior on y { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: "⚙"
                            font.pixelSize: Math.round(root.iconDimension * 0.55)
                            color: settingsMouse.containsMouse
                                ? ((root.theme && root.theme.accent) ? root.theme.accent : "#38bdf8")
                                : "#cbd5e1"
                        }
                    }

                    MouseArea {
                        id: settingsMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: {
                            settingsPopup.popup(settingsBtn, 0, -settingsPopup.height - 10)
                        }
                    }

                    ToolTip.visible: settingsMouse.containsMouse
                    ToolTip.text: "Dock Preferences"
                    ToolTip.delay: 300
                }
            }
        }

        // Context Menu for Window Management Actions & Pinning
        AppMenu {
            id: appMenu
            onActionTriggered: function(action, params) {
                dockRpc.sendAction(action, params)
            }
        }

        // macOS Style Dock Preferences Popup
        SettingsPopup {
            id: settingsPopup
            dockSize: root.dockSize
            magnification: root.magnification
            autoHide: root.autoHide
            showIndicators: root.showIndicators
            filterCurrentMonitor: root.filterMonitor
            systemRounding: root.systemRounding
            theme: root.theme

            onDockSizeChanged: function(size) {
                root.dockSize = size
                dockRpc.sendAction("size", { size: size })
            }
            onMagnificationToggled: function(enabled) {
                root.magnification = enabled
                dockRpc.sendAction("magnification", { enabled: enabled })
            }
            onAutoHideToggled: function(enabled) {
                root.autoHide = enabled
                dockRpc.sendAction("autohide", { enabled: enabled })
            }
            onShowIndicatorsToggled: function(enabled) {
                root.showIndicators = enabled
                dockRpc.sendAction("set_show_indicators", { show_indicators: enabled })
            }
            onFilterCurrentMonitorToggled: function(enabled) {
                root.filterMonitor = enabled
                if (enabled) {
                    var monId = root.shell?.screen?.id ?? 0
                    dockRpc.sendAction("monitor", { monitor_id: monId.toString() })
                } else {
                    dockRpc.sendAction("monitor", { monitor_id: "all" })
                }
            }
            onResetPinnedTriggered: function() {
                dockRpc.sendAction("reset", {})
            }
        }
    }

    // Process Bridge to dockd IPC client
    Item {
        id: dockRpc

        property string helperPath: {
            if (typeof Quickshell !== "undefined" && Quickshell.env("DOCKD_BIN")) {
                return Quickshell.env("DOCKD_BIN")
            }
            return "dockd"
        }

        function ensureDaemon() {
            if (!daemonCheckProcess.running) {
                daemonCheckProcess.command = ["sh", "-c", "pgrep -x dockd >/dev/null || (systemctl --user start dockd.socket 2>/dev/null || systemctl --user start dockd.service 2>/dev/null || ~/.local/bin/dockd --daemon 2>/dev/null || dockd --daemon 2>/dev/null) &"]
                daemonCheckProcess.running = true
            }
        }

        function launchAppDrawer() {
            drawerProcess.command = ["sh", "-c", "omarchy menu || walker || rofi -show drun || fuzzel || wofi --show drun || hyprctl dispatch exec walker"]
            drawerProcess.running = true
        }

        function sendAction(actionName, params) {
            ensureDaemon()
            var args = [actionName]
            if (params) {
                if (params.size) args.push(params.size)
                else if (params.desktop_id) args.push(params.desktop_id)
                else if (params.address) args.push(params.address)
                else if (params.target) args.push(params.target)
                else if (params.profile) args.push(params.profile)
                else if (params.enabled !== undefined) args.push(params.enabled ? "on" : "off")
                else if (params.monitor_id !== undefined) args.push(params.monitor_id.toString())
            }
            clientProcess.command = [dockRpc.helperPath].concat(args)
            clientProcess.running = true
        }

        function refreshState() {
            if (!stateProcess.running) {
                stateProcess.command = [dockRpc.helperPath, "state"]
                stateProcess.running = true
            }
            if (!roundingProc.running) {
                roundingProc.running = true
            }
        }

        // Periodic state sync timer (every 1 second or on events)
        Timer {
            interval: 1000
            running: true
            repeat: true
            onTriggered: dockRpc.refreshState()
        }

        Process {
            id: daemonCheckProcess
        }

        Process {
            id: drawerProcess
        }

        Process {
            id: clientProcess
            stdout: StdioCollector {
                onTextChanged: dockRpc.refreshState()
            }
        }

        Process {
            id: stateProcess
            stdout: StdioCollector {
                onTextChanged: {
                    try {
                        var parsed = JSON.parse(text)
                        if (parsed.items && parsed.items.length > 0) {
                            root.dockItems = parsed.items
                        }
                        if (parsed.profile) {
                            root.profile = parsed.profile
                        }
                        if (parsed.dock_size) {
                            root.dockSize = parsed.dock_size
                        }
                        if (parsed.magnification !== undefined) {
                            root.magnification = parsed.magnification
                        }
                        if (parsed.show_indicators !== undefined) {
                            root.showIndicators = parsed.show_indicators
                        }
                        if (parsed.active_address) {
                            root.activeAddress = parsed.active_address
                        }
                        if (parsed.overlap !== undefined) {
                            root.overlap = parsed.overlap
                        }
                        if (parsed.has_windows !== undefined) {
                            root.hasWindows = parsed.has_windows
                        }
                        if (parsed.autohide !== undefined) {
                            root.autoHide = parsed.autohide
                        }
                        if (parsed.theme) {
                            root.theme = parsed.theme
                        }
                    } catch (e) {
                        // ignore partial read
                    }
                }
            }
        }
    }

    Component.onCompleted: {
        dockRpc.ensureDaemon()
        dockRpc.refreshState()
        roundingProc.running = true
    }
}
