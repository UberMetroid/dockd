//! Central state management for dock launchers, layout profiles, and active windows.
//!
//! Synchronizes Hyprland client updates with pinned applications, theme, and urgency tracking.

use super::dock_item::{DockItem, WindowSummary};
use super::pinned_store::{load_pinned_ids, reorder_pinned_ids, save_pinned_ids};
use crate::hyprland::client_table::{HyprClient, normalize_addr};
use crate::syntax::json_value::JsonValue;
use crate::system::resolve_icon::resolve_icon_path;
use crate::system::scan_desktop::{DesktopEntry, scan_desktop_entries};
use crate::system::theme_sync::{ThemePalette, load_omarchy_theme};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct DockState {
    pub desktop_entries: Vec<DesktopEntry>,
    pub pinned_ids: Vec<String>,
    pub profile: String,
    pub items: Vec<DockItem>,
    pub active_address: Option<String>,
    pub urgent_addresses: BTreeSet<String>,
    pub filter_monitor: Option<i64>,
    pub overlap: bool,
    pub has_windows: bool,
    pub autohide: bool,
    pub theme: ThemePalette,
}

impl Default for DockState {
    fn default() -> Self {
        Self::new()
    }
}

impl DockState {
    pub fn new() -> Self {
        let mut state = Self {
            desktop_entries: scan_desktop_entries(),
            pinned_ids: load_pinned_ids(),
            profile: "general".to_string(),
            items: Vec::new(),
            active_address: None,
            urgent_addresses: BTreeSet::new(),
            filter_monitor: None,
            overlap: false,
            has_windows: false,
            autohide: true,
            theme: load_omarchy_theme(),
        };
        state.rebuild_items(&[], None);
        state
    }

    pub fn rebuild_items(&mut self, clients: &[HyprClient], active_addr: Option<&str>) {
        self.active_address = active_addr.map(|s| s.to_string());
        if let Some(active) = active_addr {
            self.clear_urgent(active);
        }
        self.has_windows = clients
            .iter()
            .any(|c| c.mapped && !c.hidden && !c.is_minimized());

        let mut new_items = Vec::new();
        let mut matched_clients = BTreeSet::new();

        // 1. Pinned items
        for id in &self.pinned_ids {
            let entry = self.desktop_entries.iter().find(|e| &e.id == id);
            let name = entry.map(|e| e.name.clone()).unwrap_or_else(|| id.clone());
            let exec_cmd = entry.map(|e| e.exec.clone()).unwrap_or_default();
            let icon_raw = entry.map(|e| e.icon.as_str()).unwrap_or(id.as_str());
            let icon_path = resolve_icon_path(icon_raw).unwrap_or_default();
            let wm_class = entry.and_then(|e| e.wm_class.clone()).unwrap_or_default();

            let mut windows = Vec::new();
            for (idx, client) in clients.iter().enumerate() {
                if self.filter_monitor.is_some_and(|m| client.monitor_id != m) {
                    continue;
                }
                let matches = (!wm_class.is_empty() && client.matches_app(&wm_class))
                    || client.matches_app(id)
                    || (!name.is_empty() && client.matches_app(&name));
                if matches {
                    matched_clients.insert(idx);
                    let urg = self.is_address_urgent(&client.address);
                    windows.push(WindowSummary::from_client(client, active_addr, urg));
                }
            }

            let is_urgent = windows.iter().any(|w| w.urgent);
            new_items.push(DockItem {
                desktop_id: id.clone(),
                name,
                icon_path,
                exec_cmd,
                wm_class,
                pinned: true,
                urgent: is_urgent,
                windows,
                badge_count: 0,
            });
        }

        // 2. Running unpinned items
        for (idx, client) in clients.iter().enumerate() {
            if matched_clients.contains(&idx)
                || self.filter_monitor.is_some_and(|m| client.monitor_id != m)
            {
                continue;
            }

            let entry = self.desktop_entries.iter().find(|e| {
                e.wm_class.as_deref().is_some_and(|c| client.matches_app(c))
                    || client.matches_app(&e.id)
                    || client.matches_app(&e.name)
            });

            let id = entry
                .map(|e| e.id.clone())
                .unwrap_or_else(|| client.class.clone());
            let name = entry
                .map(|e| e.name.clone())
                .unwrap_or_else(|| client.class.clone());
            let exec_cmd = entry.map(|e| e.exec.clone()).unwrap_or_default();
            let icon_raw = entry.map(|e| e.icon.as_str()).unwrap_or(&client.class);
            let icon_path = resolve_icon_path(icon_raw).unwrap_or_default();
            let wm_class = client.class.clone();

            let urg = self.is_address_urgent(&client.address);
            let win = WindowSummary::from_client(client, active_addr, urg);

            if let Some(existing) = new_items
                .iter_mut()
                .find(|it| it.wm_class == wm_class || it.desktop_id == id)
            {
                if win.urgent {
                    existing.urgent = true;
                }
                existing.windows.push(win);
            } else {
                new_items.push(DockItem {
                    desktop_id: id,
                    name,
                    icon_path,
                    exec_cmd,
                    wm_class,
                    pinned: false,
                    urgent: urg,
                    windows: vec![win],
                    badge_count: 0,
                });
            }
        }

        self.items = new_items;
    }

