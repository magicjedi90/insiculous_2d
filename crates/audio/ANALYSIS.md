# Audio Crate Analysis

## Review (January 19, 2026)

### Summary
- Provides `AudioManager`-centric playback over `rodio` with cached sound data and simple playback controls.
- Exposes ECS-facing audio components for positional attenuation and one-shot effects.
- Dependencies are minimal (`rodio`, `log`, `thiserror`) and keep the surface area focused.

### Strengths
- Straightforward API for SFX and music with `SoundHandle` + `SoundSettings` builder patterns.
- ECS integration keeps audio concerns consistent with other engine systems.
- Clear separation between asset loading/caching and playback management.

### Risks & Follow-ups
- All audio is eagerly loaded into memory; consider optional streaming for large music tracks.
- Spatial audio is limited to attenuation math (no real 3D processing); document the limitation in API docs.
- Track whether multiple music tracks or mixing buses are needed as the project grows.

## Overview

The `audio` crate provides audio playback functionality for the insiculous_2d game engine. It wraps the `rodio` audio library to provide a simple, game-oriented API for sound effects and background music.

## Architecture

### Core Components

#### AudioManager (`manager.rs`)
The main audio system that handles:
- Loading and caching sound files
- Playing sounds with configurable settings
- Managing active sound instances
- Background music with pause/resume
- Volume control (master, SFX, music)

**Design Decisions:**
- Uses `rodio` for cross-platform audio playback
- Sounds are cached in memory as raw bytes for efficient replay
- Active sounds are tracked for cleanup via `update()`
- Non-blocking audio playback via rodio's sink system

#### SoundHandle (`sound.rs`)
Unique identifier for loaded sounds:
- Auto-incrementing IDs using atomic counter
- Used to reference cached sounds for playback

#### SoundSettings (`sound.rs`)
Configuration for individual sound playback:
- Volume (0.0 to 1.0)
- Speed/pitch (1.0 = normal)
- Looping flag
- Builder pattern for fluent configuration

### ECS Integration (`ecs/audio_components.rs`)

#### AudioSource
Component for entities that emit sounds:
- Sound ID reference
- Volume, pitch, looping settings
- Spatial audio support with:
  - Max distance
  - Reference distance
  - Rolloff factor
- Distance attenuation calculation

#### AudioListener
Component for the entity that "hears" sounds:
- Active flag (only one listener per scene)
- Listener-specific volume multiplier

#### PlaySoundEffect
One-shot sound effect request component:
- Designed for event-based sounds (jump, explosion)
- Auto-removed after processing by audio system

## Dependencies

- `rodio` 0.20 - Audio playback (via cpal backend)
- `symphonia` - Audio format decoding (WAV, MP3, OGG, FLAC)
- `log` - Logging
- `thiserror` - Error handling

## Supported Formats

Via symphonia integration:
- WAV
- MP3
- OGG/Vorbis
- FLAC
- AAC

## Usage Example

```rust
// Load a sound
let jump_sound = ctx.audio.load_sound("assets/jump.wav")?;

// Play with default settings
ctx.audio.play(&jump_sound)?;

// Play with custom settings
let settings = SoundSettings::new()
    .with_volume(0.8)
    .with_speed(1.2);
ctx.audio.play_with_settings(&jump_sound, settings)?;

// Background music
ctx.audio.play_music("assets/music.ogg")?;
ctx.audio.pause_music();
ctx.audio.resume_music();

// Volume control
ctx.audio.set_master_volume(0.5);
ctx.audio.set_sfx_volume(0.8);
ctx.audio.set_music_volume(0.6);
```

## Test Coverage

- 3 unit tests in `manager.rs` (SoundSettings, SoundHandle)
- 7 unit tests in `audio_components.rs` (AudioSource, AudioListener, spatial attenuation)

## Known Limitations

These are intentional design decisions, not technical debt:

1. **No streaming for large files** - All sounds loaded into memory for instant playback
2. **No 3D/spatial audio processing** - Only attenuation calculation provided (requires integration with 3rd party library)
3. **No audio effects** - No reverb, echo, or other DSP effects (use external audio tools)
4. **Single music track** - Only one music track at a time (sufficient for most 2D games)

## Future Enhancements

1. Add streaming for large music files
2. Implement actual spatial audio positioning  
3. Add crossfade for music transitions
4. Add audio effects processing
5. Support for audio buses/groups
6. Audio occlusion for environments
