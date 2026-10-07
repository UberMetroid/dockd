//! Dispatches incoming client JSON action requests against Hyprland and state.
//!
//! Handles launch, toggle, window arrangement, pinning, reordering, and layout profile mutations.

use super::dock_state::DockState;
use crate::hyprland::{
    close_window, focus_window, force_quit_pid, minimize_window, query_active_window,
    query_clients, query_monitors, restore_window, snap_window, toggle_floating,
};
use crate::syntax::json_value::JsonValue;
use crate::system::error_status::DockError;
use crate::system::launch_binary::{launch_exec, launch_location};
use std::collections::BTreeMap;
use std::path::Path;

pub fn execute_action(
    req: &JsonValue,
    state: &mut DockState,
    hypr_cmd_sock: Option<&Path>,
) -> Result<JsonValue, DockError> {
    let action = req.get_str("action").ok_or_else(|| {
        DockError::Protocol("Missing 'action' key in request payload".to_string())
    })?;

    match action {
        "get_state" => Ok(state.to_json()),
        "set_profile" => {
            let prof = req.get_str("profile").unwrap_or("mac");
            state.set_profile(prof);
            Ok(json_ok("Profile updated"))
        }
        "set_autohide" => {
            let enabled = req.get_bool("enabled").unwrap_or(true);
            state.set_autohide(enabled);
            Ok(json_ok("Autohide updated"))
        }
        "set_dock_size" => {
            let size = req.get_str("size").unwrap_or("medium");
            state.set_dock_size(size);
            Ok(json_ok("Dock size updated"))
        }
        "set_magnification" => {
            let mag = req.get_bool("magnification").unwrap_or(true);
            state.set_magnification(mag);
            Ok(json_ok("Magnification updated"))
        }
        "set_show_indicators" => {
            let ind = req.get_bool("show_indicators").unwrap_or(true);
            state.set_show_indicators(ind);
            Ok(json_ok("Indicators updated"))
        }
        "reset_pinned" => {
            state.reset_pinned();
            sync_hypr_state(state, hypr_cmd_sock);
            Ok(json_ok("Pinned reset to defaults"))
        }
        "pin" => {
            let id = req
                .get_str("desktop_id")
                .ok_or("Missing desktop_id to pin")?;
            state.pin(id);
            sync_hypr_state(state, hypr_cmd_sock);
            Ok(json_ok("Pinned"))
        }
        "unpin" => {
            let id = req
                .get_str("desktop_id")
                .ok_or("Missing desktop_id to unpin")?;
            state.unpin(id);
            sync_hypr_state(state, hypr_cmd_sock);
            Ok(json_ok("Unpinned"))
        }
        "reorder" => {
            let id = req
                .get_str("desktop_id")
                .ok_or("Missing desktop_id to reorder")?;
            let idx = req.get_i64("index").unwrap_or(0).max(0) as usize;
            if state.reorder_pin(id, idx) {
                sync_hypr_state(state, hypr_cmd_sock);
                Ok(json_ok("Reordered"))
            } else {
                Err(DockError::NotFound(format!(
                    "Desktop ID '{id}' is not pinned"
                )))
            }
        }
        "set_monitor_filter" => {
            let mon_id = if req.get("monitor_id").is_some_and(|v| v.is_null()) {
                None
            } else {
                req.get_i64("monitor_id").filter(|&m| m >= 0)
            };
            state.set_filter_monitor(mon_id);
            sync_hypr_state(state, hypr_cmd_sock);
            Ok(json_ok("Monitor filter updated"))
        }
        "set_overlap" => {
            if let Some(overlap) = req.get_bool("overlap") {
                state.overlap = overlap;
            }
            Ok(json_ok("Overlap updated"))
        }
        "open_location" => {
            let target = req.get_str("target").unwrap_or("home");
            launch_location(target)?;
            Ok(json_ok("Location opened"))
        }
        "launch" => {
            if let Some(exec) = req.get_str("exec") {
                launch_exec(exec)?;
                return Ok(json_ok("Launched"));
            }
            if let Some(id) = req.get_str("desktop_id")
                && let Some(entry) = state.desktop_entries.iter().find(|e| e.id == id)
            {
                launch_exec(&entry.exec)?;
                return Ok(json_ok("Launched"));
            }
            Err(DockError::NotFound(
                "Application executable not found".to_string(),
            ))
        }
        "toggle" => handle_toggle(req, state, hypr_cmd_sock),
        "focus" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            focus_window(sock, addr)?;
            Ok(json_ok("Focused"))
        }
        "minimize" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            minimize_window(sock, addr)?;
            Ok(json_ok("Minimized"))
        }
        "restore" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let ws = req.get_i64("workspace_id").unwrap_or(1);
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            restore_window(sock, addr, ws)?;
            Ok(json_ok("Restored"))
        }
        "tile" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            toggle_floating(sock, addr, false)?;
            focus_window(sock, addr)?;
            Ok(json_ok("Tiled"))
        }
        "float" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            toggle_floating(sock, addr, true)?;
            focus_window(sock, addr)?;
            Ok(json_ok("Floating"))
        }
        "snap_left" | "snap_right" => handle_snap(action, req, hypr_cmd_sock),
        "close" => {
            let addr = req.get_str("address").ok_or("Missing address")?;
            let sock =
                hypr_cmd_sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
            close_window(sock, addr)?;
            Ok(json_ok("Closed"))
        }
        "force_quit" => {
            let pid = req.get_u64("pid").ok_or("Missing pid")?;
            force_quit_pid(pid)?;
            Ok(json_ok("Terminated"))
        }
        other => Err(DockError::Protocol(format!("Unknown action: {other}"))),
    }
}

