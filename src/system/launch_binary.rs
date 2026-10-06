//! Spawns applications and file locations safely without shell wrapping.
//!
//! Subprocesses are executed directly via discrete binary arguments with stdio decoupled.

use super::error_status::DockError;
use std::process::{Command, Stdio};

pub fn launch_exec(exec_cmd: &str) -> Result<(), DockError> {
    let mut parts = exec_cmd.split_whitespace();
    let binary = parts
        .next()
        .ok_or_else(|| DockError::Desktop("Empty executable command specified".to_string()))?;

    let args: Vec<&str> = parts.collect();

    let mut cmd = Command::new(binary);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    pass_desktop_env(&mut cmd);

    cmd.spawn()
        .map_err(|e| DockError::Desktop(format!("Failed to spawn '{binary}': {e}")))?;

    Ok(())
}

pub fn launch_location(target: &str) -> Result<(), DockError> {
    let uri = match target {
        "home" => std::env::var("HOME").unwrap_or_else(|_| "/home".to_string()),
        "downloads" => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
            let dl = std::path::PathBuf::from(home).join("Downloads");
            if dl.is_dir() {
                dl.to_string_lossy().into_owned()
            } else {
                std::env::var("HOME").unwrap_or_else(|_| "/home".to_string())
            }
        }
        "trash" => "trash:///".to_string(),
        other => {
            return Err(DockError::Desktop(format!(
                "Unknown location target '{other}'"
            )));
        }
    };

    let mut cmd = Command::new("gio");
    cmd.arg("open")
        .arg("--")
        .arg(uri)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    pass_desktop_env(&mut cmd);

    cmd.spawn()
        .map_err(|e| DockError::Desktop(format!("Failed to open location '{target}': {e}")))?;

    Ok(())
}

fn pass_desktop_env(cmd: &mut Command) {
    let preserved = [
        "PATH",
        "HOME",
        "USER",
        "WAYLAND_DISPLAY",
        "DISPLAY",
        "XDG_RUNTIME_DIR",
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_TYPE",
        "XDG_DATA_DIRS",
        "XDG_CONFIG_HOME",
    ];

    for var in preserved {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }
}
