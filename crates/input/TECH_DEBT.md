# Technical Debt: input — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § input.

## Game Programming Patterns Audit (July 2026 — closed; history in `log_archive.md`)
No open items (GPP-L4 resolved Jul 13 2026 — see `log_archive.md`).

## Open Items

### [GAP-002] `MousePosition` / `(f32, f32)` instead of shared Vec2 — Low
`MousePosition` duplicates a 2D vector type; `movement_delta()` returns a bare tuple. Unifying touches `ui`, `editor`, `editor_integration` — do as its own small cross-crate pass.

### [GAP-003] No touch / gesture support — Low (feature gap)
No tap/drag/pinch recognition, no `WindowEvent::Touch` handling. Track in PROJECT_ROADMAP.md if mobile/web targets become real.

### [GAP-005] Fixed axis threshold / dead zone — Low
`AXIS_ACTIVATION_THRESHOLD` (0.5) and the backend dead zone (0.15) are
engine-wide constants; `InputSource` is `Eq + Hash` so per-binding `f32`
thresholds need a design (quantized payload or a side table) if tuning is
ever wanted.

### [GAP-006] Pad ids are connection-ordered — Low
"Pad 0 / pad 1" can swap across sessions if plug order changes, so a
persisted explicit `assign_pad` re-assignment may point at the wrong
physical pad next launch. Defaults re-resolve naturally; a fix would key on
a stable device identity from the backend.

## Metrics (post-July-2026 player-input layer + gilrs backend)

| Metric | Value |
|--------|-------|
| Tests | 77 passing, 0 ignored |
| Clippy warnings | 0 (including `--all-targets`) |
| High priority open | 0 |
| Medium priority open | 0 |
| Low priority open | 4 |
