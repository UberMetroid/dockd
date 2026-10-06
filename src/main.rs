//! CLI entrypoint for dockd daemon and dock control client.
//!
//! Handles daemon execution, CLI action dispatch, version queries, and sysexits termination.

use dockd::dock::run_daemon;
use dockd::dock::serve_socket::default_socket_path;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.first().is_some_and(|s| s == "--version" || s == "-v") {
        println!("dockd {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if args.first().is_some_and(|s| s == "--help" || s == "-h") {
        print_help();
        return ExitCode::SUCCESS;
    }

    // Default to daemon mode when invoked with no args or --daemon
    if args.is_empty() || args.first().is_some_and(|s| s == "--daemon" || s == "-d") {
        match run_daemon() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("dockd daemon error: {e}");
                ExitCode::from(e.exit_code() as u8)
            }
        }
    } else {
        // CLI client mode: send command to running daemon socket
        match send_client_command(&args) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("dockd client error: {e}");
                ExitCode::from(69u8) // EX_UNAVAILABLE
            }
        }
    }
}

fn send_client_command(args: &[String]) -> Result<String, String> {
    let sock_path = default_socket_path();
    let mut stream = UnixStream::connect(&sock_path).map_err(|e| {
        format!(
            "Could not connect to dockd socket at {}: {e}",
            sock_path.display()
        )
    })?;

    let cmd_name = args.first().map(String::as_str).unwrap_or("");
    let payload = match cmd_name {
        "state" => "{\"action\":\"get_state\"}".to_string(),
        "pin" => {
            let id = args.get(1).ok_or("Provide desktop ID to pin")?;
            format!("{{\"action\":\"pin\",\"desktop_id\":\"{id}\"}}")
        }
        "unpin" => {
            let id = args.get(1).ok_or("Provide desktop ID to unpin")?;
            format!("{{\"action\":\"unpin\",\"desktop_id\":\"{id}\"}}")
        }
        "focus" => {
            let addr = args.get(1).ok_or("Provide window address to focus")?;
            format!("{{\"action\":\"focus\",\"address\":\"{addr}\"}}")
        }
        "minimize" => {
            let addr = args.get(1).ok_or("Provide window address to minimize")?;
            format!("{{\"action\":\"minimize\",\"address\":\"{addr}\"}}")
        }
        "close" => {
            let addr = args.get(1).ok_or("Provide window address to close")?;
            format!("{{\"action\":\"close\",\"address\":\"{addr}\"}}")
        }
        "tile" => {
            let addr = args.get(1).ok_or("Provide window address to tile")?;
            format!("{{\"action\":\"tile\",\"address\":\"{addr}\"}}")
        }
        "float" => {
            let addr = args.get(1).ok_or("Provide window address to float")?;
            format!("{{\"action\":\"float\",\"address\":\"{addr}\"}}")
        }
        "launch" => {
            let app = args.get(1).ok_or("Provide application ID or exec name")?;
            format!("{{\"action\":\"launch\",\"desktop_id\":\"{app}\"}}")
        }
        "profile" => {
            let p = args
                .get(1)
                .ok_or("Provide layout profile: general, windows, or mac")?;
            format!("{{\"action\":\"set_profile\",\"profile\":\"{p}\"}}")
        }
        other => return Err(format!("Unknown command '{other}'. Run dockd --help")),
    };

    stream
        .write_all(format!("{payload}\n").as_bytes())
        .map_err(|e| format!("Failed to write to dockd socket: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("Failed to read response from dockd: {e}"))?;

    Ok(response.trim().to_string())
}

fn print_help() {
    println!(
        "dockd {} — Omarchy App Dock & Window Manager Daemon",
        env!("CARGO_PKG_VERSION")
    );
    println!("\nUsage:");
    println!("  dockd [--daemon]                 Run the long-running daemon (default)");
    println!("  dockd state                      Query current dock launcher state as JSON");
    println!("  dockd launch <desktop-id>        Launch an application");
    println!("  dockd focus <address>            Focus a window by Hyprland address");
    println!("  dockd minimize <address>         Minimize a window");
    println!("  dockd close <address>            Close a window");
    println!("  dockd tile <address>             Set window to tiled layout");
    println!("  dockd float <address>            Set window to floating layout");
    println!("  dockd pin <desktop-id>           Pin application to dock");
    println!("  dockd unpin <desktop-id>         Unpin application from dock");
    println!("  dockd profile <general|windows|mac> Set layout preset profile");
    println!("  dockd --version                  Show version");
    println!("  dockd --help                     Show this help");
}
