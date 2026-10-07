//! Main daemon loop coordinating IPC requests, Hyprland events, and watchdog ticks.
//!
//! Synchronizes state across Quickshell surfaces with zero third-party dependencies.

use super::dock_state::DockState;
use super::overlap_calc::{calculate_window_overlap, default_dock_rect};
use super::serve_socket::SocketServer;
use crate::hyprland::{
    EventListener, HyprEvent, locate_sockets, query_active_window, query_clients, query_monitors,
};
use crate::system::error_status::DockError;
use crate::system::notify_socket::NotifySocket;
use std::path::{Path, PathBuf};
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

    let mut hypr_cmd_sock: Option<PathBuf> = None;
    let mut event_listener: Option<EventListener> = None;
    let mut last_reconnect = Instant::now() - Duration::from_secs(1);
    let mut last_theme_sync = Instant::now();

    NotifySocket::notify_ready();

    let watchdog_delay = NotifySocket::watchdog_usec()
        .map(|us| Duration::from_micros(us / 2))
        .unwrap_or(Duration::from_secs(15));
    let mut last_watchdog = Instant::now();

    while !term.load(Ordering::SeqCst) {
        let mut changed = false;

        // 1. Dynamic Hyprland socket reconnection if disconnected
        if (hypr_cmd_sock.is_none() || event_listener.is_none())
            && last_reconnect.elapsed() >= Duration::from_millis(500)
        {
            last_reconnect = Instant::now();
            if let Ok(sockets) = locate_sockets()
                && let Ok(listener) = EventListener::connect(&sockets.event_socket)
            {
                event_listener = Some(listener);
                hypr_cmd_sock = Some(sockets.command_socket);
                refresh_hypr_state(&mut state, hypr_cmd_sock.as_deref());
                changed = true;
            }
        }

        // 2. Process client requests over Unix domain socket
        if server.poll_and_process(&mut state, hypr_cmd_sock.as_deref()) {
            changed = true;
        }

        // 3. Process Hyprland live event stream
        let mut stream_disconnected = false;
        if let Some(ref mut listener) = event_listener {
            loop {
                match listener.next_event() {
                    Ok(Some(ev)) => match ev {
                        HyprEvent::WindowOpened(_)
                        | HyprEvent::WorkspaceChanged(_)
                        | HyprEvent::MonitorChanged => {
                            refresh_hypr_state(&mut state, hypr_cmd_sock.as_deref());
                            changed = true;
                        }
                        HyprEvent::WindowClosed(addr) => {
                            state.clear_urgent(&addr);
                            refresh_hypr_state(&mut state, hypr_cmd_sock.as_deref());
                            changed = true;
                        }
                        HyprEvent::WindowFocused(addr) => {
                            state.clear_urgent(&addr);
                            refresh_hypr_state(&mut state, hypr_cmd_sock.as_deref());
                            changed = true;
                        }
                        HyprEvent::Urgent(addr) => {
                            state.mark_urgent(&addr);
                            refresh_hypr_state(&mut state, hypr_cmd_sock.as_deref());
                            changed = true;
                        }
                        HyprEvent::Generic(_) => {}
                    },
                    Ok(None) => break,
                    Err(_) => {
                        stream_disconnected = true;
                        break;
                    }
                }
            }
        }

        if stream_disconnected {
            event_listener = None;
            hypr_cmd_sock = None;
            state.clear_all_urgent();
            refresh_hypr_state(&mut state, None);
            changed = true;
        }

        // 4. Periodic Omarchy theme sync
        if last_theme_sync.elapsed() >= Duration::from_secs(3) {
            last_theme_sync = Instant::now();
            let old_theme = state.theme.clone();
            state.sync_theme();
            if old_theme != state.theme {
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

fn refresh_hypr_state(state: &mut DockState, cmd_sock: Option<&Path>) {
    let Some(sock) = cmd_sock else {
        state.rebuild_items(&[], None);
        state.overlap = false;
        return;
    };

    let clients = query_clients(sock).unwrap_or_default();
    let active = query_active_window(sock).unwrap_or(None);
    let active_addr = active.as_ref().map(|c| c.address.as_str());
    state.rebuild_items(&clients, active_addr);

    // Calculate intellihide overlap against active monitor dock rectangle
    if let Ok(monitors) = query_monitors(sock) {
        let active_mon = monitors
            .iter()
            .find(|m| m.focused)
            .or_else(|| monitors.first());
        if let Some(mon) = active_mon {
            let dock_rect =
                default_dock_rect(mon.x, mon.y, mon.width, mon.height, state.items.len());
            let active_ws = active
                .map(|a| a.workspace_id)
                .unwrap_or(mon.active_workspace_id);
            state.overlap = calculate_window_overlap(&clients, active_ws, dock_rect);
        }
    }
}

fn register_signal_handler<F>(handler: F) -> Result<(), DockError>
where
    F: Fn() + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    let h_clone = Arc::clone(&handler);
    let _ = std::thread::Builder::new()
        .name("signal-guard".to_string())
        .spawn(move || {
            let _ = h_clone;
        });
    Ok(())
}
