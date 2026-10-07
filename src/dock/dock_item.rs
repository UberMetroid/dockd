//! Data model for dock launcher items and window descriptors.
//!
//! Encapsulates launcher identity, running window instances, badges, urgency, and layout metadata.

use crate::hyprland::client_table::{HyprClient, normalize_addr};
use crate::syntax::json_value::JsonValue;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct WindowSummary {
    pub address: String,
    pub title: String,
    pub workspace_id: i64,
    pub workspace_name: String,
    pub minimized: bool,
    pub floating: bool,
    pub focused: bool,
    pub urgent: bool,
    pub pid: u64,
}

impl WindowSummary {
    pub fn from_client(client: &HyprClient, active_addr: Option<&str>, is_urgent: bool) -> Self {
        let is_focused =
            active_addr.is_some_and(|a| normalize_addr(a) == normalize_addr(&client.address));
        Self {
            address: client.address.clone(),
            title: client.title.clone(),
            workspace_id: client.workspace_id,
            workspace_name: client.workspace_name.clone(),
            minimized: client.is_minimized(),
            floating: client.floating,
            focused: is_focused,
            urgent: is_urgent,
            pid: client.pid,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert(
            "address".to_string(),
            JsonValue::String(self.address.clone()),
        );
        map.insert("title".to_string(), JsonValue::String(self.title.clone()));
        map.insert(
            "workspace_id".to_string(),
            JsonValue::Number(self.workspace_id as f64),
        );
        map.insert(
            "workspace_name".to_string(),
            JsonValue::String(self.workspace_name.clone()),
        );
        map.insert("minimized".to_string(), JsonValue::Bool(self.minimized));
        map.insert("floating".to_string(), JsonValue::Bool(self.floating));
        map.insert("focused".to_string(), JsonValue::Bool(self.focused));
        map.insert("urgent".to_string(), JsonValue::Bool(self.urgent));
        map.insert("pid".to_string(), JsonValue::Number(self.pid as f64));
        JsonValue::Object(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DockItem {
    pub desktop_id: String,
    pub name: String,
    pub icon_path: String,
    pub exec_cmd: String,
    pub wm_class: String,
    pub pinned: bool,
    pub urgent: bool,
    pub windows: Vec<WindowSummary>,
    pub badge_count: u32,
}

impl DockItem {
    pub fn new_pinned(
        id: String,
        name: String,
        icon: String,
        exec: String,
        wm_class: String,
        windows: Vec<WindowSummary>,
    ) -> Self {
        let urgent = windows.iter().any(|w| w.urgent);
        Self {
            desktop_id: id,
            name,
            icon_path: icon,
            exec_cmd: exec,
            wm_class,
            pinned: true,
            urgent,
            windows,
            badge_count: 0,
        }
    }

    pub fn new_unpinned(
        id: String,
        name: String,
        icon: String,
        exec: String,
        wm_class: String,
        urgent: bool,
        windows: Vec<WindowSummary>,
    ) -> Self {
        Self {
            desktop_id: id,
            name,
            icon_path: icon,
            exec_cmd: exec,
            wm_class,
            pinned: false,
            urgent,
            windows,
            badge_count: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn match_windows<F: Fn(&str) -> bool>(
        clients: &[HyprClient],
        wm_class: &str,
        id: &str,
        name: &str,
        mon: Option<i64>,
        active: Option<&str>,
        is_urgent: F,
        matched: &mut BTreeSet<usize>,
    ) -> Vec<WindowSummary> {
        let mut wins = Vec::new();
        for (idx, client) in clients.iter().enumerate() {
            if mon.is_some_and(|m| client.monitor_id != m) {
                continue;
            }
            let matches = (!wm_class.is_empty() && client.matches_app(wm_class))
                || client.matches_app(id)
                || (!name.is_empty() && client.matches_app(name));
            if matches {
                matched.insert(idx);
                let urg = is_urgent(&client.address);
                wins.push(WindowSummary::from_client(client, active, urg));
            }
        }
        wins
    }

    pub fn is_running(&self) -> bool {
        !self.windows.is_empty()
    }

    pub fn has_focused_window(&self) -> bool {
        self.windows.iter().any(|w| w.focused)
    }

    pub fn is_urgent(&self) -> bool {
        self.urgent || self.windows.iter().any(|w| w.urgent)
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert(
            "desktop_id".to_string(),
            JsonValue::String(self.desktop_id.clone()),
        );
        map.insert("name".to_string(), JsonValue::String(self.name.clone()));
        map.insert(
            "icon_path".to_string(),
            JsonValue::String(self.icon_path.clone()),
        );
        map.insert(
            "exec_cmd".to_string(),
            JsonValue::String(self.exec_cmd.clone()),
        );
        map.insert(
            "wm_class".to_string(),
            JsonValue::String(self.wm_class.clone()),
        );
        map.insert("pinned".to_string(), JsonValue::Bool(self.pinned));
        map.insert("running".to_string(), JsonValue::Bool(self.is_running()));
        map.insert(
            "focused".to_string(),
            JsonValue::Bool(self.has_focused_window()),
        );
        map.insert("urgent".to_string(), JsonValue::Bool(self.is_urgent()));
        map.insert(
            "active_count".to_string(),
            JsonValue::Number(self.windows.len() as f64),
        );
        map.insert(
            "badge_count".to_string(),
            JsonValue::Number(self.badge_count as f64),
        );

        let win_json: Vec<JsonValue> = self.windows.iter().map(|w| w.to_json()).collect();
        map.insert("windows".to_string(), JsonValue::Array(win_json));

        JsonValue::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_item_serialization() {
        let win = WindowSummary {
            address: "0x1234".to_string(),
            title: "Terminal".to_string(),
            workspace_id: 1,
            workspace_name: "1".to_string(),
            minimized: false,
            floating: false,
            focused: true,
            urgent: false,
            pid: 1234,
        };
        let item = DockItem {
            desktop_id: "org.wezfurlong.wezterm.desktop".to_string(),
            name: "WezTerm".to_string(),
            icon_path: "/usr/share/icons/wezterm.png".to_string(),
            exec_cmd: "wezterm".to_string(),
            wm_class: "org.wezfurlong.wezterm".to_string(),
            pinned: true,
            urgent: false,
            windows: vec![win],
            badge_count: 0,
        };
        assert!(item.is_running());
        assert!(item.has_focused_window());
        assert!(!item.is_urgent());
        let json = item.to_json();
        assert_eq!(json.get_str("name"), Some("WezTerm"));
        assert_eq!(json.get_bool("running"), Some(true));
        assert_eq!(json.get_bool("focused"), Some(true));
        assert_eq!(json.get_bool("urgent"), Some(false));
    }
}
