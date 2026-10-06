//! dockd — pure-std application dock and window manager daemon for Omarchy & Hyprland.

pub mod dock;
pub mod hyprland;
#[cfg(test)]
pub mod qa;
pub mod syntax;
pub mod system;

pub use dock::run_daemon;
pub use system::DockError;
