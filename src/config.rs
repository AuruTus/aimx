use log::{debug, info, warn};
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
    #[serde(default = "default_close_action")]
    pub close_action: String,
}

fn default_close_action() -> String {
    "ask".to_string()
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
            close_action: default_close_action(),
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
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(s) => match serde_json::from_str(&s) {
                Ok(cfg) => {
                    info!("loaded config from {}", path.display());
                    cfg
                }
                Err(e) => {
                    warn!("corrupt config at {}: {e}, using defaults", path.display());
                    Self::default()
                }
            },
            Err(_) => {
                debug!("no config at {}, using defaults", path.display());
                Self::default()
            }
        }
    }

    /// Minimum window size needed to fully contain the crosshair.
    pub fn window_size(&self) -> f32 {
        let r = self.outer_radius.max(self.inner_radius) + self.stroke_width;
        // diameter + padding
        (r * 2.0 + 4.0).ceil().max(16.0)
    }

    pub fn save(&self) {
        let path = config_path();
        match serde_json::to_string_pretty(self) {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(_) => info!("saved config to {}", path.display()),
                Err(e) => warn!("failed to write config: {e}"),
            },
            Err(e) => warn!("failed to serialize config: {e}"),
        }
    }
}
