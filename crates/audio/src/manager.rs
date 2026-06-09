//! Audio manager for loading and playing sounds.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::error::{AudioError, AudioResult};
use crate::sound::{SoundHandle, SoundSettings};

/// Cached sound data that can be played multiple times.
struct SoundData {
    /// Raw audio bytes for replay.
    bytes: Arc<Vec<u8>>,
}

/// Active sound playback instance.
struct ActiveSound {
    sink: Sink,
    #[allow(dead_code)] // Reserved for future "stop by handle" API
    handle: SoundHandle,
}

/// Live connection to an audio output device.
struct AudioOutput {
    /// Audio output stream (must be kept alive).
    _stream: OutputStream,
    /// Handle to the output stream for creating sinks.
    handle: OutputStreamHandle,
}

/// Manages audio playback for the game engine.
///
/// The AudioManager handles:
/// - Loading and caching sound files
/// - Playing sounds with configurable settings
/// - Managing active sound instances
/// - Background music with crossfade
///
/// A manager can run in *disabled* mode (no audio device available): sounds
/// still load and validate, playback calls succeed as no-ops. This keeps
/// games runnable on headless machines and in CI.
pub struct AudioManager {
    /// Output device connection. `None` means disabled mode — playback no-ops.
    output: Option<AudioOutput>,
    /// Cached sound data by handle.
    sounds: HashMap<u32, SoundData>,
    /// Currently active sound instances.
    active_sounds: Vec<ActiveSound>,
    /// Current background music sink.
    music_sink: Option<Sink>,
    /// Master volume for all sounds.
    master_volume: f32,
    /// Volume for sound effects.
    sfx_volume: f32,
    /// Volume for background music.
    music_volume: f32,
}

