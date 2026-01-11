//! Audio system for the insiculous_2d game engine.
//!
//! This crate provides audio playback functionality including:
//! - Sound effect playback with volume and speed control
//! - Background music with crossfade support
//! - Audio resource management and caching
//!
//! # Example
//! ```ignore
//! use audio::{AudioManager, SoundHandle};
//!
//! let mut audio = AudioManager::new()?;
//! let sound = audio.load_sound("assets/jump.wav")?;
//! audio.play(&sound);
//! ```

mod error;
mod manager;
mod sound;

pub use error::AudioError;
pub use manager::AudioManager;
pub use sound::{SoundHandle, SoundSettings, PlaybackState};
