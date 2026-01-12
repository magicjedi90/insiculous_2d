//! Sound data types and playback settings.

use std::sync::atomic::{AtomicU32, Ordering};

/// Unique identifier for a loaded sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle {
    pub(crate) id: u32,
}

impl SoundHandle {
    /// Create a new sound handle with a unique ID.
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Get the numeric ID of this handle.
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Settings for sound playback.
#[derive(Debug, Clone)]
pub struct SoundSettings {
    /// Volume level (0.0 = silent, 1.0 = full volume).
    pub volume: f32,
    /// Playback speed (1.0 = normal, 2.0 = double speed).
    pub speed: f32,
    /// Whether the sound should loop.
    pub looping: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            speed: 1.0,
            looping: false,
        }
    }
}

impl SoundSettings {
    /// Create new sound settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the volume level.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the playback speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.1);
        self
    }

    /// Set whether the sound should loop.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// Current state of a sound playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Sound is currently playing.
    Playing,
    /// Sound is paused.
    Paused,
    /// Sound has stopped (finished or manually stopped).
    Stopped,
}
