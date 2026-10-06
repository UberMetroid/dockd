# dockd

> **Pure-standard-library Rust application dock and window manager daemon for Omarchy & Hyprland.**

`dockd` is an opinionated, high-performance desktop dock and window manager daemon created for the [Omarchy](https://github.com/omacom/omarchy) desktop environment. It couples a native **Quickshell** plugin frontend with a pure standard-library **Rust** daemon (`dockd`) running under native `systemd --user` supervision.

---

## Architecture & Design Principles

1. **Zero External Crates (UberMetroid Standard)**:
   - Hand-rolled JSON tokenizer, parser, and serializer in pure `std`.
   - Hand-rolled Hyprland IPC and Unix domain socket event streaming.
   - Zero `crates.io` dependencies, zero supply-chain risk, <600 KB stripped binary.

2. **The Page Rule Enforcement**:
   - Every Rust source file is strictly between **16 and 256 lines** (with shims exempt from floor only).
   - Maximum of **8 `.rs` files per directory**.
   - Banned generic drawer filenames (`util.rs`, `common.rs`, etc.).
   - Verified on every build via automated tests (`cargo test`).

3. **Zero-Panic Invariant**:
   - Zero bare `.unwrap()` or `.expect()` calls in production code.
   - All errors are modeled via domain-typed `DockError` with sysexits-aligned exit codes.

4. **Native systemd Citizenship**:
   - Socket activation supported on `$LISTEN_FDS` (FD 3) via `dockd.socket`.
   - Systemd lifecycle heartbeats: `READY=1`, dynamic `WATCHDOG=1`, and `STOPPING=1`.
   - Strict sandboxing directives (`ProtectSystem=strict`, `ProtectHome=read-only`, `NoNewPrivileges=true`).

---

## Features

- **App Dock & Pinned Launchers**:
  - Pinned applications persisted cleanly in `~/.config/omarchy/dockd-pinned.json`.
  - Running application indicators (dots, glowing active plates).
  - Multi-instance window grouping with window titles and workspace indicators.
- **Hyprland Window Management**:
  - Focus / restore minimized windows.
  - Minimize to `special:minimized`.
  - Window arrangement: Tile, Float, Snap Left Half, Snap Right Half, Center.
  - Clean window termination and process force-kill.
- **File Shortcuts**:
  - Quick-launchers for Home (`~`), Downloads (`~/Downloads`), and Trash (`trash:///`).
- **Layout Profiles**:
  - **General**: Centered floating pill with smooth hover scaling and rounded geometry.
  - **Windows**: Taskbar-style bottom anchor with running window indicator plates.
  - **macOS**: Centered dock with magnification and separated folder shortcuts.
- **Status Bar Integration**:
  - `BarWidget.qml` status bar widget for toggling visibility and switching profiles on the fly.

---

## Directory Structure

```text
dockd/
├── AGENT.md                 # House Laws and Engineering Rules
├── Cargo.toml               # Zero-dependency package manifest
├── install.sh               # One-step build and installation driver
├── plugin/                  # Omarchy Quickshell native plugin
│   ├── manifest.json        # Plugin definition (org.ubermetroid.dockd)
│   ├── DockPanel.qml        # Layer-shell dock window surface
│   ├── BarWidget.qml        # Status-bar control widget
│   └── components/
│       ├── AppMenu.qml      # Window arrangement context menu
│       ├── DockIcon.qml     # Interactive launcher icon with badges
│       └── SettingsPopup.qml# Preference and profile selector
├── src/
│   ├── lib.rs               # Library root shim
│   ├── main.rs              # CLI entrypoint and client driver
│   ├── dock/                # Dock state machine, IPC server, and daemon loop
│   ├── hyprland/            # Hyprland IPC client, client table, and layouts
│   ├── syntax/              # Pure-std JSON lexer, parser, and serializer
│   ├── system/              # Systemd integration, desktop scanning, launcher
│   └── qa/                  # Automated verification gates and Page Rule lints
└── systemd/user/
    ├── dockd.service        # systemd user service unit
    └── dockd.socket         # systemd socket activation unit
```

---

## Building & Testing

### Running the Test Suite
All gates and Page Rule invariants are verified on every test run:

```bash
cargo test
```

### Building the Release Binary
```bash
cargo build --release
```

### Running the CLI Client
While `dockd` runs as a daemon, it also serves as a command-line controller:

```bash
# Query current dock state JSON
dockd state

# Focus or minimize a window
dockd focus 0x55a3f120
dockd minimize 0x55a3f120

# Snap window to left or right screen half
dockd tile 0x55a3f120
dockd float 0x55a3f120

# Pin or unpin applications
dockd pin firefox.desktop
dockd unpin firefox.desktop

# Switch profile
dockd profile windows
```

---

## Installation

Run the automated installer:

```bash
bash install.sh
```

Enable the systemd user socket:

```bash
systemctl --user enable --now dockd.socket
```

Enable the plugin in Omarchy:

```bash
omarchy plugin enable org.ubermetroid.dockd
```

---

## License

GPL-2.0-or-later. Created for the **UberMetroid** organization.
