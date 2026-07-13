# Task Queue - Insiculous 2D


## TASK-SI-001: Space Invaders (Game 3 of the 20 Games Challenge) — NEXT UP

**Start with `/new-game space_invaders`** (the skill was refreshed Jul 13 2026 for the
promoted engine APIs). Roadmap spec (PROJECT_ROADMAP Phase A, Game 3): grid of enemies
marching left/right and descending, player fires upward; win = clear all invaders,
lose = any invader reaches the bottom. ~400 lines.

Build notes from the engine work that preceded this:
- **Bullets**: spawn + `Lifetime::new(secs)` + `LifetimeSystem` (prelude) — no manual timers.
- **Invaders/bullets as prefabs**: consider a scene with `Invader`/`Bullet` prefabs and
  `instance.spawn_prefab(...)` for runtime stamping (breakout's levels are the precedent).
- **Formation movement**: kinematic bodies + `set_kinematic_target`, or transform-driven
  (live `Transform2D` edits now reach rapier). One shared direction state, march + descend.
- **Collision consumption**: ONE `take_collision_events()` per frame, share the Vec.
- **Theme**: `ChaosTheme::for_mode` defaults; map `accent_color` = player/bullets,
  `structure_color` = barriers. Chaos meanings are the game's to define (e.g. Insane =
  faster march/more bullets, Ridiculous = two-way fire?, Insiculous = both).
- **Rule-of-three check** (in the skill): if this game needs `spawn_wall`-shaped helpers,
  the particle-preset semantics, or the Serving/Playing/GameOver flow skeleton a third
  time, PROMOTE them (see `../games/TECH_DEBT.md` GPP-03 part 2).
- Verify: game `cargo test` headless green + engine `/finish-task` if the engine changed.

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

---

## Task sourcing

Open technical debt is NOT duplicated here — it lives in the live docs
(root `TECH_DEBT.md` rollup → per-crate `TECH_DEBT.md` + `../games/TECH_DEBT.md`).
Engine feature gaps live in `PROJECT_ROADMAP.md` (next up after game 3:
Gap 1 `CameraFollow`, Gap 3 `Tilemap`). Pull from those when this queue is empty.

(The Phase 1 task list that used to live here shipped in full — entity CRUD,
picking, component add/remove, undo/redo — see `log_archive.md`. Retired
Jul 13 2026.)
