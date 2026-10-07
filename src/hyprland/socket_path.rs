//! Locates Hyprland IPC command and event Unix domain sockets.
//!
//! Discovers sockets via $HYPRLAND_INSTANCE_SIGNATURE or by scanning $XDG_RUNTIME_DIR/hypr.

use crate::system::error_status::DockError;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct HyprlandSockets {
    pub command_socket: PathBuf,
    pub event_socket: PathBuf,
}

pub fn locate_sockets() -> Result<HyprlandSockets, DockError> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid = libc_getuid();
        format!("/run/user/{uid}")
    });

    let hypr_base = PathBuf::from(runtime_dir).join("hypr");

    // 1. Try explicit HYPRLAND_INSTANCE_SIGNATURE from environment
    if let Ok(signature) = env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        let candidate = hypr_base.join(&signature);
        let cmd = candidate.join(".socket.sock");
        let ev = candidate.join(".socket2.sock");
        if cmd.exists() && ev.exists() {
            return Ok(HyprlandSockets {
                command_socket: cmd,
                event_socket: ev,
            });
        }
    }

    // 2. Discover active instance directories under $XDG_RUNTIME_DIR/hypr
    if let Ok(entries) = fs::read_dir(&hypr_base) {
        let mut best_match: Option<(SystemTime, PathBuf, PathBuf)> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let cmd = path.join(".socket.sock");
            let ev = path.join(".socket2.sock");
            if cmd.exists() && ev.exists() {
                let mtime = fs::metadata(&cmd)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                match &best_match {
                    Some((best_time, _, _)) if mtime > *best_time => {
                        best_match = Some((mtime, cmd, ev));
                    }
                    None => {
                        best_match = Some((mtime, cmd, ev));
                    }
                    _ => {}
                }
            }
        }

        if let Some((_, command_socket, event_socket)) = best_match {
            return Ok(HyprlandSockets {
                command_socket,
                event_socket,
            });
        }
    }

    Err(DockError::Hyprland(
        "Hyprland IPC sockets not found in runtime directory".to_string(),
    ))
}

fn libc_getuid() -> u32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libc_getuid_valid() {
        let uid = libc_getuid();
        assert!(uid < 1_000_000);
    }
}
