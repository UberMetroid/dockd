//! Dynamic Omarchy theme synchronization.
//!
//! Loads color palettes from ~/.config/omarchy/current/theme or shell.json with fallback defaults.

use crate::syntax::json_parse::parse;
use crate::syntax::json_value::JsonValue;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct ThemePalette {
    pub background: String,
    pub card: String,
    pub foreground: String,
    pub muted: String,
    pub accent: String,
    pub urgent: String,
    pub border: String,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            background: "#0f172a".to_string(),
            card: "#1e293b".to_string(),
            foreground: "#f8fafc".to_string(),
            muted: "#94a3b8".to_string(),
            accent: "#38bdf8".to_string(),
            urgent: "#ef4444".to_string(),
            border: "rgba(255, 255, 255, 0.12)".to_string(),
        }
    }
}

impl ThemePalette {
    pub fn to_json(&self) -> JsonValue {
        let mut map = BTreeMap::new();
        map.insert(
            "background".to_string(),
            JsonValue::String(self.background.clone()),
        );
        map.insert("card".to_string(), JsonValue::String(self.card.clone()));
        map.insert(
            "foreground".to_string(),
            JsonValue::String(self.foreground.clone()),
        );
        map.insert("muted".to_string(), JsonValue::String(self.muted.clone()));
        map.insert("accent".to_string(), JsonValue::String(self.accent.clone()));
        map.insert("urgent".to_string(), JsonValue::String(self.urgent.clone()));
        map.insert("border".to_string(), JsonValue::String(self.border.clone()));
        JsonValue::Object(map)
    }

    pub fn from_json(val: &JsonValue) -> Self {
        let mut p = Self::default();
        if let Some(bg) = val.get_str("background").or_else(|| val.get_str("bg")) {
            p.background = bg.to_string();
        }
        if let Some(c) = val.get_str("card").or_else(|| val.get_str("surface")) {
            p.card = c.to_string();
        }
        if let Some(fg) = val.get_str("foreground").or_else(|| val.get_str("fg")) {
            p.foreground = fg.to_string();
        }
        if let Some(m) = val.get_str("muted") {
            p.muted = m.to_string();
        }
        if let Some(acc) = val.get_str("accent").or_else(|| val.get_str("primary")) {
            p.accent = acc.to_string();
        }
        if let Some(urg) = val.get_str("urgent").or_else(|| val.get_str("danger")) {
            p.urgent = urg.to_string();
        }
        if let Some(b) = val.get_str("border") {
            p.border = b.to_string();
        }
        p
    }
}

pub fn load_omarchy_theme() -> ThemePalette {
    let Ok(home) = std::env::var("HOME") else {
        return ThemePalette::default();
    };

    let omarchy_dir = PathBuf::from(home).join(".config/omarchy");

    // 1. Try theme files in current/theme/
    let theme_candidates = [
        omarchy_dir.join("current/theme/colors.json"),
        omarchy_dir.join("current/theme/palette.json"),
        omarchy_dir.join("current/theme/theme.json"),
        omarchy_dir.join("shell.json"),
    ];

    for path in &theme_candidates {
        if let Some(palette) = try_load_file(path) {
            return palette;
        }
    }

    ThemePalette::default()
}

fn try_load_file(path: &Path) -> Option<ThemePalette> {
    let content = fs::read_to_string(path).ok()?;
    let json = parse(&content).ok()?;

    if let Some(colors) = json.get("colors").or_else(|| json.get("palette")) {
        return Some(ThemePalette::from_json(colors));
    }

    Some(ThemePalette::from_json(&json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_palette_json_roundtrip() {
        let palette = ThemePalette::default();
        let json = palette.to_json();
        let loaded = ThemePalette::from_json(&json);
        assert_eq!(palette, loaded);
        assert_eq!(json.get_str("accent"), Some("#38bdf8"));
    }

    #[test]
    fn test_custom_theme_palette() {
        let txt = r##"{"accent":"#ff5555","background":"#000000"}"##;
        let json = parse(txt).expect("valid json in test");
        let palette = ThemePalette::from_json(&json);
        assert_eq!(palette.accent, "#ff5555");
        assert_eq!(palette.background, "#000000");
        assert_eq!(palette.card, "#1e293b"); // default preserved
    }
}
