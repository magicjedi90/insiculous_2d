# Task Queue - Insiculous 2D


## TASK-SNAKE-001: Snake (Game 4 of the 20 Games Challenge) — NEXT UP

**Start with `/new-game snake`.** Roadmap spec (PROJECT_ROADMAP Phase A, Game 4):
growing snake controlled in four directions, eat food to grow, avoid self.
No physics needed — grid-based movement and self-collision are pure game logic
(timer-driven steps). ~300 lines.

Build notes:
- **No engine gaps**: grid logic, segment-following, food spawning are all game code.
  `hash_u32(frame_count)` for food placement (deterministic, no rand).
- **Shared scaffolding** (prelude): `MenuInput`, `spawn_background`,
  `default_playfield_grid` + `step_and_emit_grid`, `RENDER_UNIT`, `set_sprites_visible`.
- **Chaos meanings are the game's to define** (e.g. Insane = faster ticks,
  Ridiculous = two food items / wrap-around walls?, Insiculous = both).
- Segments are plain sprites (no colliders) — self/wall collision is grid-cell math,
  headless-testable.
- Verify: game `cargo test` headless green + engine `/finish-task` if the engine changed.

(TASK-SI-001 Space Invaders shipped Jul 13 2026 — see PROGRESS.md and `log_archive.md`.)

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
