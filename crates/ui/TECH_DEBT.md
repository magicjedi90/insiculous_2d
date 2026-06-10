# Technical Debt: ui

Last audited: June 2026

## Summary
- DRY violations: 7 (all resolved)
- SRP violations: 2 (open: SRP-001 FontManager, SRP-002/context.rs size)
- KISS violations: 1 (resolved)
- Architecture issues: 3 (2 resolved, ARCH-003 open)
- June 2026 audit: 10 new findings, 8 fixed, 2 tracked (see bottom)

---

## DRY Violations

### ~~[DRY-001] Duplicate glyph-to-draw-data conversion in context.rs~~ ✅ RESOLVED
- **File:** `context.rs`
- **Resolution:** Extracted `layout_to_draw_data(&TextLayout, &str, Vec2, Color, f32) -> TextDrawData` helper method.
  Both `label_styled()` and `label_with_font()` now use this helper to convert font layout to draw data.
- **Resolved:** January 2026

### ~~[DRY-002] Duplicate checkbox drawing logic~~ ✅ RESOLVED
- **File:** `context.rs`
- **Resolution:** Extracted `widget_background_color(&self, state: WidgetState) -> Color` helper method.
  Both `button_styled()` and `checkbox()` now use this helper to get the background color based on widget state.
- **Resolved:** January 2026

### ~~[DRY-003] Duplicate UIContext constructor logic~~ ✅ RESOLVED
- **File:** `context.rs`
- **Resolution:** `with_theme()` now delegates to `new()`:
  ```rust
  pub fn with_theme(theme: Theme) -> Self {
      let mut ctx = Self::new();
      ctx.theme = theme;
      ctx
  }
  ```
- **Resolved:** January 2026

### ~~[DRY-004] Repeated font check and placeholder fallback~~ ✅ RESOLVED
- **File:** `context.rs`
- **Resolution:** The pattern had grown to 4 copies (`label_styled`, `label_with_font`,
  `button_styled`, `draw_float_input_box`, plus `label_in_bounds`). Extracted:
  - `draw_text_with_font(font: Option<FontHandle>, ...)` — the layout-or-placeholder tail
  - `draw_text_at_baseline(...)` — same, with the default font
  - `text_pos_in_bounds(text, bounds, align, font_size, padding)` — measure → align → vertically center → baseline
  - `estimate_text_size(text, font_size)` — single home for the no-font character-count heuristic
    (was 5 scattered copies of `len() * font_size * 0.6` / `* 0.3` / `* 0.8` magic numbers)

  All text-drawing widgets now route through these helpers.
- **Resolved:** June 2026

---

## SRP Violations

### [SRP-001] FontManager handles too many concerns
- **File:** `font.rs`
- **Lines:** 100-315
- **Issue:** `FontManager` handles 5 distinct responsibilities:
  1. Font loading (from bytes, files)
  2. Font storage (HashMap management)
  3. Glyph rasterization
  4. Glyph caching
  5. Text layout/measurement

  The `layout_text` method (lines 215-282) is particularly complex at ~70 lines.
- **Suggested fix:** Consider splitting:
  - `FontLoader` - Loading fonts from various sources
  - `FontManager` - Font storage and retrieval
  - `TextLayouter` - Text layout and measurement
  - `GlyphCache` - Glyph caching
- **Priority:** Medium (but working well, so low urgency)

### [SRP-002] context.rs exceeds the 600-line project rule
- **File:** `context.rs` (~990 lines, ~280 of which are inline tests)
- **Issue:** The widget methods themselves are now short (June 2026 helper extraction),
  but the file still violates the project's 600-line guideline.
- **Suggested fix:** Split `impl UIContext` across modules — e.g. `text.rs` (label/measure
  family) and `widgets.rs` (button/slider/checkbox/float_input) — and/or move the inline
  test module to `tests/`. Pure mechanical move, no API change.
- **Priority:** Medium (deferred from June 2026 audit by scope decision)

---

## KISS Violations

