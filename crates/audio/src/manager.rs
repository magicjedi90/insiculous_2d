//! Audio manager for loading and playing sounds.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::error::{AudioError, AudioResult};
use crate::sound::{SoundHandle, SoundSettings};

/// Clamp a volume value to the valid 0.0..=1.0 range.
fn clamp_volume(volume: f32) -> f32 {
    volume.clamp(0.0, 1.0)
}

/// Floor a playback speed at 0.1 (rodio misbehaves at zero/negative speeds).
fn clamp_speed(speed: f32) -> f32 {
    speed.max(0.1)
}

/// Cached sound data that can be played multiple times.
struct SoundData {
    /// Raw audio bytes for replay. `Arc<[u8]>` implements `AsRef<[u8]>`, so a
    /// `Cursor<Arc<[u8]>>` can feed a decoder without copying the buffer.
    bytes: Arc<[u8]>,
}

/// Active sound playback instance.
struct ActiveSound {
    sink: Sink,
    /// Which loaded sound this instance plays (used by [`AudioManager::stop`]).
    handle: SoundHandle,
    /// The per-sound volume from `SoundSettings`, kept so bus volume changes
    /// (`set_sfx_volume` / `set_master_volume`) can re-derive the sink volume.
    base_volume: f32,
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
/// - Background music playback (looping or one-shot)
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
    /// Per-track volume of the current music, kept so bus volume changes can
    /// re-derive the music sink volume.
    music_base_volume: f32,
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
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.output.is_some()
    }

    fn with_output(output: Option<AudioOutput>) -> Self {
        Self {
            output,
            sounds: HashMap::new(),
            active_sounds: Vec::new(),
            music_sink: None,
            music_base_volume: 1.0,
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 1.0,
        }
    }

    /// Load a sound from a file path.
    ///
    /// The sound is cached and can be played multiple times.
    /// Supports WAV, MP3, OGG, and FLAC formats.
    ///
    /// File read failures return [`AudioError::IoError`]; undecodable data
    /// returns [`AudioError::DecodeError`].
    pub fn load_sound<P: AsRef<Path>>(&mut self, path: P) -> AudioResult<SoundHandle> {
        let path = path.as_ref();

        // Read the entire file into memory for replay support.
        // I/O failures convert via `From<io::Error>`.
        let bytes = std::fs::read(path)?;

        let handle = self.load_sound_from_bytes(bytes).map_err(|e| match e {
            // Re-attach the file path for decode diagnostics.
            AudioError::DecodeError(msg) => {
                AudioError::DecodeError(format!("{}: {}", path.display(), msg))
            }
            other => other,
        })?;

        log::debug!("Loaded sound: {} (handle: {})", path.display(), handle.id);

        Ok(handle)
    }

    /// Load a sound from raw bytes.
    ///
    /// Useful for embedded audio or procedurally generated sounds.
    pub fn load_sound_from_bytes(&mut self, bytes: Vec<u8>) -> AudioResult<SoundHandle> {
        let bytes: Arc<[u8]> = Arc::from(bytes);

        // Validate that the audio can be decoded. Cloning the Arc is cheap
        // (reference count bump, no buffer copy). rodio's Decoder requires a
        // `'static` reader, so a borrowed `Cursor<&[u8]>` cannot be used.
        Decoder::new(Cursor::new(Arc::clone(&bytes)))
            .map_err(|e| AudioError::DecodeError(e.to_string()))?;

        let handle = SoundHandle::new();
        self.sounds.insert(handle.id, SoundData { bytes });

        log::debug!("Loaded sound from bytes (handle: {})", handle.id);

        Ok(handle)
    }

    /// Play a sound with default settings.
    pub fn play(&mut self, handle: SoundHandle) -> AudioResult<()> {
        self.play_with_settings(&handle, SoundSettings::default())
    }

    /// Play a sound with custom settings.
    ///
    /// Volume is clamped to 0.0..=1.0 and speed floored at 0.1 here, so
    /// directly-set `SoundSettings` fields cannot bypass the valid ranges.
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

        // Decode straight from the shared cached bytes — no buffer copy.
        let cursor = Cursor::new(Arc::clone(&sound_data.bytes));
        let source = Decoder::new(cursor)
            .map_err(|e| AudioError::DecodeError(e.to_string()))?;

        // Clamp at point of use: SoundSettings fields are public, so builder
        // clamps can be bypassed.
        let base_volume = clamp_volume(settings.volume);
        sink.set_volume(base_volume * self.sfx_volume * self.master_volume);
        sink.set_speed(clamp_speed(settings.speed));

        if settings.looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.active_sounds.push(ActiveSound {
            sink,
            handle: *handle,
            base_volume,
        });

        Ok(())
    }

    /// Play background music from a file, looping forever.
    ///
    /// Only one music track can play at a time. Playing new music
    /// will stop the current track.
    pub fn play_music<P: AsRef<Path>>(&mut self, path: P) -> AudioResult<()> {
        self.play_music_with_volume(path, 1.0)
    }

    /// Play looping background music with a specific volume.
    pub fn play_music_with_volume<P: AsRef<Path>>(
        &mut self,
        path: P,
        volume: f32,
    ) -> AudioResult<()> {
        self.start_music(path.as_ref(), volume, true)
    }

    /// Play background music once (no looping), with a specific volume.
    ///
    /// The track plays to completion and then stops; use
    /// [`AudioManager::play_music`] for looping playback.
    pub fn play_music_once<P: AsRef<Path>>(
        &mut self,
        path: P,
        volume: f32,
    ) -> AudioResult<()> {
        self.start_music(path.as_ref(), volume, false)
    }

    /// Shared music startup: stops current music, opens and decodes the file,
    /// then starts playback (looping or one-shot).
    ///
    /// In disabled mode the file is still opened and decode-validated, but
    /// playback is a no-op: the call returns `Ok` while
    /// [`AudioManager::is_music_playing`] keeps reporting `false`. This keeps
    /// load errors observable on headless machines without pretending audio
    /// is audible.
    fn start_music(&mut self, path: &Path, volume: f32, looping: bool) -> AudioResult<()> {
        // Stop current music if any
        self.stop_music();

        // I/O failures convert via `From<io::Error>`.
        let file = File::open(path)?;

        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::DecodeError(format!("{}: {}", path.display(), e)))?;

        // Disabled mode: file was validated above, playback is a no-op.
        let Some(output) = &self.output else {
            return Ok(());
        };

        let sink = Sink::try_new(&output.handle)
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let base_volume = clamp_volume(volume);
        sink.set_volume(base_volume * self.music_volume * self.master_volume);
        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.music_sink = Some(sink);
        self.music_base_volume = base_volume;

        log::info!("Playing music: {} (looping: {})", path.display(), looping);

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
    ///
    /// Always `false` in disabled mode, even after a successful
    /// [`AudioManager::play_music`] call (playback is a no-op there).
    #[must_use]
    pub fn is_music_playing(&self) -> bool {
        self.music_sink.as_ref().is_some_and(|s| !s.is_paused() && !s.empty())
    }

    /// Stop all currently playing instances of the given sound.
    ///
    /// Instances of other sounds and the music track are unaffected.
    /// Unknown handles or handles with no active instances are a no-op.
    pub fn stop(&mut self, handle: SoundHandle) {
        self.active_sounds.retain(|active| {
            if active.handle == handle {
                active.sink.stop();
                false
            } else {
                true
            }
        });
    }

    /// Stop all currently playing sounds (music is unaffected).
    pub fn stop_all(&mut self) {
        for active in self.active_sounds.drain(..) {
            active.sink.stop();
        }
    }

    /// Set the master volume (affects all audio, including sounds and music
    /// that are already playing).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = clamp_volume(volume);
        self.update_all_volumes();
    }

    /// Get the current master volume.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set the sound effects volume (re-applied to currently playing sounds).
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = clamp_volume(volume);
        self.update_all_volumes();
    }

    /// Get the current sound effects volume.
    #[must_use]
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    /// Set the music volume (re-applied to currently playing music).
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = clamp_volume(volume);
        self.update_all_volumes();
    }

    /// Get the current music volume.
    #[must_use]
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    /// Re-derive sink volumes for the music track and every live SFX
    /// instance from `base * bus * master`.
    fn update_all_volumes(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(self.music_base_volume * self.music_volume * self.master_volume);
        }
        for active in &self.active_sounds {
            active
                .sink
                .set_volume(active.base_volume * self.sfx_volume * self.master_volume);
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
    #[must_use]
    pub fn active_sound_count(&self) -> usize {
        self.active_sounds.len()
    }

    /// Unload a sound from the cache.
    ///
    /// Already-playing instances of the sound continue to completion (each
    /// playback holds its own reference to the audio data); only future
    /// `play` calls with this handle will fail.
    pub fn unload(&mut self, handle: SoundHandle) {
        self.sounds.remove(&handle.id);
    }

    /// Unload all cached sounds.
    ///
    /// As with [`AudioManager::unload`], already-playing instances continue.
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
    fn test_sound_settings_speed_floored_at_point_one() {
        let settings = SoundSettings::new().with_speed(0.01);
        assert!((settings.speed - 0.1).abs() < f32::EPSILON);

        let settings = SoundSettings::new().with_speed(-3.0);
        assert!((settings.speed - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clamp_helpers_enforce_valid_ranges() {
        assert!((clamp_volume(2.0) - 1.0).abs() < f32::EPSILON);
        assert!(clamp_volume(-0.5).abs() < f32::EPSILON);
        assert!((clamp_volume(0.7) - 0.7).abs() < f32::EPSILON);

        assert!((clamp_speed(0.0) - 0.1).abs() < f32::EPSILON);
        assert!((clamp_speed(-1.0) - 0.1).abs() < f32::EPSILON);
        assert!((clamp_speed(2.5) - 2.5).abs() < f32::EPSILON);
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

    /// Write `tiny_wav` to a unique temp file and return its path.
    fn write_temp_wav(tag: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "insiculous_audio_test_{}_{}.wav",
            tag,
            std::process::id()
        ));
        std::fs::write(&path, tiny_wav()).expect("temp dir must be writable");
        path
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
        assert!(manager.play(handle).is_ok());
        assert_eq!(manager.active_sound_count(), 0, "no-op playback must not track sinks");
    }

    #[test]
    fn test_disabled_manager_still_rejects_invalid_handles() {
        let mut manager = AudioManager::disabled();
        let bogus = SoundHandle::new();
        assert!(manager.play(bogus).is_err());
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

    #[test]
    fn test_load_sound_from_file_succeeds() {
        let path = write_temp_wav("load_ok");
        let mut manager = AudioManager::disabled();
        let result = manager.load_sound(&path);
        std::fs::remove_file(&path).ok();

        let handle = result.expect("valid wav file must load");
        assert!(manager.play(handle).is_ok());
    }

    #[test]
    fn test_load_sound_missing_file_returns_io_error() {
        let mut manager = AudioManager::disabled();
        let missing = std::env::temp_dir().join("insiculous_audio_test_definitely_missing.wav");
        let err = manager.load_sound(&missing).expect_err("missing file must fail");
        assert!(
            matches!(err, AudioError::IoError(_)),
            "expected IoError, got: {err:?}"
        );
    }

    #[test]
    fn test_load_sound_from_invalid_bytes_returns_decode_error() {
        let mut manager = AudioManager::disabled();
        let err = manager
            .load_sound_from_bytes(vec![0xDE, 0xAD, 0xBE, 0xEF])
            .expect_err("garbage bytes must fail to decode");
        assert!(
            matches!(err, AudioError::DecodeError(_)),
            "expected DecodeError, got: {err:?}"
        );
    }

    #[test]
    fn test_unloaded_sound_can_no_longer_be_played() {
        let mut manager = AudioManager::disabled();
        let handle = manager.load_sound_from_bytes(tiny_wav()).unwrap();
        assert!(manager.play(handle).is_ok());

        manager.unload(handle);
        let err = manager.play(handle).expect_err("unloaded handle must be rejected");
        assert!(matches!(err, AudioError::InvalidHandle(_)));
    }

    #[test]
    fn test_unload_all_invalidates_every_handle() {
        let mut manager = AudioManager::disabled();
        let first = manager.load_sound_from_bytes(tiny_wav()).unwrap();
        let second = manager.load_sound_from_bytes(tiny_wav()).unwrap();

        manager.unload_all();

        assert!(manager.play(first).is_err());
        assert!(manager.play(second).is_err());
    }

    #[test]
    fn test_stop_on_unknown_handle_is_noop() {
        let mut manager = AudioManager::disabled();
        let bogus = SoundHandle::new();
        manager.stop(bogus);
        assert_eq!(manager.active_sound_count(), 0);
    }

    #[test]
    fn test_stop_and_stop_all_are_safe_when_nothing_plays() {
        let mut manager = AudioManager::disabled();
        let handle = manager.load_sound_from_bytes(tiny_wav()).unwrap();
        manager.play(handle).unwrap();

        manager.stop(handle);
        manager.stop_all();
        assert_eq!(manager.active_sound_count(), 0);
    }

    #[test]
    fn test_volume_setters_clamp_out_of_range_values() {
        let mut manager = AudioManager::disabled();

        manager.set_master_volume(2.0);
        assert!((manager.master_volume() - 1.0).abs() < f32::EPSILON);
        manager.set_master_volume(-1.0);
        assert!(manager.master_volume().abs() < f32::EPSILON);

        manager.set_sfx_volume(5.0);
        assert!((manager.sfx_volume() - 1.0).abs() < f32::EPSILON);
        manager.set_sfx_volume(-0.2);
        assert!(manager.sfx_volume().abs() < f32::EPSILON);

        manager.set_music_volume(1.5);
        assert!((manager.music_volume() - 1.0).abs() < f32::EPSILON);
        manager.set_music_volume(-0.5);
        assert!(manager.music_volume().abs() < f32::EPSILON);
    }

    #[test]
    fn test_disabled_manager_music_loads_but_reports_not_playing() {
        let path = write_temp_wav("music_once");
        let mut manager = AudioManager::disabled();

        let looping = manager.play_music(&path);
        let once = manager.play_music_once(&path, 0.5);
        std::fs::remove_file(&path).ok();

        assert!(looping.is_ok());
        assert!(once.is_ok());
        // Documented behavior: disabled mode validates the file but never
        // reports music as playing.
        assert!(!manager.is_music_playing());
    }

    #[test]
    fn test_play_music_missing_file_returns_io_error() {
        let mut manager = AudioManager::disabled();
        let missing = std::env::temp_dir().join("insiculous_audio_test_no_such_music.ogg");
        let err = manager
            .play_music_once(&missing, 1.0)
            .expect_err("missing music file must fail");
        assert!(matches!(err, AudioError::IoError(_)));
    }
}