fn sync_hypr_state(state: &mut DockState, sock: Option<&Path>) {
    if let Some(s) = sock {
        let clients = query_clients(s).unwrap_or_default();
        let active = query_active_window(s).unwrap_or(None);
        let active_addr = active.as_ref().map(|c| c.address.as_str());
        state.rebuild_items(&clients, active_addr);
    } else {
        state.rebuild_items(&[], None);
    }
}

fn handle_toggle(
    req: &JsonValue,
    state: &mut DockState,
    sock: Option<&Path>,
) -> Result<JsonValue, DockError> {
    let id = req.get_str("desktop_id").unwrap_or("");
    let item = state.items.iter().find(|it| it.desktop_id == id);
    if let Some(it) = item
        && !it.windows.is_empty()
    {
        let sock_path = sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
        if let Some(focused_win) = it.windows.iter().find(|w| w.focused) {
            minimize_window(sock_path, &focused_win.address)?;
            return Ok(json_ok("Minimized active window"));
        } else {
            let target = &it.windows[0];
            if target.minimized {
                restore_window(sock_path, &target.address, target.workspace_id)?;
            } else {
                focus_window(sock_path, &target.address)?;
            }
            return Ok(json_ok("Focused window"));
        }
    }
    if let Some(entry) = state.desktop_entries.iter().find(|e| e.id == id) {
        launch_exec(&entry.exec)?;
        return Ok(json_ok("Launched"));
    }
    Err(DockError::NotFound(
        "Cannot toggle unknown application".into(),
    ))
}

fn handle_snap(action: &str, req: &JsonValue, sock: Option<&Path>) -> Result<JsonValue, DockError> {
    let addr = req.get_str("address").ok_or("Missing address")?;
    let sock_path = sock.ok_or(DockError::Hyprland("Hyprland socket unavailable".into()))?;
    let clients = query_clients(sock_path)?;
    let client = clients
        .iter()
        .find(|c| c.address == addr)
        .ok_or("Client not found")?;
    let monitors = query_monitors(sock_path)?;
    let mon = monitors
        .iter()
        .find(|m| m.id == client.monitor_id)
        .or_else(|| monitors.iter().find(|m| m.focused))
        .ok_or("Monitor not found")?;

    let (x, y, w, h) = if action == "snap_left" {
        mon.snap_left_geometry()
    } else {
        mon.snap_right_geometry()
    };
    snap_window(sock_path, addr, x, y, w, h)?;
    Ok(json_ok("Snapped"))
}

fn json_ok(msg: &str) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("status".to_string(), JsonValue::String("ok".to_string()));
    map.insert("message".to_string(), JsonValue::String(msg.to_string()));
    JsonValue::Object(map)
}
