//! Game configuration structures and builders.
//!
//! This module provides configuration for game window and engine settings.

use serde::{Deserialize, Serialize};

/// Configuration for the game window and engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Window title
    pub title: String,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Target frames per second (0 = unlimited)
    pub target_fps: u32,
    /// Background clear color (RGBA)
    pub clear_color: [f32; 4],
    /// Whether the window is resizable
    pub resizable: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "Insiculous 2D Game".to_string(),
            width: 800,
            height: 600,
            target_fps: 60,
            clear_color: [0.1, 0.1, 0.15, 1.0],
            resizable: true,
        }
    }
}

impl GameConfig {
    /// Create a new game configuration with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the clear color
    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.clear_color = [r, g, b, a];
        self
    }

    /// Set target FPS
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.title, "Insiculous 2D Game");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.target_fps, 60);
        assert!(config.resizable);
    }

    #[test]
    fn test_game_config_builder() {
        let config = GameConfig::new("Test Game")
            .with_size(1024, 768)
            .with_fps(120)
            .with_clear_color(0.5, 0.5, 0.5, 1.0);

        assert_eq!(config.title, "Test Game");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.target_fps, 120);
        assert_eq!(config.clear_color, [0.5, 0.5, 0.5, 1.0]);
    }
}