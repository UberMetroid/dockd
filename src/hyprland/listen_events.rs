//! Hyprland live event stream listener over .socket2.sock.
//!
//! Parses event notifications (openwindow, closewindow, activewindowv2, workspace).

use crate::system::error_status::DockError;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum HyprEvent {
    WindowOpened(String),
    WindowClosed(String),
    WindowFocused(String),
    WorkspaceChanged(String),
    MonitorChanged,
    Generic(String),
}

pub struct EventListener {
    reader: BufReader<UnixStream>,
}

impl EventListener {
    pub fn connect(sock_path: &Path) -> Result<Self, DockError> {
        let stream = UnixStream::connect(sock_path).map_err(|e| {
            DockError::Hyprland(format!("Failed to connect to Hyprland event socket: {e}"))
        })?;
        stream.set_nonblocking(false)?;
        Ok(Self {
            reader: BufReader::new(stream),
        })
    }

    pub fn next_event(&mut self) -> Result<Option<HyprEvent>, DockError> {
        let mut line = String::new();
        let bytes = self.reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(Some(HyprEvent::Generic(String::new())));
        }

        Ok(Some(parse_event_line(trimmed)))
    }
}

fn parse_event_line(line: &str) -> HyprEvent {
    let Some((name, payload)) = line.split_once(">>") else {
        return HyprEvent::Generic(line.to_string());
    };

    match name {
        "openwindow" => {
            let addr = payload.split(',').next().unwrap_or("").to_string();
            HyprEvent::WindowOpened(addr)
        }
        "closewindow" => HyprEvent::WindowClosed(payload.to_string()),
        "activewindowv2" => HyprEvent::WindowFocused(payload.to_string()),
        "activewindow" => HyprEvent::WindowFocused(payload.to_string()),
        "workspace" => HyprEvent::WorkspaceChanged(payload.to_string()),
        "focusedmon" => HyprEvent::MonitorChanged,
        _ => HyprEvent::Generic(line.to_string()),
    }
}
