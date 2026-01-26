//! Game loop manager - extracts game loop responsibilities from GameRunner
//!
//! This module implements the Single Responsibility Principle by extracting
//! game loop timing and lifecycle management from the overloaded GameRunner.

use std::time::Instant;

/// Manages game loop timing and frame delta calculations
pub struct GameLoopManager {
    last_frame_time: Instant,
    delta_time: f32,
    frame_count: u64,
    total_time: f32,
}

impl GameLoopManager {
    /// Create a new game loop manager
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            last_frame_time: now,
            delta_time: 0.0,
            frame_count: 0,
            total_time: 0.0,
        }
    }

    /// Update the game loop timing and return delta time
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_count += 1;
        self.total_time += self.delta_time;
        self.delta_time
    }

    /// Get the current delta time
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Get the total elapsed time
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Get the frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Reset the game loop (useful for scene transitions)
    pub fn reset(&mut self) {
        self.last_frame_time = Instant::now();
        self.delta_time = 0.0;
        self.frame_count = 0;
        self.total_time = 0.0;
    }
}

impl Default for GameLoopManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_game_loop_manager_creation() {
        let manager = GameLoopManager::new();
        assert_eq!(manager.delta_time(), 0.0);
        assert_eq!(manager.total_time(), 0.0);
        assert_eq!(manager.frame_count(), 0);
    }

    #[test]
    fn test_game_loop_manager_update() {
        let mut manager = GameLoopManager::new();
        
        sleep(Duration::from_millis(10));
        let dt = manager.update();
        assert!(dt > 0.0, "Delta time should be positive");
        assert!(dt < 0.02, "Delta time should be reasonable");
        assert_eq!(manager.frame_count(), 1);
        assert!(manager.total_time() > 0.0);
    }

    #[test]
    fn test_game_loop_manager_multiple_updates() {
        let mut manager = GameLoopManager::new();
        
        let dt1 = manager.update();
        sleep(Duration::from_millis(5));
        let dt2 = manager.update();
        
        assert_eq!(manager.frame_count(), 2);
        assert!(manager.total_time() > 0.0, "Total time should accumulate");
        assert_eq!(manager.total_time(), dt1 + dt2, "Total time should equal sum of deltas");
    }

    #[test]
    fn test_game_loop_manager_reset() {
        let mut manager = GameLoopManager::new();
        
        manager.update();
        manager.update();
        assert_eq!(manager.frame_count(), 2);
        
        manager.reset();
        assert_eq!(manager.delta_time(), 0.0);
        assert_eq!(manager.total_time(), 0.0);
        assert_eq!(manager.frame_count(), 0);
    }
}