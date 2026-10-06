//! Native systemd socket activation listener ($LISTEN_FDS).
//!
//! Inspects inherited descriptors starting at SD_LISTEN_FDS_START (3).

#![allow(unsafe_code)]

use super::error_status::DockError;
use std::env;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixListener;

const SD_LISTEN_FDS_START: i32 = 3;

pub fn try_listen_fds() -> Result<Option<UnixListener>, DockError> {
    let Ok(fds_str) = env::var("LISTEN_FDS") else {
        return Ok(None);
    };
    let num_fds: usize = fds_str
        .parse()
        .map_err(|_| DockError::Systemd("Invalid LISTEN_FDS value".to_string()))?;
    if num_fds == 0 {
        return Ok(None);
    }

    if let Ok(pid_str) = env::var("LISTEN_PID") {
        let pid: u32 = pid_str
            .parse()
            .map_err(|_| DockError::Systemd("Invalid LISTEN_PID value".to_string()))?;
        if pid != std::process::id() {
            return Ok(None);
        }
    }

    // SAFETY: systemd contracts guarantee that when LISTEN_FDS >= 1, file descriptor
    // 3 is an open socket passed directly to this process. We take ownership of FD 3
    // exactly once during startup.
    let listener = unsafe { UnixListener::from_raw_fd(SD_LISTEN_FDS_START) };
    listener.set_nonblocking(true)?;
    Ok(Some(listener))
}
