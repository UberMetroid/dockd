//! Intellihide overlap geometry calculations for the dock panel.
//!
//! Detects whether any active or floating window intersects the dock region.

use crate::hyprland::client_table::HyprClient;

pub fn rects_overlap(r1: (i64, i64, i64, i64), r2: (i64, i64, i64, i64)) -> bool {
    let (x1, y1, w1, h1) = r1;
    let (x2, y2, w2, h2) = r2;

    if w1 <= 0 || h1 <= 0 || w2 <= 0 || h2 <= 0 {
        return false;
    }

    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

pub fn default_dock_rect(
    mon_x: i64,
    mon_y: i64,
    mon_w: i64,
    mon_h: i64,
    item_count: usize,
) -> (i64, i64, i64, i64) {
    let dock_h = 56;
    let computed_w = ((item_count as i64) * 52 + 48).clamp(200, (mon_w - 32).max(200));
    let dock_x = mon_x + (mon_w - computed_w) / 2;
    let dock_y = mon_y + mon_h - dock_h - 8;
    (dock_x, dock_y, computed_w, dock_h)
}

pub fn calculate_window_overlap(
    clients: &[HyprClient],
    active_workspace_id: i64,
    dock_rect: (i64, i64, i64, i64),
) -> bool {
    for client in clients {
        if !client.mapped || client.hidden || client.is_minimized() {
            continue;
        }

        // Only windows on the current active workspace can overlap
        if client.workspace_id != active_workspace_id {
            continue;
        }

        let win_rect = (client.at.0, client.at.1, client.size.0, client.size.1);
        if rects_overlap(win_rect, dock_rect) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rects_overlap() {
        let dock = (500, 1000, 400, 56);
        let win_above = (500, 200, 400, 400);
        let win_overlapping = (500, 800, 400, 300);

        assert!(!rects_overlap(dock, win_above));
        assert!(rects_overlap(dock, win_overlapping));
    }

    #[test]
    fn test_calculate_window_overlap() {
        let dock = (500, 1000, 400, 56);
        let mut client = HyprClient {
            address: "0x1".to_string(),
            mapped: true,
            hidden: false,
            at: (500, 900),
            size: (400, 200),
            workspace_id: 1,
            workspace_name: "1".to_string(),
            monitor_id: 0,
            class: "wezterm".to_string(),
            title: "terminal".to_string(),
            initial_class: "wezterm".to_string(),
            initial_title: "terminal".to_string(),
            pid: 100,
            xwayland: false,
            floating: false,
            fullscreen: false,
        };

        assert!(calculate_window_overlap(&[client.clone()], 1, dock));

        // Different workspace should not overlap
        assert!(!calculate_window_overlap(&[client.clone()], 2, dock));

        // Floating window on different workspace should not overlap
        let mut float_other_ws = client.clone();
        float_other_ws.floating = true;
        float_other_ws.workspace_id = 2;
        assert!(!calculate_window_overlap(&[float_other_ws], 1, dock));

        // Minimized should not overlap
        client.hidden = true;
        assert!(!calculate_window_overlap(&[client], 1, dock));
    }
}
