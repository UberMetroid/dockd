//! Locates Hyprland IPC command and event Unix domain sockets.
//!
//! Inspects $HYPRLAND_INSTANCE_SIGNATURE and $XDG_RUNTIME_DIR to construct socket paths.

use crate::system::error_status::DockError;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HyprlandSockets {
    pub command_socket: PathBuf,
    pub event_socket: PathBuf,
}

pub fn locate_sockets() -> Result<HyprlandSockets, DockError> {
    let signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
        DockError::Hyprland("HYPRLAND_INSTANCE_SIGNATURE environment variable not set".to_string())
    })?;

    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid = libc_getuid();
        format!("/run/user/{uid}")
    });

    let base = PathBuf::from(runtime_dir).join("hypr").join(signature);
    let command_socket = base.join(".socket.sock");
    let event_socket = base.join(".socket2.sock");

    Ok(HyprlandSockets {
        command_socket,
        event_socket,
    })
}

fn libc_getuid() -> u32 {
    // Pure std fallback: get uid from /proc/self/status or fallback 1000
    if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && let Ok(uid) = parts[1].parse::<u32>()
                {
                    return uid;
                }
            }
        }
    }
    1000
}
