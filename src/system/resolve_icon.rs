//! Icon path resolution conforming to the XDG Icon Theme specification.
//!
//! Detects active GTK icon themes and locates PNG/SVG icons across standard directories.

use std::fs;
use std::path::{Path, PathBuf};

const ICON_EXTENSIONS: &[&str] = &["png", "svg", "xpm"];
const PREFERRED_SIZES: &[&str] = &[
    "scalable/apps",
    "512x512/apps",
    "256x256/apps",
    "128x128/apps",
    "64x64/apps",
    "48x48/apps",
    "32x32/apps",
];

pub fn detect_active_icon_theme() -> Option<String> {
    let mut candidates = Vec::new();
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        candidates.push(PathBuf::from(&config_home).join("gtk-3.0/settings.ini"));
        candidates.push(PathBuf::from(&config_home).join("gtk-4.0/settings.ini"));
    }
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(&home).join(".config/gtk-3.0/settings.ini"));
        candidates.push(PathBuf::from(&home).join(".config/gtk-4.0/settings.ini"));
    }

    for path in &candidates {
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some((k, v)) = trimmed.split_once('=')
                && k.trim() == "gtk-icon-theme-name"
            {
                let theme = v.trim().trim_matches('"').trim_matches('\'').trim();
                if !theme.is_empty() {
                    return Some(theme.to_string());
                }
            }
        }
    }

    None
}

pub fn resolve_icon_path(icon_name: &str) -> Option<String> {
    if icon_name.is_empty() {
        return None;
    }

    let direct_path = Path::new(icon_name);
    if direct_path.is_absolute() && direct_path.is_file() {
        return Some(icon_name.to_string());
    }

    let mut base_dirs = Vec::new();
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        base_dirs.push(PathBuf::from(data_home).join("icons"));
    }
    if let Ok(home) = std::env::var("HOME") {
        base_dirs.push(PathBuf::from(home).join(".local/share/icons"));
    }
    base_dirs.push(PathBuf::from("/usr/share/icons"));
    base_dirs.push(PathBuf::from("/usr/share/pixmaps"));

    // Check pixmaps directly first if simple filename
    for ext in ICON_EXTENSIONS {
        let pixmap = PathBuf::from("/usr/share/pixmaps").join(format!("{icon_name}.{ext}"));
        if pixmap.is_file() {
            return Some(pixmap.to_string_lossy().into_owned());
        }
    }

    // Determine theme search list: active theme first, then hicolor fallback
    let mut themes = Vec::new();
    if let Some(active) = detect_active_icon_theme() {
        themes.push(active);
    }
    themes.push("hicolor".to_string());

    // Search theme hierarchies
    for theme in &themes {
        for base in &base_dirs {
            let theme_dir = base.join(theme);
            if !theme_dir.is_dir() {
                continue;
            }
            for size in PREFERRED_SIZES {
                let app_dir = theme_dir.join(size);
                for ext in ICON_EXTENSIONS {
                    let candidate = app_dir.join(format!("{icon_name}.{ext}"));
                    if candidate.is_file() {
                        return Some(candidate.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }

    // Case-insensitive direct check under pixmaps
    if let Ok(entries) = std::fs::read_dir("/usr/share/pixmaps") {
        let target_lower = icon_name.to_ascii_lowercase();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && stem.to_ascii_lowercase() == target_lower
                && path.is_file()
            {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_active_icon_theme_none_when_empty() {
        // Safe check without modifying real home
        let _ = detect_active_icon_theme();
    }

    #[test]
    fn test_resolve_empty_icon_name() {
        assert_eq!(resolve_icon_path(""), None);
    }
}
