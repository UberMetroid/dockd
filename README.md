# dockd

> **Pure-standard-library Rust application dock and window manager daemon for Omarchy & Hyprland.**

`dockd` is an opinionated, high-performance desktop dock and window manager daemon created for the [Omarchy](https://github.com/omacom/omarchy) desktop environment. It couples a native **Quickshell** plugin frontend with a pure standard-library **Rust** daemon (`dockd`) running under native `systemd --user` supervision.

---

## Architecture & Design Principles

1. **Zero External Crates (UberMetroid Standard)**:
   - Hand-rolled JSON tokenizer, parser, and serializer in pure `std`.
   - Hand-rolled Hyprland IPC, non-blocking socket streaming, and event buffering.
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

## Features & Capabilities

### Compositor Resilience & Hot-Reload
- **Dynamic Hyprland Socket Reconnection**: Continuously reconnects to Hyprland's command and event sockets if Hyprland reloads, restarts, or if `dockd` starts prior to compositor initialization.
- **Urgency Tracking**: Live tracking of `urgent>>ADDR` events from Hyprland, flagged across window summaries and launcher items (`urgent: true`), triggering pulsating glowing visual feedback in the dock.

### Desktop UX & Window Management
- **Multi-Window Hover Previews**: Hovering over an application with multiple open instances opens a preview card displaying each window title, workspace identifier, focused state, quick-focus switcher, and close buttons.
- **Pin Reordering**: Reorder pinned launchers on the fly via socket protocol (`{"action":"reorder","desktop_id":"...","index":2}`) or CLI (`dockd reorder <desktop-id> <index>`).
- **Per-Monitor Window Filtering**: Filter running tasks by current monitor ID in the daemon state, toggleable directly from `SettingsPopup.qml` or via `dockd monitor <id|all>`.
- **Intellihide Overlap Detection**: Real-time geometric calculation determining whether any active or floating window intersects the dock panel rectangle, broadcasting `overlap: true/false` for smooth sliding auto-hide.
- **Window Controls**: Focus, restore, minimize to `special:minimized`, toggle floating/tiling, snap left/right half, and terminate processes.

### Omarchy Ecosystem & Theming
- **Dynamic Omarchy Theme Synchronization**: Synchronizes palette definitions from `~/.config/omarchy/current/theme/` or `~/.config/omarchy/shell.json`, propagating accent, card, background, border, and urgent colors into QML.
- **Active Icon Theme Auto-Detection**: Auto-detects active desktop icon themes from `~/.config/gtk-3.0/settings.ini` and `~/.config/gtk-4.0/settings.ini` (`gtk-icon-theme-name`), cascading gracefully through theme hierarchies and standard fallbacks.

---

## Directory Structure

```text
dockd/
├── AGENT.md                 # House Laws and Engineering Rules
├── Cargo.toml               # Zero-dependency package manifest
├── install.sh               # One-step installer & uninstaller driver
├── uninstall.sh             # Clean uninstaller script
├── plugin/                  # Omarchy Quickshell native plugin
│   ├── manifest.json        # Plugin definition (org.ubermetroid.dockd)
│   ├── DockPanel.qml        # Layer-shell dock window surface
│   ├── BarWidget.qml        # Status-bar control widget
│   └── components/
│       ├── AppMenu.qml      # Window arrangement context menu
│       ├── DockIcon.qml     # Interactive launcher icon with urgency glow & previews
│       └── SettingsPopup.qml# Preference, profile, and filter selector
├── src/
│   ├── lib.rs               # Library root shim
│   ├── main.rs              # CLI entrypoint and client driver
│   ├── dock/                # Dock state machine, IPC server, overlap & pin store
│   ├── hyprland/            # Hyprland IPC client, client table, events & layouts
│   ├── syntax/              # Pure-std JSON lexer, parser, and serializer
│   ├── system/              # Systemd integration, desktop scanning, theme sync
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
cargo clippy -- -D warnings
cargo fmt --check
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

# Reorder pinned launchers
dockd reorder firefox.desktop 0

# Filter dock windows by monitor ID (or 'all' to show all windows)
dockd monitor 0
dockd monitor all

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

### 1. Universal Web Installer (Recommended)
One-line automated installation for Omarchy and Hyprland desktops:

```bash
curl -fsSL https://ubermetroid.github.io/dockd/install.sh | bash
```

### 2. DNF / RPM (Fedora, RHEL, CentOS)
Install native RPM package directly via the web installer or prebuilt package:

```bash
# Automated via web installer:
curl -fsSL https://ubermetroid.github.io/dockd/install.sh | bash -s -- --dnf

# Or install local / downloaded package:
sudo dnf install ./dist/dockd-0.1.0-1.*.rpm
```

### 3. DEB / APT (Debian, Ubuntu)
Install native `.deb` package directly via the web installer or prebuilt package:

```bash
# Automated via web installer:
curl -fsSL https://ubermetroid.github.io/dockd/install.sh | bash -s -- --deb

# Or install local / downloaded package:
sudo apt install ./dist/dockd_0.1.0-1_*.deb
```

### 4. Arch Linux & Omarchy (PKGBUILD / Pacman)
Omarchy runs natively on Arch Linux. Build and install via PKGBUILD:

```bash
# Automated via web installer:
curl -fsSL https://ubermetroid.github.io/dockd/install.sh | bash -s -- --pkgbuild

# Or build from source checkout:
cd packaging/arch && makepkg -si
```

### 5. Local Source Installation
From a clone of this repository:

```bash
./install.sh
```

---

## Uninstallation

Cleanly remove `dockd`, systemd user units, sockets, and Omarchy plugins:

```bash
# 1. Via installed CLI helper (if already installed)
dockd-uninstall

# 2. Via canonical web uninstaller
curl -fsSL https://ubermetroid.github.io/dockd/uninstall.sh | bash

# 3. Complete purge (including pinned app configuration)
dockd-uninstall --purge
# or: curl -fsSL https://ubermetroid.github.io/dockd/uninstall.sh | bash -s -- --purge
```

---

## Enabling & Starting dockd

After installation, enable the systemd socket activation unit:

```bash
systemctl --user enable --now dockd.socket
```

Enable the plugin inside Omarchy:

```bash
omarchy plugin enable org.ubermetroid.dockd
```

---

## Clean Uninstallation

`dockd` includes a comprehensive uninstaller that cleanly stops and disables systemd user units, removes socket files, unregisters the Omarchy plugin, deletes binaries, and optionally purges preferences:

### Via uninstaller CLI (installed to PATH)
```bash
dockd-uninstall
# Or purge configuration files as well:
dockd-uninstall --purge
```

### Via installer script
```bash
./install.sh --uninstall
```

### Via repository script
```bash
./uninstall.sh --purge
```

### Via package managers
```bash
# Fedora / RPM
sudo dnf remove dockd

# Debian / Ubuntu
sudo apt remove dockd

# Arch Linux
sudo pacman -R dockd
```

---

## License

GPL-2.0-or-later. Created for the **UberMetroid** organization.
