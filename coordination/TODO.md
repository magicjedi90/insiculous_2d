# Task Queue - Insiculous 2D


## (queue empty — next up: game 6, Frogger)

Phase A (games 1–5) AND Phase B (engine gaps: CameraFollow, Lifetime,
Tilemap) are COMPLETE — see PROGRESS.md and `log_archive.md`
(TASK-GAP-CAMERA-001 retired Jul 16 2026). Next per PROJECT_ROADMAP.md:
game 6 (Frogger) in `../games/`, the first Tilemap consumer — use the
`/new-game` skill.

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

---

## Task sourcing

Open technical debt is NOT duplicated here — it lives in the live docs
(root `TECH_DEBT.md` rollup → per-crate `TECH_DEBT.md` + `../games/TECH_DEBT.md`).
Engine feature gaps live in `PROJECT_ROADMAP.md` (Phase B gaps all shipped;
next work is Phase C games). Pull from those when this queue is empty.

(The Phase 1 task list that used to live here shipped in full — entity CRUD,
picking, component add/remove, undo/redo — see `log_archive.md`. Retired
Jul 13 2026.)
