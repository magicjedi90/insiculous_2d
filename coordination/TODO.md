# Task Queue - Insiculous 2D


## TASK-GAP-CAMERA-001: `CameraFollow` behavior (Phase B, Gap 1) — NEXT UP

Phase A (games 1–5) is COMPLETE — Asteroids shipped Jul 13 2026 (see
PROGRESS.md and `log_archive.md`). Next per PROJECT_ROADMAP.md: Phase B
engine gaps, starting with Gap 1 `CameraFollow` (blocks games 10–19), then
Gap 3 `Tilemap` (blocks Frogger/Pac-Man/Zelda-likes). Full specs live in
`PROJECT_ROADMAP.md` Phase B — write the task breakdown here when claiming.

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
