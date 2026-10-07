pub mod error_status;
pub mod launch_binary;
pub mod listen_fds;
pub mod notify_socket;
pub mod resolve_icon;
pub mod scan_desktop;
pub mod theme_sync;

pub use error_status::DockError;
pub use launch_binary::{launch_exec, launch_location};
pub use listen_fds::try_listen_fds;
pub use notify_socket::NotifySocket;
pub use resolve_icon::resolve_icon_path;
pub use scan_desktop::{DesktopEntry, scan_desktop_entries};
pub use theme_sync::{ThemePalette, load_omarchy_theme};