### ~~[KISS-001] WidgetPersistentState has unused flexibility~~ ✅ RESOLVED
- **File:** `interaction.rs`
- **Resolution:** Deleted `float_value` and `bool_value` (set only in tests, never read by
  widgets). Kept `string_value` (used by `float_input`'s edit buffer) and `seen_this_frame`
  (GC flag).
- **Resolved:** June 2026

---

## Architecture Issues

### ~~[ARCH-001] Glyph cache duplicates in engine_core~~ ✅ RESOLVED
- **Files:** `font.rs` (this crate), `engine_core/src/contexts.rs`
- **Resolution:** The dual caching is intentional and correct - they serve different purposes:
  1. `FontManager.glyph_cache` in ui crate - caches CPU-side rasterized glyph bitmaps (avoids re-rasterization)
  2. `glyph_textures` in engine_core - caches GPU textures created from bitmaps (avoids GPU uploads)

  **Fixed:** `layout_text()` now properly uses `rasterize_glyph()` which utilizes the glyph cache.
  Previously, `layout_text()` was bypassing the cache and re-rasterizing every glyph on every call.
  This was the actual bug causing performance waste.
- **Resolved:** January 2026

### ~~[ARCH-002] rect.rs is essentially a re-export~~ ✅ RESOLVED
- **File:** `rect.rs` (removed)
- **Resolution:** Removed `rect.rs` entirely. `Rect` is now re-exported directly from `common` in `lib.rs`:
  ```rust
  pub use common::Rect;
  ```
  The duplicate tests (4 tests) were removed as they duplicated coverage from `common::rect` tests.
- **Resolved:** January 2026

### [ARCH-003] TextDrawData duplicates GlyphDrawData info
- **File:** `draw.rs`
- **Lines:** 26-43
- **Issue:** `TextDrawData` contains:
  - `text: String` (the text string)
  - `glyphs: Vec<GlyphDrawData>` (includes character for each glyph)

  The character information is duplicated - stored both as the string and in each glyph. The `text` field is only used "for reference" per the comment.
- **Suggested fix:** Either remove `text` field (reconstruct from glyphs if needed) or remove `character` from `GlyphDrawData` since position and bitmap are what's needed for rendering.
- **Priority:** Low

---

## Previously Resolved (Reference)

These issues from ANALYSIS.md have been resolved:

| Issue | Resolution |
|-------|------------|
| First-frame font rendering bug | FIXED: Removed static PRINTED flag in `label_styled()` |
| Font retry logic | FIXED: Now retries font rendering every frame |

---

## Metrics

| Metric | Value (June 2026) |
|--------|-------------------|
| Total source files | 6 |
| Total source lines | ~2,000 (incl. inline tests) |
| Test coverage | 68 tests (100% pass rate) |
| `#[allow(...)]` | 2 (`clippy::too_many_arguments` on `DrawList::slider`, `clippy::should_implement_trait` on `WidgetId::from_str`) |
| High priority issues | 0 |
| Medium priority issues | 3 (SRP-001, SRP-002, JUN-T1) |
| Low priority issues | 3 (ARCH-003, JUN-T2, JUN-T3) |

---

## Recommendations

### Immediate Actions
None required - the crate is well-structured with no high-priority issues.

### Short-term Improvements
1. ~~**Fix DRY-001** - Extract glyph-to-draw-data conversion helper~~ ✅ DONE
2. ~~**Fix DRY-004** - Extract common text rendering logic~~ ✅ DONE (June 2026)
3. ~~**Address ARCH-001** - Review glyph caching strategy with engine_core~~ ✅ RESOLVED

### Technical Debt Backlog
- SRP-001: Consider FontManager refactoring if it grows
- SRP-002: Split context.rs to satisfy the 600-line rule
- ~~ARCH-002: Clean up rect.rs re-export~~ ✅ DONE
- ARCH-003: Reduce TextDrawData redundancy
- JUN-T1: General text input (character events)

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| First-frame bug | "Font Rendering First-Frame Bug Fix" | RESOLVED |
| DRY-001: Glyph conversion | Not tracked | RESOLVED |
| ARCH-001: Dual glyph caching | Related to engine_core KISS-001 | RESOLVED |

---

## Code Quality Notes

The ui crate is well-designed overall:
- Clean immediate-mode API
- Proper separation between context, drawing, interaction, and styling
- Good test coverage (42 tests)
- Correctly re-exports common types (Color, Rect) to avoid duplication
- Theme system is flexible with dark/light presets

The identified issues are mostly minor DRY violations and architectural considerations rather than fundamental problems.

---

## New Findings (February 2026 Audit)

4 new issues (0 High, 1 Medium, 3 Low)

### ~~[PERF-001] measure_text() re-rasterizes glyphs bypassing cache~~ ✅ RESOLVED
- **File:** `src/font.rs:332-352`
- **Resolution:** Replaced `font.rasterize()` with `font.metrics()` which returns advance width without performing bitmap rasterization. No signature change needed.
- **Resolved:** February 2026

### ~~[DRY-005] Depth calculation repeated 8x in DrawList~~ ✅ RESOLVED
- **File:** `src/draw.rs`
- **Resolution:** Extracted `next_depth(&self) -> f32` helper method. All 7 occurrences replaced with calls to the helper.
- **Resolved:** February 2026

### ~~[DRY-006] Text baseline positioning duplicated~~ ✅ RESOLVED
- **File:** `src/context.rs`
- **Resolution:** Extracted `baseline_y(&self, text_top, font_size, font_handle) -> f32` helper method. Both `button_styled()` and `label_in_bounds()` now use this helper.
- **Resolved:** February 2026

### ~~[DRY-007] Theme color hex values scattered~~ ✅ RESOLVED
- **File:** `src/style.rs`
- **Resolution:** Added a private `palette` module with named `dark`/`light` constants
  (`SURFACE`, `SURFACE_HOVERED`, `BORDER`, `ACCENT`, …). All style `Default` impls and
  `Theme::light()` now reference the palette; each hex value is defined once per theme.
- **Resolved:** June 2026

---

## June 2026 Audit (DRY/SRP/KISS/best-practices/reusability)

Scope decision: fix correctness, DRY, dead-code, and theming issues; defer structural
splits (SRP-001, SRP-002) as tracked debt.

### Fixed

| ID | Issue | Resolution |
|----|-------|------------|
| JUN-001 | `unwrap()` outside tests in `font.rs` (`rasterize_glyph` double-lookup ×2, `layout_text` after `contains_key`) — violated workspace rule | Restructured around single fallible lookups with `ok_or_else` |
| JUN-002 | Per-frame double glyph-bitmap clone: `layout_text` cloned `GlyphInfo` (incl. `Vec<u8>` bitmap) per glyph, then `layout_to_draw_data` cloned the bitmap again — two heap copies per glyph per label per frame | `RasterizedGlyph.bitmap` and `GlyphDrawData.bitmap` are now `Arc<[u8]>`; clones are O(1) refcount bumps. Downstream `&glyph.bitmap` coerces to `&[u8]` unchanged |
| JUN-003 | Persistent-state GC dropped a focused widget's state (e.g. `float_input` edit buffer) the moment it skipped a frame; comment falsely claimed a multi-frame grace period | `end_frame` retains the focused widget's state even when unseen; comment now describes the real policy. Regression tests added |
| JUN-004 | `DrawCommand::depth()` returns `0.0` for clip commands — sorting by depth would tear `PushClipRect`/`PopClipRect` pairs apart | Documented the consume-in-submission-order contract on `depth()` (current consumer `engine_core::ui_integration` already iterates in order) |
| JUN-005 | Dead public API with zero workspace callers | Deleted: `DrawList::button()`, `DrawList::text_simple()` (fake `Text` with empty glyphs), `DrawList::with_base_depth()` (base depth folded into `UI_BASE_DEPTH` const), `InteractionManager::interact_draggable()` (misleading "delta" = offset-from-center), `FontManager::load_default_font()` (stub that always errored) |
| JUN-006 | `float_input` duplicated commit logic ×3 (parse + clamp + unfocus + draw + return) inside an 80-line state machine | Extracted `commit_float_input()` and `draw_float_value()`; Enter/Tab and click-outside share one path |
| JUN-007 | Theme bypass: `draw_float_input_box` hardcoded 4 colors, `font_size 13.0`, `padding 4.0`; `label_in_bounds`/`checkbox_labeled` hardcoded `8.0` padding — widgets couldn't be reskinned via `Theme` | Added `TextInputStyle` to `Theme` (dark + light variants); paddings routed through `theme.panel.padding` / `theme.button.padding` |
| JUN-008 | Cosmetic: `font_metrics` returned `crate::font::FontMetrics` instead of the re-export; pre-existing clippy lints in tests (`len() >= 1`, `3.14` approx-PI) | Fixed |

### New tracked items

### [JUN-T1] Text input is numeric-only and keyboard-layout-blind
- **File:** `interaction.rs` (`keycode_to_char`, `InputState::from_input_handler`)
- **Issue:** `InputState` synthesizes typed characters from a hardcoded list of digit/period/minus
  keycodes. General text input (entity names, search boxes) is impossible, and the physical-key →
  char mapping ignores keyboard layouts. A real text widget needs winit character events plumbed
  through `InputHandler`.
- **Priority:** Medium (blocks editor rename/search widgets) | **Effort:** Medium (input crate change)

### [JUN-T2] `scroll_delta` is captured but no widget consumes it
- **File:** `interaction.rs`
- **Issue:** `InputState.scroll_delta` is snapshotted every frame but there is no scroll-area
  widget; consumers (editor panels) implement their own scrolling.
- **Suggested fix:** Either add a `scroll_area` widget or drop the field until one exists.
- **Priority:** Low

### [JUN-T3] No layout helpers
- **Issue:** Every caller hand-places absolute `Rect`s; there is no row/column/anchor layout.
  Fine for the current consumers, but a reusability gap for game UIs at scale. Phase 2+ concern.
- **Priority:** Low (roadmap)
