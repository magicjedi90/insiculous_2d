# Technical Debt: ui

Last audited: January 2026

## Summary
- DRY violations: 4 (3 resolved)
- SRP violations: 2
- KISS violations: 1
- Architecture issues: 3 (2 resolved)

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

### [DRY-004] Repeated font check and placeholder fallback
- **File:** `context.rs`
- **Lines:** 211-249, 252-279
- **Issue:** Both `label_styled()` and `label_with_font()` have similar structure:
  1. Attempt font layout
  2. On success: create TextDrawData and push
  3. On failure: push placeholder

  The main difference is `label_styled` checks for default font first.
- **Suggested fix:** Create a private helper `fn render_text_with_font(&mut self, text: &str, position: Vec2, color: Color, font_size: f32, font: FontHandle)` that handles the common logic.
- **Priority:** Medium

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

### [SRP-002] UIContext has large widget methods
- **File:** `context.rs`
- **Lines:** 153-479
- **Issue:** `UIContext` has ~330 lines of widget methods. Each widget method does:
  1. Interaction handling
  2. Style lookup
  3. State-based color selection
  4. Draw command generation

  While reasonable for immediate-mode UI, the file is large.
- **Suggested fix:** Consider extracting widget implementations to a `widgets.rs` module with helper structs.
- **Priority:** Low (idiomatic for immediate-mode UI)

---

## KISS Violations

### [KISS-001] WidgetPersistentState has unused flexibility
- **File:** `interaction.rs`
- **Lines:** 98-109
- **Issue:** `WidgetPersistentState` stores generic values (`float_value`, `bool_value`, `string_value`) that can be used for any widget. However, this flexibility is only used for garbage collection (the `seen_this_frame` flag). The other fields are:
  - Set in tests
  - Never used by actual widgets

  The slider stores its value in the caller, not in persistent state.
- **Suggested fix:** Either use this for actual widget state persistence or simplify to just track widget lifetime.
- **Priority:** Low (not causing problems)

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

| Metric | Value |
|--------|-------|
| Total source files | 7 |
| Total lines | ~1,500 |
| Test coverage | 39 tests (100% pass rate) |
| `#[allow(...)]` | 2 instances (clippy lints in draw.rs) |
| High priority issues | 0 |
| Medium priority issues | 3 |
| Low priority issues | 6 |

---

## Recommendations

### Immediate Actions
None required - the crate is well-structured with no high-priority issues.

### Short-term Improvements
1. ~~**Fix DRY-001** - Extract glyph-to-draw-data conversion helper~~ ✅ DONE
2. **Fix DRY-004** - Extract common text rendering logic
3. ~~**Address ARCH-001** - Review glyph caching strategy with engine_core~~ ✅ RESOLVED

### Technical Debt Backlog
- SRP-001: Consider FontManager refactoring if it grows
- ~~ARCH-002: Clean up rect.rs re-export~~ ✅ DONE
- ARCH-003: Reduce TextDrawData redundancy

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

### [DRY-007] Theme color hex values scattered
- **File:** `src/style.rs:34-72`
- **Issue:** Color constants duplicated between ButtonStyle, PanelStyle, and light theme defaults
- **Suggested fix:** Define theme_colors constants module
- **Priority:** Low | **Effort:** Small