impl AudioManager {
    /// Create a new audio manager.
    ///
    /// This initializes the audio device and output stream.
    pub fn new() -> AudioResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::DeviceInitError(e.to_string()))?;

        log::debug!("Audio system initialized");

        Ok(Self::with_output(Some(AudioOutput {
            _stream: stream,
            handle: stream_handle,
        })))
    }

    /// Create a disabled audio manager that has no output device.
    ///
    /// Sounds can still be loaded (and are decode-validated); all playback
    /// calls succeed silently. Use this when audio hardware is unavailable.
    pub fn disabled() -> Self {
        Self::with_output(None)
    }

    /// Create an audio manager, falling back to disabled mode if no audio
    /// device is available. Never fails — the game keeps running either way.
    pub fn new_or_disabled() -> Self {
        match Self::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::warn!("Failed to initialize audio: {}. Audio will be disabled.", e);
                Self::disabled()
            }
        }
    }

    /// Whether an output device is connected. `false` means playback no-ops.
    pub fn is_enabled(&self) -> bool {
        self.output.is_some()
    }

    fn with_output(output: Option<AudioOutput>) -> Self {
        Self {
            output,
            sounds: HashMap::new(),
            active_sounds: Vec::new(),
            music_sink: None,
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 1.0,
        }
    }

    /// Load a sound from a file path.
    ///
    /// The sound is cached and can be played multiple times.
    /// Supports WAV, MP3, OGG, and FLAC formats.
    pub fn load_sound<P: AsRef<Path>>(&mut self, path: P) -> AudioResult<SoundHandle> {
        let path = path.as_ref();

        // Read the entire file into memory for replay support
        let bytes = std::fs::read(path)
            .map_err(|e| AudioError::LoadError(format!("{}: {}", path.display(), e)))?;

        // Validate that the audio can be decoded by trying to decode a clone
        let bytes_clone = bytes.clone();
        let cursor = std::io::Cursor::new(bytes_clone);
        let _ = Decoder::new(BufReader::new(cursor))
            .map_err(|e| AudioError::DecodeError(format!("{}: {}", path.display(), e)))?;

        let handle = SoundHandle::new();
        self.sounds.insert(handle.id, SoundData {
            bytes: Arc::new(bytes),
        });

        log::debug!("Loaded sound: {} (handle: {})", path.display(), handle.id);

        Ok(handle)
    }

    /// Load a sound from raw bytes.
    ///
    /// Useful for embedded audio or procedurally generated sounds.
    pub fn load_sound_from_bytes(&mut self, bytes: Vec<u8>) -> AudioResult<SoundHandle> {
        // Validate that the audio can be decoded by trying to decode a clone
        let bytes_clone = bytes.clone();
        let cursor = std::io::Cursor::new(bytes_clone);
        let _ = Decoder::new(BufReader::new(cursor))
            .map_err(|e| AudioError::DecodeError(e.to_string()))?;

        let handle = SoundHandle::new();
        self.sounds.insert(handle.id, SoundData {
            bytes: Arc::new(bytes),
        });

        log::debug!("Loaded sound from bytes (handle: {})", handle.id);

        Ok(handle)
    }

    /// Play a sound with default settings.
    pub fn play(&mut self, handle: &SoundHandle) -> AudioResult<()> {
        self.play_with_settings(handle, SoundSettings::default())
    }

    /// Play a sound with custom settings.
    pub fn play_with_settings(
        &mut self,
        handle: &SoundHandle,
        settings: SoundSettings,
    ) -> AudioResult<()> {
        let sound_data = self.sounds.get(&handle.id)
            .ok_or(AudioError::InvalidHandle(handle.id))?;

        // Disabled mode: handle was validated above, playback is a no-op.
        let Some(output) = &self.output else {
            return Ok(());
        };

        let sink = Sink::try_new(&output.handle)
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        // Create a decoder from the cached bytes
        let cursor = std::io::Cursor::new(sound_data.bytes.as_ref().clone());
        let source = Decoder::new(BufReader::new(cursor))
            .map_err(|e| AudioError::DecodeError(e.to_string()))?;

        // Apply settings
        let volume = settings.volume * self.sfx_volume * self.master_volume;
        sink.set_volume(volume);
        sink.set_speed(settings.speed);

        if settings.looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.active_sounds.push(ActiveSound {
            sink,
            handle: *handle,
        });

        Ok(())
    }

    /// Play background music from a file.
    ///
    /// Only one music track can play at a time. Playing new music
    /// will stop the current track.
    pub fn play_music<P: AsRef<Path>>(&mut self, path: P) -> AudioResult<()> {
        self.play_music_with_volume(path, 1.0)
    }

    /// Play background music with a specific volume.
    pub fn play_music_with_volume<P: AsRef<Path>>(
        &mut self,
        path: P,
        volume: f32,
    ) -> AudioResult<()> {
        // Stop current music if any
        self.stop_music();

        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| AudioError::LoadError(format!("{}: {}", path.display(), e)))?;

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::DecodeError(format!("{}: {}", path.display(), e)))?;

        // Disabled mode: file was validated above, playback is a no-op.
        let Some(output) = &self.output else {
            return Ok(());
        };

        let sink = Sink::try_new(&output.handle)
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let effective_volume = volume * self.music_volume * self.master_volume;
        sink.set_volume(effective_volume);
        sink.append(source.repeat_infinite());

        self.music_sink = Some(sink);

        log::info!("Playing music: {}", path.display());

        Ok(())
    }

    /// Stop the current background music.
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
    }

    /// Pause the current background music.
    pub fn pause_music(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.pause();
        }
    }

    /// Resume the paused background music.
    pub fn resume_music(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.play();
        }
    }

    /// Check if music is currently playing.
    pub fn is_music_playing(&self) -> bool {
        self.music_sink.as_ref().is_some_and(|s| !s.is_paused() && !s.empty())
    }

    /// Stop all currently playing sounds.
    pub fn stop_all(&mut self) {
        for active in self.active_sounds.drain(..) {
            active.sink.stop();
        }
    }

    /// Set the master volume (affects all audio).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.update_all_volumes();
    }

    /// Get the current master volume.
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set the sound effects volume.
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
        // Note: This only affects future sounds, not currently playing ones
    }

    /// Get the current sound effects volume.
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    /// Set the music volume.
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(self.music_volume * self.master_volume);
        }
    }

    /// Get the current music volume.
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Update volumes for music when master volume changes.
    fn update_all_volumes(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(self.music_volume * self.master_volume);
        }
    }

    /// Clean up finished sound instances.
    ///
    /// Call this periodically (e.g., once per frame) to free resources
    /// from sounds that have finished playing.
    pub fn update(&mut self) {
        self.active_sounds.retain(|active| !active.sink.empty());
    }

    /// Get the number of currently active sounds.
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }

    /// Unload a sound from the cache.
    pub fn unload(&mut self, handle: &SoundHandle) {
        self.sounds.remove(&handle.id);
    }

    /// Unload all cached sounds.
    pub fn unload_all(&mut self) {
        self.sounds.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_settings_builder() {
        let settings = SoundSettings::new()
            .with_volume(0.5)
            .with_speed(1.5)
            .with_looping(true);

        assert!((settings.volume - 0.5).abs() < f32::EPSILON);
        assert!((settings.speed - 1.5).abs() < f32::EPSILON);
        assert!(settings.looping);
    }

    #[test]
    fn test_sound_settings_volume_clamping() {
        let settings = SoundSettings::new().with_volume(2.0);
        assert!((settings.volume - 1.0).abs() < f32::EPSILON);

        let settings = SoundSettings::new().with_volume(-1.0);
        assert!(settings.volume.abs() < f32::EPSILON);
    }

    #[test]
    fn test_sound_handle_unique() {
        let handle1 = SoundHandle::new();
        let handle2 = SoundHandle::new();
        assert_ne!(handle1.id, handle2.id);
    }

    /// Minimal valid WAV file (44-byte header + one silent 16-bit sample).
    fn tiny_wav() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&38u32.to_le_bytes()); // chunk size
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
        bytes.extend_from_slice(&44100u32.to_le_bytes()); // sample rate
        bytes.extend_from_slice(&88200u32.to_le_bytes()); // byte rate
        bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
        bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&2u32.to_le_bytes()); // data size
        bytes.extend_from_slice(&0i16.to_le_bytes()); // one silent sample
        bytes
    }

    #[test]
    fn test_disabled_manager_reports_not_enabled() {
        let manager = AudioManager::disabled();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_disabled_manager_loads_and_plays_as_noop() {
        let mut manager = AudioManager::disabled();
        let handle = manager.load_sound_from_bytes(tiny_wav()).unwrap();
        assert!(manager.play(&handle).is_ok());
        assert_eq!(manager.active_sound_count(), 0, "no-op playback must not track sinks");
    }

    #[test]
    fn test_disabled_manager_still_rejects_invalid_handles() {
        let mut manager = AudioManager::disabled();
        let bogus = SoundHandle::new();
        assert!(manager.play(&bogus).is_err());
    }

    #[test]
    fn test_disabled_manager_music_controls_are_safe() {
        let mut manager = AudioManager::disabled();
        manager.stop_music();
        manager.pause_music();
        manager.resume_music();
        assert!(!manager.is_music_playing());
        manager.update();
    }

    #[test]
    fn test_new_or_disabled_never_fails() {
        // With or without an audio device, construction must succeed.
        let _manager = AudioManager::new_or_disabled();
    }
}
