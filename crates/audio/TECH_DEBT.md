# Technical Debt: audio

Last audited: June 11, 2026 (remediation pass)

## Summary
- DRY violations: 0 (2 fixed 2026-06-11)
- SRP violations: 0
- KISS violations: 0
- Architecture issues: 0

**Overall Assessment:** Clean after the June 2026 remediation. All playback
logic is headless-testable via disabled mode.

---

## Remediated (2026-06-11)

1. **Per-play full-buffer clone** — `play_with_settings` cloned the entire
   decoded file (`Arc<Vec<u8>>` deref-clone) on every play. Fixed: bytes are
   `Arc<[u8]>`, decoder reads `Cursor<Arc<[u8]>>` directly; pointless
   `BufReader` wrappers around in-memory cursors removed.
2. **Master/SFX volume ignored live sounds** — `update_all_volumes` only
   touched the music sink. Fixed: `ActiveSound` stores `base_volume`, and the
   music track stores `music_base_volume`; all volume setters re-apply
   `base * bus * master` to every live sink.
3. **Clamping bypassable** — `SoundSettings` fields are public so builder
   clamps could be bypassed. Fixed: volume/speed clamped at point of use in
   `play_with_settings` via `clamp_volume` / `clamp_speed` helpers (tested).
4. **Dead `PlaybackState` enum** — exported but never produced/accepted.
   Deleted (coordinator removes the `engine_core` prelude re-export).
5. **`AudioError::IoError` never constructed** — file-read failures were
   stringified into `LoadError`. Fixed: I/O failures convert via
   `#[from] io::Error`; `LoadError` reserved for non-IO load problems.
6. **`#[allow(dead_code)]` on `ActiveSound.handle`** — banned by project
   rules. Fixed by implementing the missing feature: `stop(handle)` stops all
   active instances of a sound.
7. **`AudioResult<T>` not exported** — appeared in every public signature but
   consumers couldn't name it. Fixed: re-exported from `lib.rs`.
8. **Music always looped** — added `play_music_once(path, volume)` for
   one-shot tracks; `play_music` / `play_music_with_volume` keep looping
   semantics.
9. **`play`/`unload` took `&SoundHandle`** — now take the 4-byte Copy handle
   by value (`play_with_settings` keeps `&SoundHandle` for existing external
   callers).
10. **DRY: `load_sound` duplicated decode validation** — now reads the file
    and delegates to `load_sound_from_bytes` (validation decodes from a cheap
    `Arc` clone, no buffer copy — rodio's `Decoder` requires a `'static`
    reader so `Cursor<&[u8]>` is not possible).
11. **DRY: `set_music_volume` re-inlined `update_all_volumes`** — now calls
    the shared helper.
12. **False "crossfade support" claims** in `lib.rs`/`manager.rs` docs —
    removed (crossfade was never implemented; `play_music` hard-stops).
13. **Missing `#[must_use]`** on `SoundSettings` builders and pure
    `AudioManager` getters — added.

---

## Known Limitations (By Design)

1. **No streaming for large files** — all sounds loaded into memory for
   instant playback
2. **No spatial audio in this crate** — `AudioSource`/`AudioListener`/
   `PlaySoundEffect` live in `crates/ecs/src/audio_components.rs` and are not
   consumed by any runtime system yet (editor-inspectable data only)
3. **No audio effects** — no reverb, echo, or other DSP
4. **Single music track** — only one music track at a time
5. **Disabled mode reports `is_music_playing() == false`** even after a
   successful `play_music` call — documented choice (no logical "pretend
   playing" flag)
6. **`IoError` carries no file path** — `#[from] io::Error` conversion loses
   path context (decode errors do include the path)

---

## Future Enhancements (Not Technical Debt)

1. Streaming for large music files
2. Runtime audio system bridging the ECS audio components (spatial playback)
3. Crossfade for music transitions
4. Audio effects processing
5. Generic bus/group API (currently fixed master/sfx/music buses)
6. Audio occlusion for environments

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 4 (`lib.rs`, `manager.rs`, `sound.rs`, `error.rs`) |
| Total lines | ~700 (incl. tests) |
| Test count | 20 (all headless, no audio device needed) |
| High priority issues | 0 |
| Medium priority issues | 0 |
| Low priority issues | 0 |
