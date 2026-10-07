//! Window client tracking and metadata representation.
//!
//! Parses client window descriptors from Hyprland JSON payloads.

use crate::syntax::json_value::JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub struct HyprClient {
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub at: (i64, i64),
    pub size: (i64, i64),
    pub workspace_id: i64,
    pub workspace_name: String,
    pub monitor_id: i64,
    pub class: String,
    pub title: String,
    pub initial_class: String,
    pub initial_title: String,
    pub pid: u64,
    pub xwayland: bool,
    pub floating: bool,
    pub fullscreen: bool,
}

impl HyprClient {
    pub fn from_json(val: &JsonValue) -> Option<Self> {
        let address = val.get_str("address")?.to_string();
        let mapped = val.get_bool("mapped").unwrap_or(true);
        let hidden = val.get_bool("hidden").unwrap_or(false);
        let workspace_id = val
            .get("workspace")
            .and_then(|w| w.get_i64("id"))
            .or_else(|| val.get_i64("workspace"))
            .unwrap_or(0);
        let workspace_name = val
            .get("workspace")
            .and_then(|w| w.get_str("name"))
            .unwrap_or("")
            .to_string();
        let monitor_id = val.get_i64("monitor").unwrap_or(0);
        let class = val.get_str("class").unwrap_or("").to_string();
        let title = val.get_str("title").unwrap_or("").to_string();
        let initial_class = val.get_str("initialClass").unwrap_or(&class).to_string();
        let initial_title = val.get_str("initialTitle").unwrap_or(&title).to_string();
        let pid = val.get_u64("pid").unwrap_or(0);
        let xwayland = val.get_bool("xwayland").unwrap_or(false);
        let floating = val.get_bool("floating").unwrap_or(false);
        let fullscreen = val
            .get_bool("fullscreen")
            .or_else(|| val.get_i64("fullscreen").map(|n| n != 0))
            .unwrap_or(false);

        let (at_x, at_y) = if let Some(arr) = val.get("at").and_then(JsonValue::as_array) {
            (
                arr.first().and_then(JsonValue::as_i64).unwrap_or(0),
                arr.get(1).and_then(JsonValue::as_i64).unwrap_or(0),
            )
        } else {
            (0, 0)
        };

        let (size_w, size_h) = if let Some(arr) = val.get("size").and_then(JsonValue::as_array) {
            (
                arr.first().and_then(JsonValue::as_i64).unwrap_or(0),
                arr.get(1).and_then(JsonValue::as_i64).unwrap_or(0),
            )
        } else {
            (0, 0)
        };

        Some(Self {
            address,
            mapped,
            hidden,
            at: (at_x, at_y),
            size: (size_w, size_h),
            workspace_id,
            workspace_name,
            monitor_id,
            class,
            title,
            initial_class,
            initial_title,
            pid,
            xwayland,
            floating,
            fullscreen,
        })
    }

    pub fn matches_app(&self, app_key: &str) -> bool {
        let key_lower = app_key.to_ascii_lowercase();
        self.class.to_ascii_lowercase() == key_lower
            || self.initial_class.to_ascii_lowercase() == key_lower
            || self.class.to_ascii_lowercase().contains(&key_lower)
            || key_lower.contains(&self.class.to_ascii_lowercase())
    }

    pub fn is_minimized(&self) -> bool {
        self.hidden || self.workspace_name.starts_with("special:minimized")
    }
}

pub fn normalize_addr(addr: &str) -> String {
    let lower = addr.to_ascii_lowercase();
    lower.strip_prefix("0x").unwrap_or(&lower).to_string()
}
