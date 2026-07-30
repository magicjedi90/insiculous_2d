# Task Queue - Insiculous 2D

## The Deion Pivot (Jul 28 2026) — Phase E + H1 are the open front

Roadmap reworked via adversarial review (kimi round 1, 13 accepted / 2
rebutted; plan settled with Jesse). The 20 Games Challenge is PAUSED at game 7
(Tetris) until Phases E–I land. Full phase specs: `PROJECT_ROADMAP.md`.

(H1 listen test PASSED Jul 30 2026 — Jesse confirmed all four audio checks
in Firefox: SFX bus math, seamless music loop, live master duck, stop.
Audio decision FINAL: stay on rodio. Render was screenshot-verified by the
spike agent; note Firefox needs a full restart after enabling
`dom.webgpu.enabled`, not just a new tab.)

### Ready now

- **TASK-E3** — `SpriteAnimation` rework (named clips, system writes
  `Sprite.tex_region` while playing; consume `common::SheetGrid` —
  `uv_rect_checked` is the intended accessor). SINGLE-AGENT — crosses the
  SSOT chain (scene_data/serializer/loader, editor registry, undo).
  UNBLOCKED: E2 shipped Jul 30. Pre-task: extract `TextureFilter` into its
  own renderer module first — `texture.rs` sits at 591/600 lines.
- **TASK-E4** — `load_sprite_sheet()` + `.sheet.ron` schema (sheets default
  Nearest; clips are the stable API). UNBLOCKED: E1+E2 shipped Jul 30.
  Open schema decision: `SheetGrid::Deserialize` requires explicit `cell_uv`
  on the wire; make it optional-and-derived via a serde shim if `.sheet.ron`
  wants that. **E2+E4 merged = schema freeze** gating all asset production
  (F2+).

(E1, E2, E6 shipped Jul 30 2026 — see PROGRESS.md.)

### Blocked / sequenced
- **TASK-E5 (remaining half)** — the `#rgba` per-sprite save-error, which
  flips ONLY after F3 migrates Frogger's tileset to PNGs. (The round-trip
  half — `#solid:RRGGBB` recording, `tex_region`/`visible` serde — shipped
  Jul 30 2026, see PROGRESS.md.)
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
