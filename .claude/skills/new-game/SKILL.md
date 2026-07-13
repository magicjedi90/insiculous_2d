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

`main.rs` (Game impl; asset anchoring via `engine_core::game_root!()` — a
macro so YOUR crate's manifest dir is baked in), `constants.rs`, `types.rs`,
`spawning.rs`, `gameplay.rs`, `menu.rs`, `drawing.rs`, `achievements.rs`,
`effects.rs`. A `chaos_theme.rs` is only needed if you override the engine
palette (see below) — pong uses the engine defaults directly.

## Non-negotiable conventions

1. **Chaos modes**: all four variants (Normal / Insane / Ridiculous /
   Insiculous) with a per-game meaning for each. The LOOK comes from the
   engine: `ChaosTheme::for_mode(mode)` (prelude) gives the shared palette
   (`bg_color`, `structure_color`, `accent_color`, `grid_color`, banner,
   `particle_count_mult`); override individual fields with struct-update
   syntax if your game's art differs (see breakout's thin `theme_for()`).
   Mirror the menu selection into `ctx.chaos_mode` (it's read-write and
   persisted by the engine). Insiculous = insane + ridiculous simultaneously —
   `is_insane()`/`is_ridiculous()` both return true for it, so branch on the
   two predicates independently.
2. **Achievements**: engine `AchievementManager`, persisted to
   `saves/<game>_achievements.json`, plus a `DISPLAY_SECTIONS` coverage test
   (every registered achievement appears in a display section).
3. **Neon look**: GridMesh deforming background driven by
   `engine_core::grid::step_and_emit_grid(self.grid.as_mut(), ctx.world,
   ctx.lines, ctx.delta_time, self.debug_colliders)` (also gives you the F1
   collider overlay), emissive sprites, particle bursts via `ParticleConfig`.
4. **Sizing**: `RENDER_UNIT = 80` — `Transform2D.scale × 80` = pixel size.
   **Colliders use absolute pixels; physics IGNORES Transform2D.scale.** Size
   them from the same constants or sprites and colliders drift apart.
5. **Determinism**: pseudo-random via `hash_f32(frame_count)` /
   `hash_u32(...)` from the prelude. No `rand` crate, no hand-rolled hashing.
6. **Headless tests** for all pure logic: layout math, bounce/steering
   direction, menu navigation, achievement registration. Every test must run
   without GPU/window.

## Proven gameplay patterns (from breakout — reuse, don't reinvent)

- Offset-based paddle bounce (`paddle_bounce_direction`).
- Minimum-vertical-velocity enforcement (anti-stalemate).
- Serve-glue via per-frame `reset_body` (buffered, safe on same-frame spawns).
- Ball/entity loss = sensor hit OR out-of-bounds safety net (both).
- Collision events: drain ONCE per frame with
  `self.physics.take_collision_events()` and share the Vec with every
  consumer (gameplay + `Pickups::collect`). A second take returns empty.
- Movement: `PhysicsSystem::set_velocity(entity, linear, angular)` for every
  launch/move case — never reach into rapier unless you need mass-aware impulses.
- Bullets/effects/debris: attach `Lifetime::new(seconds)` and run a
  `LifetimeSystem` (prelude) — no per-entity timer bookkeeping.
- Pickups/drops: `engine_core::pickups::Pickups<K>` + `EffectTimer`.
- Menu-state sprite hiding: `set_sprites_visible(ctx.world, entities, visible)`
  (prelude) — you supply the entity list and the state match.
- Scene-defined templates: `instance.spawn_prefab(world, assets, "Name", &overrides)`
  stamps runtime copies of RON prefabs (breakout levels are scene-driven).

## Rule-of-three check (game 3+)

`../games/TECH_DEBT.md` GPP-03 part 2 lists genre-flavored duplication
deferred from pong/breakout (spawner shapes, particle preset semantics, the
Serving/Playing/GameOver flow skeleton). If this game duplicates one of them
a THIRD time, promote it to the engine as part of this task (engine tests +
refactor all games onto it); if not, leave it game-side.

## If the engine is missing something

Log it (and fix it in the engine, with tests) rather than working around it in
the game — e.g. MouseButton was added to the engine_core prelude this way.
Engine changes still require the full engine checklist:
`cargo test --workspace` + clippy clean in the engine repo.

## Verify

The game builds and its own `cargo test` passes headless. If you touched the
engine, run the engine's full verification too (`/finish-task`).
