//! Persistence and ordering for pinned dock application launchers.
//!
//! Manages loading, saving, and reordering pinned .desktop entries to ~/.config/omarchy.

use crate::syntax::json_parse::parse;
use crate::syntax::json_render::render;
use crate::syntax::json_value::JsonValue;
use std::fs;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
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

pub fn default_pinned() -> Vec<String> {
    vec![
        "firefox.desktop".to_string(),
        "org.wezfurlong.wezterm.desktop".to_string(),
        "nautilus.desktop".to_string(),
        "code.desktop".to_string(),
    ]
}

pub fn load_pinned_ids() -> Vec<String> {
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

pub fn save_pinned_ids(ids: &[String]) {
    let p = config_path();
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let arr = JsonValue::Array(ids.iter().map(|s| JsonValue::String(s.clone())).collect());
    let _ = fs::write(p, render(&arr));
}

pub fn reorder_pinned_ids(ids: &mut Vec<String>, desktop_id: &str, target_idx: usize) -> bool {
    let current_pos = ids.iter().position(|id| id == desktop_id);
    let Some(old_idx) = current_pos else {
        return false;
    };

    let item = ids.remove(old_idx);
    let clamped_idx = target_idx.min(ids.len());
    ids.insert(clamped_idx, item);
    save_pinned_ids(ids);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reorder_pinned_ids() {
        let mut ids = vec![
            "firefox.desktop".to_string(),
            "term.desktop".to_string(),
            "files.desktop".to_string(),
        ];

        // Move term to first position
        assert!(reorder_pinned_ids(&mut ids, "term.desktop", 0));
        assert_eq!(ids[0], "term.desktop");
        assert_eq!(ids[1], "firefox.desktop");
        assert_eq!(ids[2], "files.desktop");

        // Move to beyond length (clamps to end)
        assert!(reorder_pinned_ids(&mut ids, "term.desktop", 99));
        assert_eq!(ids[2], "term.desktop");

        // Unknown id fails
        assert!(!reorder_pinned_ids(&mut ids, "unknown.desktop", 0));
    }
}
