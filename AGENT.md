# dockd — Engineering Rules (AGENT.md)

Rules for human and agent contributors working in this repository. Derived from the `UberMetroid` architectural standards: a pure standard-library Linux dock and window manager daemon whose defining property is that every shipped line is provably ours.

**Precedence:** Where this file and CI disagree, this file wins. A rule that is not enforced anywhere is a bug in this file, not a suggestion to ignore.

**Reading the markers:**
- `[GATE]` — a named automated test in `cargo test` fails the build if broken.
- `[POLICY]` — not machine-enforced; human-read standard for architecture and restraint.

---

## 1. Zero External Dependencies (Zero Crates)

- `[GATE]` `Cargo.toml` has no `[dependencies]` section.
- `[GATE]` No `extern crate` anywhere in `src/`.
- `[POLICY]` All protocol parsers (JSON tokenizing/parsing/rendering, Hyprland IPC, Unix domain socket framing), `.desktop` parsers, and systemd integration are implemented using Rust `std`, `core`, `alloc`, and explicit platform FFI.

**Why:** A zero-crate tree can be audited, vendored, and built anywhere with only the Rust toolchain. Zero supply-chain vulnerabilities, zero external breaking changes.

---

## 2. The Page Rule (Code Layout & Sizing)

- `[GATE]` Every committed `.rs` file must be between **16 and 256 lines** inclusive, counted as `content.lines().count()` (comments and blank lines count).
- `[GATE]` **Shim Exemption (Floor Only):** A `mod.rs` or `lib.rs` whose non-blank, non-comment lines consist strictly of module declarations (`mod `, `pub mod `) or re-exports (`use `, `pub use `) is exempt from the 16-line floor. The 256-line ceiling still applies.
- `[GATE]` **Directory Density ($\le 8$ files):** At most **8 `.rs` files per directory**, tests included.
- `[GATE]` **Banned Drawer Names:** `util.rs`, `utils.rs`, `helper.rs`, `helpers.rs`, `common.rs`, `misc.rs`, `shared.rs`, `base.rs`, `core.rs`.
- `[POLICY]` Action files lead with a verb (`focus_window.rs`, `scan_desktop.rs`, `dispatch_command.rs`). State files name what they own (`client_table.rs`, `dock_state.rs`).
- `[POLICY]` Never pad files with blanks or comments to reach the 16-line floor; fold under-sized files into their caller or sibling.

---

## 3. Pure Rust & The Zero-Panic Invariant

- `[GATE]` No bare `.unwrap()` or `.expect()` in production code. Permitted only inside `#[cfg(test)]`.
- `[POLICY]` All errors must be propagated via domain-specific `Result<T, Error>` with hand-written `Display` and `std::error::Error` implementations.
- `[GATE]` `#![deny(unsafe_code)]` at crate root.
- `[POLICY]` If raw syscalls/FFI are needed (e.g. `sd_notify` socket calls), `#![allow(unsafe_code)]` is permitted only on that specific file, accompanied by a mandatory `// SAFETY:` comment detailing invariants.

---

## 4. Native systemd Citizenship

- `[POLICY]` Designed to run as a user service under `systemd --user` (`systemd/user/dockd.service`).
- `[POLICY]` Socket activation supported via `$LISTEN_FDS` (FD 3) with fallback to native Unix socket bind (`$XDG_RUNTIME_DIR/dockd.sock`).
- `[POLICY]` Lifecycle signals emitted to `$NOTIFY_SOCKET`: `READY=1`, dynamic `WATCHDOG=1`, and `STOPPING=1`.
- `[POLICY]` Exit codes honor standard `sysexits.h` conventions (`0` for OK, `78` for `EX_CONFIG`, `69` for `EX_UNAVAILABLE`).
- `[POLICY]` Logging formatted for `systemd-journald` consumption.

---

## 5. Linux Zero-Trust Safety & Subprocesses

- `[POLICY]` File descriptors and sockets must always set `SOCK_CLOEXEC` / `O_CLOEXEC`.
- `[POLICY]` Never spawn shell interpreters (`/bin/sh -c`). Subprocesses must be executed directly via `std::process::Command::new(binary)` with sanitized environments.
