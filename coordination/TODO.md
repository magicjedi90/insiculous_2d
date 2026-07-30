# Task Queue - Insiculous 2D

## The Deion Pivot (Jul 28 2026) — Phase E + H1 are the open front

Roadmap reworked via adversarial review (kimi round 1, 13 accepted / 2
rebutted; plan settled with Jesse). The 20 Games Challenge is PAUSED at game 7
(Tetris) until Phases E–I land. Full phase specs: `PROJECT_ROADMAP.md`.

### Ready now (parallel-safe: different crates)

- **TASK-E1** — `TextureFilter` knob (renderer + engine_core assets): config
  default + per-call override, `Linear` default for plain loads. Crates:
  renderer, engine_core.
- **TASK-E2** — `common::SheetGrid`: extract Tilemap's grid-UV math, refactor
  `Tilemap`/`tilemap_render.rs` onto it, behavior-identical + test-locked.
  Crates: common, ecs, engine_core.
- **TASK-E6** — Delete `crates/renderer/src/atlas.rs` (dead stub: never
  uploads pixels, zero consumers) + prelude/lib re-exports. Crate: renderer.
- **TASK-H1** — WASM spike (timeboxed, findings + working demo, NOT merged
  engine code): minimal wgpu28+winit hello triangle-or-sprite in browser with
  async init + `spawn_app` + `web-time`, one fetched texture, **audible sound
  end-to-end** (rodio-wasm go/no-go; fallback candidates: kira, web-sys
  AudioContext). Deliverables: `coordination/H1_SPIKE.md` findings +
  per-dependency wasm pass/fail list + web audio backend decision.

### Blocked / sequenced

- **TASK-E3** — `SpriteAnimation` rework (named clips, system writes
  `Sprite.tex_region` while playing). SINGLE-AGENT — crosses the SSOT chain
  (scene_data/serializer/loader, editor registry, undo). After E2.
- **TASK-E4** — `load_sprite_sheet()` + `.sheet.ron` schema (sheets default
  Nearest; clips are the stable API). After E1+E2. **E2+E4 merged = schema
  freeze** gating all asset production (F2+).
- **TASK-E5** — Scene serialization fixes (`#solid:RRGGBB` round-trip,
  `tex_region`/`visible` with serde defaults). The `#rgba` per-sprite
  save-error flips ONLY after F3 migrates Frogger's tileset to PNGs.
- **TASK-E7** — Sprite-shader alpha-cutoff (configurable threshold); closes
  renderer TECH_DEBT alpha/depth item.
- **TASK-E8/E9** — Inspector wiring (`/add-component`) + docs.
- **Phase F** — F1 style doc DRAFTED Jul 28 (`docs/DEION_STYLE.md` — castings
  + palette are proposals awaiting Jesse's sign-off; edit in place). F2–F5
  gated on schema freeze.
- **Phase G/H2+/I** — per PROJECT_ROADMAP.md dependency table.

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

## Task sourcing

Open technical debt is NOT duplicated here — it lives in the live docs
(root `TECH_DEBT.md` rollup → per-crate `TECH_DEBT.md` + `../games/TECH_DEBT.md`).
Engine feature gaps live in `PROJECT_ROADMAP.md` (now organized as Deion Pivot
Phases E–I). Pull from those when this queue is empty.

(Phase 1 editor task list and the Phase A/B queues shipped in full — see
`log_archive.md`.)
