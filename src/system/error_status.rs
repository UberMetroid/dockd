//! Standard domain errors and exit codes for dockd.
//!
//! Maps error states to standard sysexits exit codes without unwrap or panic.

use std::fmt;

#[derive(Debug)]
pub enum DockError {
    Io(std::io::Error),
    Protocol(String),
    Hyprland(String),
    Systemd(String),
    Desktop(String),
    NotFound(String),
}

impl fmt::Display for DockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Protocol(msg) => write!(f, "Protocol violation: {msg}"),
            Self::Hyprland(msg) => write!(f, "Hyprland IPC error: {msg}"),
            Self::Systemd(msg) => write!(f, "systemd integration error: {msg}"),
            Self::Desktop(msg) => write!(f, "Desktop entry error: {msg}"),
            Self::NotFound(msg) => write!(f, "Resource not found: {msg}"),
        }
    }
}

impl std::error::Error for DockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for DockError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<String> for DockError {
    fn from(msg: String) -> Self {
        Self::Protocol(msg)
    }
}

impl From<&str> for DockError {
    fn from(msg: &str) -> Self {
        Self::Protocol(msg.to_string())
    }
}

impl DockError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Systemd(_) => 69,  // EX_UNAVAILABLE
            Self::Protocol(_) => 78, // EX_CONFIG
            Self::Hyprland(_) => 69, // EX_UNAVAILABLE
            Self::Io(_) => 74,       // EX_IOERR
            Self::Desktop(_) => 78,  // EX_CONFIG
            Self::NotFound(_) => 69, // EX_UNAVAILABLE
        }
    }
}
