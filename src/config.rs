use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub offset_x: f32,
    pub offset_y: f32,
    pub color: [u8; 3],
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub stroke_width: f32,
    pub stroke_color: [u8; 3],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            color: [255, 0, 0],
            inner_radius: 3.0,
            outer_radius: 3.5,
            stroke_width: 1.0,
            stroke_color: [0, 0, 0],
        }
    }
}

fn config_path() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("aimx_config.json")
}

impl Config {
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), json);
        }
    }
}
