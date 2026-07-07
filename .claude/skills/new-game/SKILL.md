---
name: new-game
description: Scaffold a new game for the 20 Games Challenge following the established Pong/Breakout conventions (module layout, chaos modes, achievements, neon look, headless tests). Use when starting any new game in ../games/.
---

# New Game — 20 Games Challenge Scaffold

Games are **standalone cargo projects in `../games/<name>/`** (sibling of the
engine repo) — NOT workspace members, NOT git repos. Before writing anything,
open `../games/breakout/` (most recent) and `../games/pong/` and mirror their
structure. When conventions here conflict with what breakout actually does,
breakout wins — read the code.

## Cargo setup

- Depend on `engine_core` by path (`../../insiculous_2d/crates/engine_core`).
- Optional `editor_integration` dep behind an `editor` feature (copy breakout's
  `Cargo.toml`).

## Required module layout

`main.rs` (Game impl + `game_root()` asset anchoring: exe-dir →
`CARGO_MANIFEST_DIR` fallback), `constants.rs`, `types.rs`, `spawning.rs`,
`gameplay.rs`, `menu.rs`, `drawing.rs`, `chaos_theme.rs`, `achievements.rs`,
`effects.rs`.

## Non-negotiable conventions

1. **Chaos modes**: all four variants (Normal / Insane / Ridiculous /
   Insiculous) with a per-game meaning for each, `ChaosTheme` colors, HUD
   banner. Mirror the menu selection into `ctx.chaos_mode` (it's read-write and
   persisted by the engine). Insiculous = insane + ridiculous simultaneously —
   `is_insane()`/`is_ridiculous()` both return true for it, so branch on the
   two predicates independently.
2. **Achievements**: engine `AchievementManager`, persisted to
   `saves/<game>_achievements.json`, plus a `DISPLAY_SECTIONS` coverage test
   (every registered achievement appears in a display section).
3. **Neon look**: GridMesh deforming background, emissive sprites, particle
   bursts via `ParticleConfig`.
4. **Sizing**: `RENDER_UNIT = 80` — `Transform2D.scale × 80` = pixel size.
   **Colliders use absolute pixels; physics IGNORES Transform2D.scale.** Size
   them from the same constants or sprites and colliders drift apart.
5. **Determinism**: pseudo-random via frame_count hash (multiplier 2654435761).
   No `rand` crate.
6. **Headless tests** for all pure logic: layout math, bounce/steering
   direction, menu navigation, achievement registration. Every test must run
   without GPU/window.

## Proven gameplay patterns (from breakout — reuse, don't reinvent)

- Offset-based paddle bounce (`paddle_bounce_direction`).
- Minimum-vertical-velocity enforcement (anti-stalemate).
- Serve-glue via per-frame `reset_body` (buffered, safe on same-frame spawns).
- Ball/entity loss = sensor hit OR out-of-bounds safety net (both).
- Collision events: snapshot with `.to_vec()` before consuming.
- Movement: `PhysicsSystem::set_velocity(entity, linear, angular)` for every
  launch/move case — never reach into rapier unless you need mass-aware impulses.

## If the engine is missing something

Log it (and fix it in the engine, with tests) rather than working around it in
the game — e.g. MouseButton was added to the engine_core prelude this way.
Engine changes still require the full engine checklist:
`cargo test --workspace` + clippy clean in the engine repo.

## Verify

The game builds and its own `cargo test` passes headless. If you touched the
engine, run the engine's full verification too (`/finish-task`).
