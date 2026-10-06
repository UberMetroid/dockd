//! Unix domain socket IPC server for dockd clients (Quickshell / CLI).
//!
//! Provides non-blocking JSON command evaluation and client event broadcast.

use super::dock_state::DockState;
use super::handle_action::execute_action;
use crate::syntax::json_parse::parse;
use crate::syntax::json_render::render;
use crate::syntax::json_value::JsonValue;
use crate::system::error_status::DockError;
use crate::system::listen_fds::try_listen_fds;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

pub struct SocketServer {
    listener: UnixListener,
    socket_path: Option<PathBuf>,
    clients: Vec<UnixStream>,
}

impl SocketServer {
    pub fn bind_default() -> Result<Self, DockError> {
        if let Some(inherited) = try_listen_fds()? {
            return Ok(Self {
                listener: inherited,
                socket_path: None,
                clients: Vec::new(),
            });
        }

        let sock_path = default_socket_path();
        if sock_path.exists() {
            let _ = std::fs::remove_file(&sock_path);
        }

        if let Some(parent) = sock_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let listener = UnixListener::bind(&sock_path)?;
        listener.set_nonblocking(true)?;

        Ok(Self {
            listener,
            socket_path: Some(sock_path),
            clients: Vec::new(),
        })
    }

    pub fn poll_and_process(
        &mut self,
        state: &mut DockState,
        hypr_cmd_sock: Option<&Path>,
    ) -> bool {
        let mut state_changed = false;

        // Accept new connections
        while let Ok((stream, _)) = self.listener.accept() {
            let _ = stream.set_nonblocking(true);
            self.clients.push(stream);
        }

        let mut to_remove = Vec::new();

        for (idx, client) in self.clients.iter_mut().enumerate() {
            let mut reader = BufReader::new(client);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => to_remove.push(idx),
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let resp = match parse(trimmed) {
                        Ok(req) => match execute_action(&req, state, hypr_cmd_sock) {
                            Ok(res) => {
                                state_changed = true;
                                res
                            }
                            Err(e) => json_err(&e.to_string()),
                        },
                        Err(e) => json_err(&format!("Invalid JSON request: {e}")),
                    };

                    let mut out = render(&resp);
                    out.push('\n');
                    let stream = reader.into_inner();
                    if stream.write_all(out.as_bytes()).is_err() {
                        to_remove.push(idx);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => to_remove.push(idx),
            }
        }

        for idx in to_remove.into_iter().rev() {
            if idx < self.clients.len() {
                self.clients.swap_remove(idx);
            }
        }

        state_changed
    }

    pub fn broadcast_state(&mut self, state: &DockState) {
        let mut msg = render(&state.to_json());
        msg.push('\n');

        let mut to_remove = Vec::new();
        for (idx, client) in self.clients.iter_mut().enumerate() {
            if client.write_all(msg.as_bytes()).is_err() {
                to_remove.push(idx);
            }
        }

        for idx in to_remove.into_iter().rev() {
            if idx < self.clients.len() {
                self.clients.swap_remove(idx);
            }
        }
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        if let Some(ref path) = self.socket_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn default_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("dockd.sock")
    } else {
        PathBuf::from("/tmp/dockd.sock")
    }
}

fn json_err(msg: &str) -> JsonValue {
    let mut map = BTreeMap::new();
    map.insert("status".to_string(), JsonValue::String("error".to_string()));
    map.insert("message".to_string(), JsonValue::String(msg.to_string()));
    JsonValue::Object(map)
}
