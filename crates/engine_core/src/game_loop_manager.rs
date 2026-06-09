//! Game loop manager - extracts game loop responsibilities from GameRunner
//!
//! This module implements the Single Responsibility Principle by extracting
//! game loop timing and lifecycle management from the overloaded GameRunner.

use std::time::{Duration, Instant};

/// Upper bound on a single frame's delta time, in seconds.
///
/// Stalls longer than this (debugger pause, OS suspend, window drag) are
/// reported as one clamped step instead of the real elapsed time, so
/// delta-scaled game logic and physics don't leap or explode on resume.
pub const MAX_DELTA_TIME: f32 = 0.1;

/// Manages game loop timing and frame delta calculations
pub struct GameLoopManager {
    last_frame_time: Instant,
    delta_time: f32,
    frame_count: u64,
    total_time: f32,
    /// Minimum duration of a frame, derived from the target FPS.
    /// `None` means uncapped (rely on vsync / present mode).
    min_frame_time: Option<Duration>,
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
            min_frame_time: None,
        }
    }

    /// Cap the frame rate at `fps`. Pass 0 to uncap (vsync still applies).
    pub fn set_target_fps(&mut self, fps: u32) {
        self.min_frame_time = if fps == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(1.0 / fps as f64))
        };
    }

    /// Sleep out the remainder of the current frame's budget.
    ///
    /// Call once per frame after update/render work. No-op when uncapped or
    /// when the frame already used its full budget.
    pub fn throttle(&self) {
        if let Some(min_frame_time) = self.min_frame_time {
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < min_frame_time {
                std::thread::sleep(min_frame_time - elapsed);
            }
        }
    }

    /// Update the game loop timing and return delta time
    ///
    /// The returned delta is clamped to [`MAX_DELTA_TIME`] so long stalls
    /// don't propagate huge timesteps into game logic.
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.delta_time = (now - self.last_frame_time).as_secs_f32().min(MAX_DELTA_TIME);
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
    fn test_delta_time_is_clamped_after_a_stall() {
        let mut manager = GameLoopManager::new();

        // Simulate a stall longer than the clamp window.
        sleep(Duration::from_millis(120));
        let dt = manager.update();
        assert!(
            dt <= MAX_DELTA_TIME,
            "delta {} exceeds clamp {}",
            dt,
            MAX_DELTA_TIME
        );
    }

    #[test]
    fn test_throttle_enforces_target_fps() {
        let mut manager = GameLoopManager::new();
        manager.set_target_fps(100); // 10ms frame budget

        manager.update();
        let frame_start = Instant::now();
        manager.throttle();
        assert!(
            frame_start.elapsed() >= Duration::from_millis(5),
            "throttle should sleep out most of the 10ms frame budget"
        );
    }

    #[test]
    fn test_throttle_is_noop_when_uncapped() {
        let mut manager = GameLoopManager::new();
        manager.set_target_fps(0);

        manager.update();
        let frame_start = Instant::now();
        manager.throttle();
        assert!(
            frame_start.elapsed() < Duration::from_millis(5),
            "uncapped throttle must not sleep"
        );
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