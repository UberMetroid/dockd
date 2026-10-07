import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Menu {
    id: root

    property var itemData: null
    property int systemRounding: 12
    signal actionTriggered(string action, var params)

    title: itemData ? itemData.name : "Application"

    // Section 1: Open Windows (if running)
    Instantiator {
        model: (root.itemData && root.itemData.windows) ? root.itemData.windows : []
        delegate: Menu {
            title: (modelData.minimized ? "[Min] " : "") + (modelData.title || "Window")

            MenuItem {
                text: "Focus / Restore"
                onTriggered: root.actionTriggered("focus", { address: modelData.address })
            }
            MenuItem {
                text: "Minimize"
                enabled: !modelData.minimized
                onTriggered: root.actionTriggered("minimize", { address: modelData.address })
            }
            MenuSeparator {}
            MenuItem {
                text: "Snap Left"
                onTriggered: root.actionTriggered("snap_left", { address: modelData.address })
            }
            MenuItem {
                text: "Snap Right"
                onTriggered: root.actionTriggered("snap_right", { address: modelData.address })
            }
            MenuItem {
                text: modelData.floating ? "Tile Window" : "Float Window"
                onTriggered: root.actionTriggered(modelData.floating ? "tile" : "float", { address: modelData.address })
            }
            MenuSeparator {}
            MenuItem {
                text: "Close Window"
                onTriggered: root.actionTriggered("close", { address: modelData.address })
            }
        }
        onObjectAdded: function(index, object) { root.insertMenu(index, object) }
        onObjectRemoved: function(index, object) { root.removeMenu(object) }
    }

    MenuSeparator {
        visible: root.itemData && root.itemData.windows && root.itemData.windows.length > 0
    }

    // Section 2: Standard App Actions
    MenuItem {
        text: "New Window"
        onTriggered: root.actionTriggered("launch", { desktop_id: root.itemData.desktop_id })
    }

    MenuItem {
        text: (root.itemData && root.itemData.pinned) ? "Unpin from Dock" : "Pin to Dock"
        onTriggered: {
            if (root.itemData.pinned) {
                root.actionTriggered("unpin", { desktop_id: root.itemData.desktop_id })
            } else {
                root.actionTriggered("pin", { desktop_id: root.itemData.desktop_id })
            }
        }
    }

    MenuSeparator {
        visible: root.itemData && root.itemData.windows && root.itemData.windows.length > 0
    }

    MenuItem {
        text: "Quit Application"
        visible: root.itemData && root.itemData.windows && root.itemData.windows.length > 0
        onTriggered: {
            if (root.itemData && root.itemData.windows) {
                for (var i = 0; i < root.itemData.windows.length; i++) {
                    root.actionTriggered("close", { address: root.itemData.windows[i].address })
                }
            }
        }
    }
}
