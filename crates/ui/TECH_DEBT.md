# Technical Debt: ui — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § ui.

## Game Programming Patterns Audit (July 2026 — closed; history in `log_archive.md`)
- [ ] **GPP-L8 (Low, Flyweight):** `GlyphInfo` stores `character`/`font_size` duplicating its cache key (`font/glyph_cache.rs:38-42`); `TextDrawData` char duplication tracked as ARCH-003 below. Strip opportunistically.

## Open Items

### [JUN-T1] Text input is numeric-only and keyboard-layout-blind — Medium
- **File:** `interaction.rs` (`keycode_to_char`, `InputState::from_input_handler`)
- **Issue:** typed characters synthesized from a hardcoded digit/period/minus keycode list. General text input (entity names, search boxes) is impossible; physical-key→char mapping ignores keyboard layouts.
- **Fix:** plumb winit character events through `InputHandler` (input crate change).
- **Priority:** Medium (blocks editor rename/search widgets) | **Effort:** Medium

### [ARCH-003] TextDrawData duplicates GlyphDrawData info — Low
- **File:** `draw.rs:26-43` — `text: String` + per-glyph `character` duplicate character info.
- **Fix:** remove `text` (reconstruct from glyphs) or remove `character` from `GlyphDrawData`.

### [JUN-T2] `scroll_delta` is captured but no widget consumes it — Low
- **File:** `interaction.rs` — snapshotted every frame but there is no scroll-area widget.
- **Fix:** add a `scroll_area` widget or drop the field until one exists.

### [JUN-T3] No layout helpers — Low (roadmap)
- Every caller hand-places absolute `Rect`s; no row/column/anchor layout. Phase 2+ concern.

## Metrics

| Metric | Value (June 2026) |
|--------|-------------------|
| Test coverage | 80 tests (100% pass rate) |
| `#[allow(...)]` | 2 (documented clippy exceptions) |
| High priority open | 0 |
| Medium priority open | 1 (JUN-T1) |
| Low priority open | 4 (ARCH-003, JUN-T2, JUN-T3, GPP-L8) |
