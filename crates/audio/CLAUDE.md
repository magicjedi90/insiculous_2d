# Audio Crate — Agent Context

Audio playback via rodio with spatial audio support.

## Key Types
- `AudioManager` — sound loading, playback, volume control
- `AudioSource` — ECS component: volume, pitch, spatial settings, looping
- `AudioListener` — ECS component: listener position for spatial audio
- `PlaySoundEffect` — one-shot sound trigger component

## Known Tech Debt
- All audio loaded eagerly into memory (no streaming for large music files)

## Testing
- 3 tests, run with `cargo test -p audio`

## Godot Oracle
- Audio architecture: `servers/audio_server.cpp`, `scene/audio/audio_stream_player.cpp`
