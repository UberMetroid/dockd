//! Central state management for dock launchers, layout profiles, and active windows.
//!
//! Synchronizes Hyprland client updates with pinned applications and persisted configurations.

use super::dock_item::{DockItem, WindowSummary};
use crate::hyprland::client_table::HyprClient;
use crate::syntax::json_parse::parse;
use crate::syntax::json_render::render;
use crate::syntax::json_value::JsonValue;
use crate::system::resolve_icon::resolve_icon_path;
use crate::system::scan_desktop::{DesktopEntry, scan_desktop_entries};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DockState {
    pub desktop_entries: Vec<DesktopEntry>,
    pub pinned_ids: Vec<String>,
    pub profile: String,
    pub items: Vec<DockItem>,
    pub active_address: Option<String>,
}

impl Default for DockState {
    fn default() -> Self {
        Self::new()
    }
}

impl DockState {
    pub fn new() -> Self {
        let desktop_entries = scan_desktop_entries();
        let pinned_ids = load_pinned_ids();
        let mut state = Self {
            desktop_entries,
            pinned_ids,
            profile: "general".to_string(),
            items: Vec::new(),
            active_address: None,
        };
        state.rebuild_items(&[], None);
        state
    }

    pub fn rebuild_items(&mut self, clients: &[HyprClient], active_addr: Option<&str>) {
        self.active_address = active_addr.map(|s| s.to_string());
        let mut new_items = Vec::new();
        let mut matched_clients = std::collections::BTreeSet::new();

        // Add pinned items first
        for id in &self.pinned_ids {
            let entry = self.desktop_entries.iter().find(|e| &e.id == id);
            let name = entry.map(|e| e.name.clone()).unwrap_or_else(|| id.clone());
            let exec_cmd = entry.map(|e| e.exec.clone()).unwrap_or_default();
            let icon_raw = entry.map(|e| e.icon.as_str()).unwrap_or(id.as_str());
            let icon_path = resolve_icon_path(icon_raw).unwrap_or_default();
            let wm_class = entry.and_then(|e| e.wm_class.clone()).unwrap_or_default();

            let mut windows = Vec::new();
            for (idx, client) in clients.iter().enumerate() {
                let matches = (!wm_class.is_empty() && client.matches_app(&wm_class))
                    || client.matches_app(id)
                    || (!name.is_empty() && client.matches_app(&name));
                if matches {
                    matched_clients.insert(idx);
                    windows.push(summarize_window(client, active_addr));
                }
            }

            new_items.push(DockItem {
                desktop_id: id.clone(),
                name,
                icon_path,
                exec_cmd,
                wm_class,
                pinned: true,
                windows,
                badge_count: 0,
            });
        }

        // Add unpinned running clients
        for (idx, client) in clients.iter().enumerate() {
            if matched_clients.contains(&idx) {
                continue;
            }
            let matching_entry = self.desktop_entries.iter().find(|e| {
                e.wm_class.as_deref().is_some_and(|c| client.matches_app(c))
                    || client.matches_app(&e.id)
                    || client.matches_app(&e.name)
            });

            let id = matching_entry
                .map(|e| e.id.clone())
                .unwrap_or_else(|| client.class.clone());
            let name = matching_entry
                .map(|e| e.name.clone())
                .unwrap_or_else(|| client.class.clone());
            let exec_cmd = matching_entry.map(|e| e.exec.clone()).unwrap_or_default();
            let icon_raw = matching_entry
                .map(|e| e.icon.as_str())
                .unwrap_or(&client.class);
            let icon_path = resolve_icon_path(icon_raw).unwrap_or_default();
            let wm_class = client.class.clone();

            let win_summary = summarize_window(client, active_addr);

            // Group into existing item if class matches
            if let Some(existing) = new_items
                .iter_mut()
                .find(|it| it.wm_class == wm_class || it.desktop_id == id)
            {
                existing.windows.push(win_summary);
            } else {
                new_items.push(DockItem {
                    desktop_id: id,
                    name,
                    icon_path,
                    exec_cmd,
                    wm_class,
                    pinned: false,
                    windows: vec![win_summary],
                    badge_count: 0,
                });
            }
        }

        self.items = new_items;
    }

    pub fn pin(&mut self, desktop_id: &str) -> bool {
        if !self.pinned_ids.iter().any(|id| id == desktop_id) {
            self.pinned_ids.push(desktop_id.to_string());
            save_pinned_ids(&self.pinned_ids);
            true
        } else {
            false
        }
    }

    pub fn unpin(&mut self, desktop_id: &str) -> bool {
        let initial_len = self.pinned_ids.len();
        self.pinned_ids.retain(|id| id != desktop_id);
        if self.pinned_ids.len() != initial_len {
            save_pinned_ids(&self.pinned_ids);
            true
        } else {
            false
        }
    }

    pub fn set_profile(&mut self, profile: &str) {
        self.profile = profile.to_string();
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert(
            "profile".to_string(),
            JsonValue::String(self.profile.clone()),
        );
        let active = match &self.active_address {
            Some(a) => JsonValue::String(a.clone()),
            None => JsonValue::Null,
        };
        map.insert("active_address".to_string(), active);

        let items_json: Vec<JsonValue> = self.items.iter().map(DockItem::to_json).collect();
        map.insert("items".to_string(), JsonValue::Array(items_json));

        JsonValue::Object(map)
    }
}

fn summarize_window(client: &HyprClient, active_addr: Option<&str>) -> WindowSummary {
    let is_focused = active_addr.is_some_and(|a| a == client.address);
    WindowSummary {
        address: client.address.clone(),
        title: client.title.clone(),
        workspace_id: client.workspace_id,
        workspace_name: client.workspace_name.clone(),
        minimized: client.is_minimized(),
        floating: client.floating,
        focused: is_focused,
        pid: client.pid,
    }
}

fn config_path() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home)
            .join("omarchy")
            .join("dockd-pinned.json")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".config")
            .join("omarchy")
            .join("dockd-pinned.json")
    } else {
        PathBuf::from("/tmp/dockd-pinned.json")
    }
}

fn load_pinned_ids() -> Vec<String> {
    let p = config_path();
    let Ok(txt) = fs::read_to_string(p) else {
        return default_pinned();
    };
    if let Ok(JsonValue::Array(arr)) = parse(&txt) {
        let ids: Vec<String> = arr
            .iter()
            .filter_map(JsonValue::as_str)
            .map(String::from)
            .collect();
        if !ids.is_empty() {
            return ids;
        }
    }
    default_pinned()
}

fn default_pinned() -> Vec<String> {
    vec![
        "firefox.desktop".to_string(),
        "org.wezfurlong.wezterm.desktop".to_string(),
        "nautilus.desktop".to_string(),
        "code.desktop".to_string(),
    ]
}

fn save_pinned_ids(ids: &[String]) {
    let p = config_path();
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let arr = JsonValue::Array(ids.iter().map(|s| JsonValue::String(s.clone())).collect());
    let _ = fs::write(p, render(&arr));
}
