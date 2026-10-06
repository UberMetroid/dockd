//! Sends window manipulation commands to Hyprland's IPC socket.
//!
//! Formats dispatch arguments for focus, minimize, tile, float, move, and close.

use crate::system::error_status::DockError;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub fn send_dispatch(sock_path: &Path, dispatch_args: &str) -> Result<String, DockError> {
    let mut stream = UnixStream::connect(sock_path).map_err(|e| {
        DockError::Hyprland(format!("Failed to connect to Hyprland command socket: {e}"))
    })?;

    let cmd = format!("dispatch {dispatch_args}");
    stream
        .write_all(cmd.as_bytes())
        .map_err(|e| DockError::Hyprland(format!("Failed to write dispatch command: {e}")))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| DockError::Hyprland(format!("Failed to read dispatch response: {e}")))?;

    Ok(response.trim().to_string())
}

pub fn focus_window(sock_path: &Path, address: &str) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    send_dispatch(sock_path, &format!("focuswindow address:{addr}"))?;
    Ok(())
}

pub fn minimize_window(sock_path: &Path, address: &str) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    send_dispatch(
        sock_path,
        &format!("movetoworkspacesilent special:minimized,address:{addr}"),
    )?;
    Ok(())
}

pub fn restore_window(
    sock_path: &Path,
    address: &str,
    target_workspace: i64,
) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    let ws = if target_workspace > 0 {
        target_workspace
    } else {
        1
    };
    send_dispatch(sock_path, &format!("movetoworkspace {ws},address:{addr}"))?;
    send_dispatch(sock_path, &format!("focuswindow address:{addr}"))?;
    Ok(())
}

pub fn toggle_floating(
    sock_path: &Path,
    address: &str,
    make_floating: bool,
) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    let flag = if make_floating { "1" } else { "0" };
    send_dispatch(sock_path, &format!("setfloating address:{addr} {flag}"))?;
    Ok(())
}

pub fn snap_window(
    sock_path: &Path,
    address: &str,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    send_dispatch(sock_path, &format!("setfloating address:{addr} 1"))?;
    send_dispatch(
        sock_path,
        &format!("resizewindowpixel exact {w} {h},address:{addr}"),
    )?;
    send_dispatch(
        sock_path,
        &format!("movewindowpixel exact {x} {y},address:{addr}"),
    )?;
    send_dispatch(sock_path, &format!("focuswindow address:{addr}"))?;
    Ok(())
}

pub fn close_window(sock_path: &Path, address: &str) -> Result<(), DockError> {
    let addr = sanitize_address(address);
    send_dispatch(sock_path, &format!("closewindow address:{addr}"))?;
    Ok(())
}

pub fn force_quit_pid(pid: u64) -> Result<(), DockError> {
    if pid > 1 {
        // Send SIGKILL to the process
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(format!("{pid}"))
            .status();
        Ok(())
    } else {
        Err(DockError::Protocol(
            "Invalid PID for termination".to_string(),
        ))
    }
}

fn sanitize_address(raw: &str) -> &str {
    raw.trim().trim_start_matches("address:")
}
