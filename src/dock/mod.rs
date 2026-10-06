pub mod dock_item;
pub mod dock_state;
pub mod handle_action;
pub mod serve_socket;
pub mod server_loop;

pub use dock_item::{DockItem, WindowSummary};
pub use dock_state::DockState;
pub use handle_action::execute_action;
pub use serve_socket::SocketServer;
pub use server_loop::run_daemon;
