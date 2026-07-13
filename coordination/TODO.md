# Task Queue - Insiculous 2D


## TASK-ASTEROIDS-001: Asteroids (Game 5 of the 20 Games Challenge) — NEXT UP

**Start with `/new-game asteroids`.** Roadmap spec (PROJECT_ROADMAP Phase A, Game 5):
ship rotates and thrusts in 2D space, asteroids split on hit, screen wraps.
~400 lines.

Build notes:
- **No engine gaps**: rotation-based movement (dynamic ship with angular
  velocity via `set_velocity`), screen-wrap teleports (`Transform2D` writes —
  GPP-09 means live physics bodies follow), splitting = destroy + spawn
  smaller pair, invincibility frames.
- Bullets: dynamic sensors + `Lifetime` (space_invaders recipe). Prove every
  physics pair the game depends on with a headless sim test first — rapier
  does not report kinematic-vs-kinematic/static pairs.
- **Shared scaffolding** (prelude): `MenuInput`, `spawn_background`,
  `default_playfield_grid` + `step_and_emit_grid`, `RENDER_UNIT`,
  `set_sprites_visible`, `hash_u32`/`hash_f32` for asteroid shapes/spawns.
- Chaos meanings are the game's to define.
- Verify: game `cargo test` headless green + engine `/finish-task` if the engine changed.

(TASK-SNAKE-001 Snake shipped Jul 13 2026 — see PROGRESS.md and `log_archive.md`.)

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

---

## Task sourcing

Open technical debt is NOT duplicated here — it lives in the live docs
(root `TECH_DEBT.md` rollup → per-crate `TECH_DEBT.md` + `../games/TECH_DEBT.md`).
Engine feature gaps live in `PROJECT_ROADMAP.md` (next engine gaps:
Gap 1 `CameraFollow`, Gap 3 `Tilemap`). Pull from those when this queue is empty.

(The Phase 1 task list that used to live here shipped in full — entity CRUD,
picking, component add/remove, undo/redo — see `log_archive.md`. Retired
Jul 13 2026.)
