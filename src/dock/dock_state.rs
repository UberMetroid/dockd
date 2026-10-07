//! Central state management for dock launchers, layout profiles, and active windows.
use super::dock_item::{DockItem, WindowSummary};
use super::pinned_store::{default_pinned, load_pinned_ids, reorder_pinned_ids, save_pinned_ids};
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
    pub dock_size: String,
    pub magnification: bool,
    pub show_indicators: bool,
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
            profile: "mac".to_string(),
            dock_size: "medium".to_string(),
            magnification: true,
            show_indicators: true,
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

            let windows = DockItem::match_windows(
                clients,
                &wm_class,
                id,
                &name,
                self.filter_monitor,
                active_addr,
                |a| self.is_address_urgent(a),
                &mut matched_clients,
            );

            new_items.push(DockItem::new_pinned(
                id.clone(),
                name,
                icon_path,
                exec_cmd,
                wm_class,
                windows,
            ));
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
                new_items.push(DockItem::new_unpinned(
                    id,
                    name,
                    icon_path,
                    exec_cmd,
                    wm_class,
                    urg,
                    vec![win],
                ));
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
    #[rustfmt::skip]
    pub fn clear_all_urgent(&mut self) { self.urgent_addresses.clear(); }
    pub fn is_address_urgent(&self, addr: &str) -> bool {
        self.urgent_addresses.contains(&normalize_addr(addr))
    }

    #[rustfmt::skip]
    pub fn set_filter_monitor(&mut self, m: Option<i64>) { self.filter_monitor = m; }
    #[rustfmt::skip]
    pub fn set_profile(&mut self, p: &str) { self.profile = p.to_string(); }
    #[rustfmt::skip]
    pub fn set_autohide(&mut self, a: bool) { self.autohide = a; }
    #[rustfmt::skip]
    pub fn set_dock_size(&mut self, s: &str) { self.dock_size = s.to_string(); }
    #[rustfmt::skip]
    pub fn set_magnification(&mut self, m: bool) { self.magnification = m; }
    #[rustfmt::skip]
    pub fn set_show_indicators(&mut self, i: bool) { self.show_indicators = i; }
    #[rustfmt::skip]
    pub fn sync_theme(&mut self) { self.theme = load_omarchy_theme(); }

    #[rustfmt::skip]
    pub fn reset_pinned(&mut self) { self.pinned_ids = default_pinned(); save_pinned_ids(&self.pinned_ids); }

    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert("profile".into(), JsonValue::String(self.profile.clone()));
        map.insert(
            "dock_size".into(),
            JsonValue::String(self.dock_size.clone()),
        );
        map.insert("magnification".into(), JsonValue::Bool(self.magnification));
        map.insert(
            "show_indicators".into(),
            JsonValue::Bool(self.show_indicators),
        );
        let active = self
            .active_address
            .as_deref()
            .map_or(JsonValue::Null, |a| JsonValue::String(a.to_string()));
        let mon = self
            .filter_monitor
            .map_or(JsonValue::Null, |m| JsonValue::Number(m as f64));
        map.insert("active_address".into(), active);
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
    fn test_dock_state_features() {
        let mut state = DockState::new();
        state.mark_urgent("0x559e2a80");
        assert!(state.is_address_urgent("0x559e2a80") && state.is_address_urgent("559e2a80"));
        state.clear_urgent("559e2a80");
        assert!(!state.is_address_urgent("0x559e2a80"));
        assert_eq!(state.filter_monitor, None);
        state.set_filter_monitor(Some(1));
        assert_eq!(state.filter_monitor, Some(1));
    }
}
