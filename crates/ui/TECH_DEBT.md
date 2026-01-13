# Technical Debt: ui

Last audited: January 2026

## Summary
- DRY violations: 4
- SRP violations: 2
- KISS violations: 1
- Architecture issues: 3

---

## DRY Violations

### [DRY-001] Duplicate glyph-to-draw-data conversion in context.rs
- **File:** `context.rs`
- **Lines:** 215-235, 255-279
- **Issue:** The pattern of converting `LayoutGlyph` to `GlyphDrawData` is duplicated between `label_styled()` and `label_with_font()`:
  ```rust
  let glyphs: Vec<GlyphDrawData> = layout.glyphs.iter().map(|g| {
      GlyphDrawData {
          bitmap: g.info.rasterized.bitmap.clone(),
          width: g.info.rasterized.width,
          height: g.info.rasterized.height,
          x: g.x,
          y: g.y,
          character: g.character,
      }
  }).collect();
  ```
  And the identical `TextDrawData` construction follows.
- **Suggested fix:** Extract to `fn layout_to_draw_data(layout: TextLayout, text: &str, position: Vec2, color: Color, font_size: f32) -> TextDrawData`.
- **Priority:** Medium

### [DRY-002] Duplicate checkbox drawing logic
- **File:** `context.rs`
- **Lines:** 375-403
- **Issue:** Checkbox uses the same background/border drawing pattern as buttons but reimplements it instead of reusing a helper. Same applies to checkmark inner rect drawing.
- **Suggested fix:** Extract common widget background drawing to a helper method like `draw_widget_background(&mut self, bounds: Rect, state: WidgetState, style: &ButtonStyle)`.
- **Priority:** Low

### [DRY-003] Duplicate UIContext constructor logic
- **File:** `context.rs`
- **Lines:** 53-61, 64-72
- **Issue:** `new()` and `with_theme()` have identical field initialization except for the theme:
  ```rust
  Self {
      interaction: InteractionManager::new(),
      draw_list: DrawList::new(),
      theme: ...,  // Only this differs
      window_size: Vec2::new(800.0, 600.0),
      font_manager: FontManager::new(),
  }
  ```
- **Suggested fix:** Use `new()` in `with_theme()`: `let mut ctx = Self::new(); ctx.theme = theme; ctx`.
- **Priority:** Low

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

### [ARCH-001] Glyph cache duplicates in engine_core
- **Files:** `font.rs` (this crate), `engine_core/src/contexts.rs`
- **Issue:** Two separate glyph caching mechanisms exist:
  1. `FontManager.glyph_cache` in ui crate - caches `GlyphInfo` by `GlyphKey`
  2. `GlyphCacheKey` in engine_core - caches GPU textures by (char, font_size, color_rgb)

  This duplication means glyphs may be cached twice at different levels.
- **Suggested fix:** Consolidate caching strategy. The ui crate should handle CPU-side caching (bitmaps), and engine_core should only cache GPU resources.
- **Priority:** Medium (related to engine_core KISS-001)

### [ARCH-002] rect.rs is essentially a re-export
- **File:** `rect.rs`
- **Lines:** 1-45
- **Issue:** This entire file just re-exports `common::Rect` and has tests. The tests are duplicating tests that likely exist in the common crate.
- **Suggested fix:** Either:
  1. Remove rect.rs entirely and re-export from lib.rs directly
  2. Or add UI-specific Rect extensions if needed
- **Priority:** Low (working, just redundant)

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
| Test coverage | 42 tests (100% pass rate) |
| `#[allow(...)]` | 2 instances (clippy lints in draw.rs) |
| High priority issues | 0 |
| Medium priority issues | 4 |
| Low priority issues | 6 |

---

## Recommendations

### Immediate Actions
None required - the crate is well-structured with no high-priority issues.

### Short-term Improvements
1. **Fix DRY-001** - Extract glyph-to-draw-data conversion helper
2. **Fix DRY-004** - Extract common text rendering logic
3. **Address ARCH-001** - Review glyph caching strategy with engine_core

### Technical Debt Backlog
- SRP-001: Consider FontManager refactoring if it grows
- ARCH-002: Clean up rect.rs re-export
- ARCH-003: Reduce TextDrawData redundancy

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| First-frame bug | "Font Rendering First-Frame Bug Fix" | RESOLVED |
| DRY-001: Glyph conversion | Not tracked | New finding |
| ARCH-001: Dual glyph caching | Related to engine_core KISS-001 | Known relationship |

**New issues to add to PROJECT_ROADMAP.md:**
- DRY-001: Duplicate glyph-to-draw-data conversion in context.rs
- ARCH-001: Glyph caching exists in both ui and engine_core crates

---

## Code Quality Notes

The ui crate is well-designed overall:
- Clean immediate-mode API
- Proper separation between context, drawing, interaction, and styling
- Good test coverage (42 tests)
- Correctly re-exports common types (Color, Rect) to avoid duplication
- Theme system is flexible with dark/light presets

The identified issues are mostly minor DRY violations and architectural considerations rather than fundamental problems.