    pub fn pin(&mut self, desktop_id: &str) -> bool {
        if self.pinned_ids.iter().any(|id| id == desktop_id) {
            return false;
        }
        self.pinned_ids.push(desktop_id.to_string());
        save_pinned_ids(&self.pinned_ids);
        true
    }

    pub fn unpin(&mut self, desktop_id: &str) -> bool {
        let initial_len = self.pinned_ids.len();
        self.pinned_ids.retain(|id| id != desktop_id);
        let changed = self.pinned_ids.len() != initial_len;
        if changed {
            save_pinned_ids(&self.pinned_ids);
        }
        changed
    }

    pub fn reorder_pin(&mut self, desktop_id: &str, target_idx: usize) -> bool {
        let changed = reorder_pinned_ids(&mut self.pinned_ids, desktop_id, target_idx);
        if changed {
            save_pinned_ids(&self.pinned_ids);
        }
        changed
    }

    pub fn mark_urgent(&mut self, addr: &str) {
        self.urgent_addresses.insert(normalize_addr(addr));
    }
    pub fn clear_urgent(&mut self, addr: &str) {
        self.urgent_addresses.remove(&normalize_addr(addr));
    }
    pub fn clear_all_urgent(&mut self) {
        self.urgent_addresses.clear();
    }
    pub fn is_address_urgent(&self, addr: &str) -> bool {
        self.urgent_addresses.contains(&normalize_addr(addr))
    }

    pub fn set_filter_monitor(&mut self, monitor_id: Option<i64>) {
        self.filter_monitor = monitor_id;
    }
    pub fn set_profile(&mut self, profile: &str) {
        self.profile = profile.to_string();
    }
    pub fn set_autohide(&mut self, enabled: bool) {
        self.autohide = enabled;
    }
    pub fn sync_theme(&mut self) {
        self.theme = load_omarchy_theme();
    }

    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert("profile".into(), JsonValue::String(self.profile.clone()));
        let active = self
            .active_address
            .as_ref()
            .map_or(JsonValue::Null, |a| JsonValue::String(a.clone()));
        map.insert("active_address".into(), active);
        let mon = self
            .filter_monitor
            .map_or(JsonValue::Null, |m| JsonValue::Number(m as f64));
        map.insert("filter_monitor".into(), mon);
        map.insert("overlap".into(), JsonValue::Bool(self.overlap));
        map.insert("has_windows".into(), JsonValue::Bool(self.has_windows));
        map.insert("autohide".into(), JsonValue::Bool(self.autohide));
        map.insert("theme".into(), self.theme.to_json());
        let items: Vec<JsonValue> = self.items.iter().map(DockItem::to_json).collect();
        map.insert("items".into(), JsonValue::Array(items));
        JsonValue::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_tracking() {
        let mut state = DockState::new();
        state.mark_urgent("0x559e2a80");
        assert!(state.is_address_urgent("0x559e2a80"));
        assert!(state.is_address_urgent("559e2a80"));
        state.clear_urgent("559e2a80");
        assert!(!state.is_address_urgent("0x559e2a80"));
    }

    #[test]
    fn test_filter_monitor() {
        let mut state = DockState::new();
        assert_eq!(state.filter_monitor, None);
        state.set_filter_monitor(Some(1));
        assert_eq!(state.filter_monitor, Some(1));
    }
}
