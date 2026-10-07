import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "components"

PanelWindow {
    id: root

    // Host properties injected by Omarchy plugin loader
    property var shell: null
    property var manifest: null

    // Dock configuration state
    property string profile: "general"
    property bool fileShortcuts: true
    property bool autoHide: false
    property bool dockVisible: true
    property var dockItems: []
    property string activeAddress: ""
    property bool overlap: false
    property bool filterMonitor: false
    property var theme: null

    // Layer-shell geometry
    anchors.bottom: true
    anchors.horizontalCenter: true
    margins.bottom: profile === "windows" ? 0 : 8
    exclusiveZone: profile === "windows" ? implicitHeight : -1

    implicitWidth: dockBar.implicitWidth + 24
    implicitHeight: 56

    color: "transparent"

    readonly property bool isDockHidden: root.autoHide && root.overlap && !dockHoverArea.containsMouse

    // Background pill/taskbar styling
    Rectangle {
        id: dockBar
        anchors.centerIn: parent
        height: 52
        width: contentRow.implicitWidth + 16

        radius: root.profile === "windows" ? 0 : 16
        color: root.profile === "windows"
            ? ((root.theme && root.theme.background) ? root.theme.background : "#0f172a")
            : ((root.theme && root.theme.background) ? Qt.rgba(0.06, 0.09, 0.16, 0.88) : Qt.rgba(0.06, 0.09, 0.16, 0.85))
        border.color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.12)
        border.width: root.profile === "windows" ? 0 : 1

        opacity: root.isDockHidden ? 0.2 : 1.0
        transform: Translate {
            y: root.isDockHidden ? 44 : 0
            Behavior on y {
                NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
            }
        }
        Behavior on opacity {
            NumberAnimation { duration: 180 }
        }

        MouseArea {
            id: dockHoverArea
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }

        RowLayout {
            id: contentRow
            anchors.centerIn: parent
            spacing: root.profile === "mac" ? 8 : 4

            // Application launcher icons
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

            // Separator before file shortcuts
            Rectangle {
                visible: root.fileShortcuts && root.dockItems.length > 0
                width: 1
                height: 28
                color: (root.theme && root.theme.border) ? root.theme.border : Qt.rgba(1, 1, 1, 0.15)
                Layout.alignment: Qt.AlignVCenter
                Layout.leftMargin: 4
                Layout.rightMargin: 4
            }

            // File shortcut: Home
            DockIcon {
                visible: root.fileShortcuts
                name: "Home"
                iconPath: "/usr/share/icons/hicolor/scalable/apps/user-home.svg"
                profile: root.profile
                theme: root.theme
                onClicked: dockRpc.sendAction("open_location", { target: "home" })
            }

            // File shortcut: Downloads
            DockIcon {
                visible: root.fileShortcuts
                name: "Downloads"
                iconPath: "/usr/share/icons/hicolor/scalable/apps/folder-download.svg"
                profile: root.profile
                theme: root.theme
                onClicked: dockRpc.sendAction("open_location", { target: "downloads" })
            }

            // File shortcut: Trash
            DockIcon {
                visible: root.fileShortcuts
                name: "Trash"
                iconPath: "/usr/share/icons/hicolor/scalable/apps/user-trash.svg"
                profile: root.profile
                theme: root.theme
                onClicked: dockRpc.sendAction("open_location", { target: "trash" })
            }
        }
    }

    // Context Menu for Window Management Actions
    AppMenu {
        id: appMenu
        onActionTriggered: function(action, params) {
            dockRpc.sendAction(action, params)
        }
    }

    // Settings Popup
    SettingsPopup {
        id: settingsPopup
        currentProfile: root.profile
        showFileShortcuts: root.fileShortcuts
        autoHide: root.autoHide
        filterCurrentMonitor: root.filterMonitor
        onProfileChanged: function(p) {
            root.profile = p
            dockRpc.sendAction("profile", { profile: p })
        }
        onFileShortcutsToggled: function(enabled) {
            root.fileShortcuts = enabled
        }
        onAutoHideToggled: function(enabled) {
            root.autoHide = enabled
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
    }

    // Process Bridge to dockd IPC client
    Item {
        id: dockRpc

        property string helperPath: (typeof Quickshell !== "undefined" && Quickshell.env("DOCKD_BIN")) ? Quickshell.env("DOCKD_BIN") : "dockd"
        property var pendingActions: []

        function sendAction(actionName, params) {
            var args = [actionName]
            if (params) {
                if (params.desktop_id) args.push(params.desktop_id)
                else if (params.address) args.push(params.address)
                else if (params.target) args.push(params.target)
                else if (params.profile) args.push(params.profile)
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
        }

        // Periodic state sync timer (every 1 second or on events)
        Timer {
            interval: 1000
            running: true
            repeat: true
            onTriggered: dockRpc.refreshState()
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
                        if (parsed.items) {
                            root.dockItems = parsed.items
                        }
                        if (parsed.profile) {
                            root.profile = parsed.profile
                        }
                        if (parsed.active_address) {
                            root.activeAddress = parsed.active_address
                        }
                        if (parsed.overlap !== undefined) {
                            root.overlap = parsed.overlap
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
        dockRpc.refreshState()
    }
}
