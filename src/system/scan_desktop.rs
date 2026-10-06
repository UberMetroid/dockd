//! Scans and parses XDG .desktop application entries from system and user paths.
//!
//! Extracts application names, executable commands, icon names, and window classes.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub wm_class: Option<String>,
    pub no_display: bool,
    pub terminal: bool,
}

pub fn scan_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();
    let mut search_dirs = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        search_dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    search_dirs.push(PathBuf::from("/usr/local/share/applications"));
    search_dirs.push(PathBuf::from("/usr/share/applications"));

    let mut seen_ids = std::collections::BTreeSet::new();

    for dir in search_dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(read_dir) = fs::read_dir(dir) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if seen_ids.contains(file_name) {
                continue;
            }
            if let Some(parsed) = parse_desktop_file(&path, file_name) {
                seen_ids.insert(file_name.to_string());
                entries.push(parsed);
            }
        }
    }
    entries
}

fn parse_desktop_file(path: &Path, id: &str) -> Option<DesktopEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut name = String::new();
    let mut exec = String::new();
    let mut icon = String::new();
    let mut wm_class = None;
    let mut no_display = false;
    let mut terminal = false;
    let mut is_application = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_entry = trimmed == "[Desktop Entry]";
            continue;
        }
        if !in_entry || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, val)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim();

        match key {
            "Type" => is_application = val == "Application",
            "Name" if name.is_empty() => name = val.to_string(),
            "Exec" => exec = sanitize_exec(val),
            "Icon" => icon = val.to_string(),
            "StartupWMClass" => wm_class = Some(val.to_string()),
            "NoDisplay" => no_display = val.eq_ignore_ascii_case("true"),
            "Terminal" => terminal = val.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    if is_application && !name.is_empty() && !exec.is_empty() {
        Some(DesktopEntry {
            id: id.to_string(),
            name,
            exec,
            icon,
            wm_class,
            no_display,
            terminal,
        })
    } else {
        None
    }
}

fn sanitize_exec(raw: &str) -> String {
    // Strip standard desktop field codes: %f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m
    let parts: Vec<&str> = raw
        .split_whitespace()
        .filter(|arg| !arg.starts_with('%'))
        .collect();
    parts.join(" ")
}
