//! Monitor layout tracking and window geometry arrangement calculations.
//!
//! Computes tile, snap, and center coordinates relative to monitor scaling and reserved areas.

use crate::syntax::json_value::JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub struct HyprMonitor {
    pub id: i64,
    pub name: String,
    pub width: i64,
    pub height: i64,
    pub x: i64,
    pub y: i64,
    pub scale: f64,
    pub focused: bool,
    pub reserved: [i64; 4], // [top, bottom, left, right]
}

impl HyprMonitor {
    pub fn from_json(val: &JsonValue) -> Option<Self> {
        let id = val.get_i64("id")?;
        let name = val.get_str("name")?.to_string();
        let width = val.get_i64("width").unwrap_or(1920);
        let height = val.get_i64("height").unwrap_or(1080);
        let x = val.get_i64("x").unwrap_or(0);
        let y = val.get_i64("y").unwrap_or(0);
        let scale = val.get_f64("scale").unwrap_or(1.0);
        let focused = val.get_bool("focused").unwrap_or(false);

        let mut reserved = [0i64; 4];
        if let Some(res_arr) = val.get("reserved").and_then(JsonValue::as_array) {
            for (idx, r) in res_arr.iter().take(4).enumerate() {
                reserved[idx] = r.as_i64().unwrap_or(0);
            }
        }

        Some(Self {
            id,
            name,
            width,
            height,
            x,
            y,
            scale,
            focused,
            reserved,
        })
    }

    pub fn work_area(&self) -> (i64, i64, i64, i64) {
        let eff_scale = if self.scale > 0.0 { self.scale } else { 1.0 };
        let w = ((self.width as f64) / eff_scale).round() as i64;
        let h = ((self.height as f64) / eff_scale).round() as i64;

        let left = self.reserved[2];
        let right = self.reserved[3];
        let top = self.reserved[0];
        let bottom = self.reserved[1];

        let area_x = self.x + left;
        let area_y = self.y + top;
        let area_w = (w - left - right).max(200);
        let area_h = (h - top - bottom).max(150);

        (area_x, area_y, area_w, area_h)
    }

    pub fn snap_left_geometry(&self) -> (i64, i64, i64, i64) {
        let (x, y, w, h) = self.work_area();
        let half_w = w / 2;
        (x, y, half_w, h)
    }

    pub fn snap_right_geometry(&self) -> (i64, i64, i64, i64) {
        let (x, y, w, h) = self.work_area();
        let half_w = w / 2;
        (x + half_w, y, w - half_w, h)
    }

    pub fn center_geometry(&self, win_w: i64, win_h: i64) -> (i64, i64) {
        let (x, y, w, h) = self.work_area();
        let target_x = x + ((w - win_w) / 2).max(0);
        let target_y = y + ((h - win_h) / 2).max(0);
        (target_x, target_y)
    }
}
