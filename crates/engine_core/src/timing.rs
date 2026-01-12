//! Timing functionality for the engine.
//!
//! This module provides utilities for tracking time and managing frame rates.

use std::time::{Duration, Instant};

/// A timer for tracking elapsed time
pub struct Timer {
    start_time: Instant,
    last_update: Instant,
    delta_time: Duration,
    elapsed: Duration,
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            delta_time: Duration::from_secs(0),
            elapsed: Duration::from_secs(0),
        }
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_update = now;
        self.delta_time = Duration::from_secs(0);
        self.elapsed = Duration::from_secs(0);
    }

    /// Update the timer
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_update);
        self.elapsed = now.duration_since(self.start_time);
        self.last_update = now;
    }

    /// Get the delta time (time since last update)
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Get the delta time in seconds
    pub fn delta_seconds(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    /// Get the elapsed time (time since timer creation or reset)
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Get the elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
