pub mod dock_item;
pub mod dock_state;
pub mod handle_action;
pub mod overlap_calc;
pub mod pinned_store;
pub mod serve_socket;
pub mod server_loop;

pub use dock_item::{DockItem, WindowSummary};
pub use dock_state::DockState;
pub use handle_action::execute_action;
pub use overlap_calc::{calculate_window_overlap, default_dock_rect, rects_overlap};
pub use pinned_store::{load_pinned_ids, reorder_pinned_ids, save_pinned_ids};
pub use serve_socket::SocketServer;
pub use server_loop::run_daemon;
