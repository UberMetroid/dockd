//! Icon path resolution conforming to the XDG Icon Theme specification.
//!
//! Locates PNG and SVG icons across standard system directories and user overrides.

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

pub fn resolve_icon_path(icon_name: &str) -> Option<String> {
    if icon_name.is_empty() {
        return None;
    }

    let direct_path = Path::new(icon_name);
    if direct_path.is_absolute() && direct_path.is_file() {
        return Some(icon_name.to_string());
    }

    let mut base_dirs = Vec::new();
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

    // Search theme hierarchies (hicolor fallback)
    for base in &base_dirs {
        let hicolor = base.join("hicolor");
        if !hicolor.is_dir() {
            continue;
        }
        for size in PREFERRED_SIZES {
            let app_dir = hicolor.join(size);
            for ext in ICON_EXTENSIONS {
                let candidate = app_dir.join(format!("{icon_name}.{ext}"));
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().into_owned());
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
