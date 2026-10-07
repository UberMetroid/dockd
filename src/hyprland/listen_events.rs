//! Hyprland live event stream listener over .socket2.sock.
//!
//! Parses event notifications (openwindow, closewindow, activewindowv2, urgent, workspace).

use crate::system::error_status::DockError;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum HyprEvent {
    WindowOpened(String),
    WindowClosed(String),
    WindowFocused(String),
    WorkspaceChanged(String),
    MonitorChanged,
    Urgent(String),
    Generic(String),
}

pub struct EventListener {
    stream: UnixStream,
    buffer: Vec<u8>,
}

impl EventListener {
    pub fn connect(sock_path: &Path) -> Result<Self, DockError> {
        let stream = UnixStream::connect(sock_path).map_err(|e| {
            DockError::Hyprland(format!("Failed to connect to Hyprland event socket: {e}"))
        })?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buffer: Vec::new(),
        })
    }

    pub fn next_event(&mut self) -> Result<Option<HyprEvent>, DockError> {
        // First check if a line is already buffered
        if let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if line.is_empty() {
                return Ok(Some(HyprEvent::Generic(String::new())));
            }
            return Ok(Some(parse_event_line(&line)));
        }

        // Read available chunk from non-blocking stream
        let mut chunk = [0u8; 1024];
        match self.stream.read(&mut chunk) {
            Ok(0) => Err(DockError::Hyprland(
                "Hyprland event socket stream closed".to_string(),
            )),
            Ok(n) => {
                self.buffer.extend_from_slice(&chunk[..n]);
                if let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
                    let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
                    if line.is_empty() {
                        return Ok(Some(HyprEvent::Generic(String::new())));
                    }
                    Ok(Some(parse_event_line(&line)))
                } else {
                    Ok(None)
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(DockError::Hyprland(e.to_string())),
        }
    }
}

pub fn parse_event_line(line: &str) -> HyprEvent {
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
        "urgent" => HyprEvent::Urgent(payload.to_string()),
        _ => HyprEvent::Generic(line.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_urgent_event() {
        let ev = parse_event_line("urgent>>559e2a80");
        assert_eq!(ev, HyprEvent::Urgent("559e2a80".to_string()));
    }

    #[test]
    fn test_parse_openwindow_event() {
        let ev = parse_event_line("openwindow>>0x1234,workspace1,kitty,title");
        assert_eq!(ev, HyprEvent::WindowOpened("0x1234".to_string()));
    }
}
