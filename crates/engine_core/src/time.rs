use std::time::{Duration, Instant};

/// Tracks frame‑to‑frame timing for fixed‑timestep updates.
pub struct ApplicationClock {
    previous_instant: Instant,
    frame_delta:      Duration,
}

impl ApplicationClock {
    pub fn new() -> Self {
        Self {
            previous_instant: Instant::now(),
            frame_delta:      Duration::ZERO,
        }
    }

    pub fn advance_frame(&mut self) {
        let now = Instant::now();
        self.frame_delta = now - self.previous_instant;
        self.previous_instant = now;
    }

    pub fn delta_seconds(&self) -> f32 {
        self.frame_delta.as_secs_f32()
    }
}
