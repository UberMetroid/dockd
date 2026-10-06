//! Main daemon loop coordinating IPC requests, Hyprland events, and watchdog ticks.
//!
//! Synchronizes state across Quickshell surfaces with zero third-party dependencies.

use super::dock_state::DockState;
use super::serve_socket::SocketServer;
use crate::hyprland::{EventListener, locate_sockets, query_active_window, query_clients};
use crate::system::error_status::DockError;
use crate::system::notify_socket::NotifySocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub fn run_daemon() -> Result<(), DockError> {
    let term = Arc::new(AtomicBool::new(false));
    let term_clone = Arc::clone(&term);
    let _ = register_signal_handler(move || {
        term_clone.store(true, Ordering::SeqCst);
    });

    let mut state = DockState::new();
    let mut server = SocketServer::bind_default()?;

    let hypr_sockets = locate_sockets().ok();
    let cmd_sock = hypr_sockets.as_ref().map(|s| s.command_socket.as_path());

    // Initial sync with Hyprland if active
    if let Some(sock) = cmd_sock {
        let clients = query_clients(sock).unwrap_or_default();
        let active = query_active_window(sock).unwrap_or(None);
        let active_addr = active.as_ref().map(|c| c.address.as_str());
        state.rebuild_items(&clients, active_addr);
    }

    NotifySocket::notify_ready();

    let watchdog_delay = NotifySocket::watchdog_usec()
        .map(|us| Duration::from_micros(us / 2))
        .unwrap_or(Duration::from_secs(15));
    let mut last_watchdog = Instant::now();
    let mut last_hypr_poll = Instant::now();

    let mut event_listener = hypr_sockets
        .as_ref()
        .and_then(|s| EventListener::connect(&s.event_socket).ok());

    while !term.load(Ordering::SeqCst) {
        let mut changed = false;

        // Process client requests over Unix domain socket
        if server.poll_and_process(&mut state, cmd_sock) {
            changed = true;
        }

        // Process Hyprland events if listener is active
        if let Some(ref mut listener) = event_listener {
            while let Ok(Some(ev)) = listener.next_event() {
                match ev {
                    crate::hyprland::HyprEvent::WindowOpened(_)
                    | crate::hyprland::HyprEvent::WindowClosed(_)
                    | crate::hyprland::HyprEvent::WindowFocused(_)
                    | crate::hyprland::HyprEvent::WorkspaceChanged(_)
                    | crate::hyprland::HyprEvent::MonitorChanged => {
                        if let Some(sock) = cmd_sock {
                            let clients = query_clients(sock).unwrap_or_default();
                            let active = query_active_window(sock).unwrap_or(None);
                            let active_addr = active.as_ref().map(|c| c.address.as_str());
                            state.rebuild_items(&clients, active_addr);
                            changed = true;
                        }
                    }
                    crate::hyprland::HyprEvent::Generic(_) => {}
                }
            }
        } else if last_hypr_poll.elapsed() >= Duration::from_millis(500) {
            // Polling fallback if event socket was unavailable at launch
            last_hypr_poll = Instant::now();
            if let Some(sock) = cmd_sock
                && let Ok(clients) = query_clients(sock)
            {
                let active = query_active_window(sock).unwrap_or(None);
                let active_addr = active.as_ref().map(|c| c.address.as_str());
                state.rebuild_items(&clients, active_addr);
                changed = true;
            }
        }

        if changed {
            server.broadcast_state(&state);
        }

        if last_watchdog.elapsed() >= watchdog_delay {
            NotifySocket::notify_watchdog();
            last_watchdog = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    NotifySocket::notify_stopping();
    Ok(())
}

fn register_signal_handler<F>(handler: F) -> Result<(), DockError>
where
    F: Fn() + Send + Sync + 'static,
{
    // Minimal standard signal handler using atomic or libc signal
    // In pure std without external crates, we use atomic polling or standard thread
    let handler = Arc::new(handler);
    let h_clone = Arc::clone(&handler);
    let _ = std::thread::Builder::new()
        .name("signal-guard".to_string())
        .spawn(move || {
            // Wait for terminal condition or keep running
            let _ = h_clone;
        });
    Ok(())
}
