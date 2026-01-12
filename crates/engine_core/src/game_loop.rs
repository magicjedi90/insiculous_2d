//! Game loop functionality for the engine.
//!
//! This module provides the main game loop that drives the engine.

use crate::timing::Timer;
use crate::EngineError;
use std::time::Duration;

/// Configuration for the game loop
#[derive(Debug, Clone)]
pub struct GameLoopConfig {
    /// Target frames per second
    pub target_fps: u32,
    /// Whether to use fixed timestep
    pub fixed_timestep: bool,
}

impl Default for GameLoopConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            fixed_timestep: true,
        }
    }
}

/// The main game loop
pub struct GameLoop {
    config: GameLoopConfig,
    timer: Timer,
    running: bool,
}

impl GameLoop {
    /// Create a new game loop with the given configuration
    pub fn new(config: GameLoopConfig) -> Self {
        Self {
            config,
            timer: Timer::new(),
            running: false,
        }
    }

    /// Start the game loop
    pub fn start(&mut self) -> Result<(), EngineError> {
        self.running = true;
        self.timer.reset();
        log::info!("Game loop started");
        Ok(())
    }

    /// Stop the game loop
    pub fn stop(&mut self) {
        self.running = false;
        log::info!("Game loop stopped");
    }

    /// Check if the game loop is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the target frame duration
    pub fn target_frame_duration(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.config.target_fps as f64)
    }
    pub fn timer(&self) -> &Timer {
        &self.timer
    }
}
