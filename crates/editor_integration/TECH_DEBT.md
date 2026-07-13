# Technical Debt: editor_integration — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § editor_integration.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
(GPP-13 resolved Jul 13 2026 — editable inspector is registry-generated; see `log_archive.md`.)
- **GPP-12 (cross-ref):** ARCH-101 below is the same stringly-typed Type Object smell as breakout's `parse_brick_tag` — see `../games/TECH_DEBT.md`.

## Open Items

### [UX-001] No file picker dialog — scene path is a single constant — Low
- **File:** `src/constants.rs` (`DEFAULT_SCENE_PATH`), used by `editor_game/scene_io.rs`, `menu_actions.rs`, `shortcuts.rs`
- **Issue:** Open/Save As always use `scenes/scene.ron`. Real file dialog is Phase 2+ work; the path is at least defined in exactly one place.

### [ARCH-101] `handle_create_action` matches on menu label strings — Low
- **File:** `src/entity_ops.rs` (+ `editor_game/menu_actions.rs`)
- **Issue:** menu actions arrive as `&str` ("Create Sprite", …) matched by string; a typo'd label fails silently.
- **Fix:** a `MenuAction` enum produced by the menu system (compile-checked). **Effort:** Medium (touches editor menu API)

## Metrics

| Metric | Value |
|--------|-------|
| Test coverage | 66 tests (100% pass rate) |
| Files over 600 lines | 0 |
| Clippy warnings | 0 |
| High priority open | 0 |
| Medium priority open | 0 |
| Low priority open | 2 (UX-001, ARCH-101) |
