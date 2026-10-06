//! Queries current window, monitor, and active client state from Hyprland.
//!
//! Sends JSON inspection commands (`j/clients`, `j/monitors`, `j/activewindow`).

use super::client_table::HyprClient;
use super::monitor_layout::HyprMonitor;
use crate::syntax::json_parse::parse;
use crate::syntax::json_value::JsonValue;
use crate::system::error_status::DockError;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub fn query_raw_json(sock_path: &Path, command: &str) -> Result<JsonValue, DockError> {
    let mut stream = UnixStream::connect(sock_path)
        .map_err(|e| DockError::Hyprland(format!("Failed to connect to Hyprland socket: {e}")))?;

    stream
        .write_all(command.as_bytes())
        .map_err(|e| DockError::Hyprland(format!("Failed to send query '{command}': {e}")))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| DockError::Hyprland(format!("Failed to read query response: {e}")))?;

    parse(&response)
        .map_err(|e| DockError::Protocol(format!("Failed to parse Hyprland JSON response: {e}")))
}

pub fn query_clients(sock_path: &Path) -> Result<Vec<HyprClient>, DockError> {
    let json = query_raw_json(sock_path, "j/clients")?;
    let arr = json
        .as_array()
        .ok_or_else(|| DockError::Protocol("j/clients returned non-array JSON".to_string()))?;

    let clients = arr.iter().filter_map(HyprClient::from_json).collect();
    Ok(clients)
}

pub fn query_monitors(sock_path: &Path) -> Result<Vec<HyprMonitor>, DockError> {
    let json = query_raw_json(sock_path, "j/monitors")?;
    let arr = json
        .as_array()
        .ok_or_else(|| DockError::Protocol("j/monitors returned non-array JSON".to_string()))?;

    let monitors = arr.iter().filter_map(HyprMonitor::from_json).collect();
    Ok(monitors)
}

pub fn query_active_window(sock_path: &Path) -> Result<Option<HyprClient>, DockError> {
    let json = query_raw_json(sock_path, "j/activewindow")?;
    if json.is_null() {
        return Ok(None);
    }
    // If activewindow has an empty address, no window is focused
    if let Some(addr) = json.get_str("address")
        && (addr.is_empty() || addr == "0x0")
    {
        return Ok(None);
    }
    Ok(HyprClient::from_json(&json))
}
