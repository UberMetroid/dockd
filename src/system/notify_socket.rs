//! Pure standard library implementation of systemd sd_notify.
//!
//! Communicates with the service manager via $NOTIFY_SOCKET Unix datagram socket.

use std::env;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

pub struct NotifySocket;

impl NotifySocket {
    pub fn notify(state: &str) -> bool {
        let Ok(sock_path) = env::var("NOTIFY_SOCKET") else {
            return false;
        };
        if sock_path.is_empty() {
            return false;
        }

        let sock = match UnixDatagram::unbound() {
            Ok(s) => s,
            Err(_) => return false,
        };

        let target_addr = if let Some(stripped) = sock_path.strip_prefix('@') {
            match SocketAddr::from_abstract_name(stripped.as_bytes()) {
                Ok(addr) => addr,
                Err(_) => return false,
            }
        } else {
            match SocketAddr::from_pathname(&sock_path) {
                Ok(addr) => addr,
                Err(_) => return false,
            }
        };

        sock.connect_addr(&target_addr).is_ok() && sock.send(state.as_bytes()).is_ok()
    }

    pub fn notify_ready() -> bool {
        Self::notify("READY=1")
    }

    pub fn notify_stopping() -> bool {
        Self::notify("STOPPING=1")
    }

    pub fn notify_watchdog() -> bool {
        Self::notify("WATCHDOG=1")
    }

    pub fn watchdog_usec() -> Option<u64> {
        let pid_str = env::var("WATCHDOG_PID").ok();
        if let Some(ref p) = pid_str {
            let current_pid = std::process::id();
            if p.parse::<u32>().ok()? != current_pid {
                return None;
            }
        }
        let usec_str = env::var("WATCHDOG_USEC").ok()?;
        usec_str.parse::<u64>().ok()
    }
}
