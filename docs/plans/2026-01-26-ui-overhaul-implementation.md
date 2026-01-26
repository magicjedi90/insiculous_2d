# UI Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix text positioning (baseline), add scissor rect clipping, support DPI scaling, and standardize layout constants.

**Architecture:** Four independent concerns implemented in dependency order: (1) font metrics foundation, (2) baseline text positioning, (3) scissor clipping, (4) DPI scaling. Layout constants are a simple refactor done early.

**Tech Stack:** Rust, fontdue (font metrics), wgpu (scissor rects), winit (DPI events)

---

## Task 1: Add FontMetrics Struct and API

**Files:**
- Modify: `crates/ui/src/font.rs:1-100` (add struct and method)
- Modify: `crates/ui/src/lib.rs:49` (add export)

**Step 1: Write the failing test**

Add to `crates/ui/src/font.rs` at end of `mod tests`:

```rust
#[test]
fn test_font_metrics_struct() {
    let metrics = FontMetrics {
        ascent: 14.0,
        descent: -4.0,
        line_height: 20.0,
    };
    assert_eq!(metrics.ascent, 14.0);
    assert_eq!(metrics.descent, -4.0);
    assert_eq!(metrics.line_height, 20.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui font::tests::test_font_metrics_struct`
Expected: FAIL with "cannot find type `FontMetrics`"

**Step 3: Write minimal implementation**

Add after `FontHandle` struct (around line 27) in `crates/ui/src/font.rs`:

```rust
/// Font measurement information for layout calculations.
///
/// These metrics are essential for proper text positioning:
/// - `ascent`: Use to know how much space text needs above the baseline
/// - `descent`: Use to know how much space text needs below the baseline
/// - `line_height`: Use for spacing between lines of text
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Distance from baseline to top of tallest glyph (positive value)
    pub ascent: f32,
    /// Distance from baseline to bottom of descenders (negative value)
    pub descent: f32,
    /// Recommended distance between baselines for line spacing
    pub line_height: f32,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui font::tests::test_font_metrics_struct`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/src/font.rs
git commit -m "feat(ui): add FontMetrics struct for text layout"
```

---

## Task 2: Add FontManager::metrics() Method

**Files:**
- Modify: `crates/ui/src/font.rs:118-310` (add method to FontManager impl)

**Step 1: Write the failing test**

Add to `crates/ui/src/font.rs` in `mod tests`:

```rust
#[test]
fn test_font_manager_metrics_no_font() {
    let manager = FontManager::new();
    let handle = FontHandle { id: 999 };
    assert!(manager.metrics(handle, 16.0).is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui font::tests::test_font_manager_metrics_no_font`
Expected: FAIL with "no method named `metrics`"

**Step 3: Write minimal implementation**

Add to `FontManager` impl in `crates/ui/src/font.rs` (after `get_font` method, around line 175):

```rust
/// Get font metrics for layout calculations.
///
/// Returns None if the font handle is invalid or metrics unavailable.
pub fn metrics(&self, handle: FontHandle, font_size: f32) -> Option<FontMetrics> {
    let font = self.fonts.get(&handle.id)?;
    let line_metrics = font.horizontal_line_metrics(font_size)?;

    Some(FontMetrics {
        ascent: line_metrics.ascent,
        descent: line_metrics.descent,
        line_height: line_metrics.ascent - line_metrics.descent + line_metrics.line_gap,
    })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui font::tests::test_font_manager_metrics_no_font`
Expected: PASS

**Step 5: Export FontMetrics from lib.rs**

Modify `crates/ui/src/lib.rs` line 49, change:
```rust
pub use font::{FontError, FontHandle, FontManager, GlyphInfo, LayoutGlyph, RasterizedGlyph, TextLayout};
```
to:
```rust
pub use font::{FontError, FontHandle, FontManager, FontMetrics, GlyphInfo, LayoutGlyph, RasterizedGlyph, TextLayout};
```

**Step 6: Run all font tests**

Run: `cargo test -p ui font::tests`
Expected: All pass

**Step 7: Commit**

```bash
git add crates/ui/src/font.rs crates/ui/src/lib.rs
git commit -m "feat(ui): add FontManager::metrics() for baseline positioning"
```

---

## Task 3: Create Editor Layout Constants

**Files:**
- Create: `crates/editor/src/layout.rs`
- Modify: `crates/editor/src/lib.rs` (add module)

**Step 1: Create layout.rs with constants**

Create `crates/editor/src/layout.rs`:

```rust
//! Editor layout constants for consistent spacing.
//!
//! Use these constants instead of magic numbers throughout the editor.
//! Changing a value here updates the entire editor's layout.

/// Standard padding inside panels and containers
pub const PADDING: f32 = 8.0;

/// Smaller padding for tight spaces
pub const PADDING_SMALL: f32 = 4.0;

/// Space between adjacent elements
pub const SPACING: f32 = 4.0;

/// Panel header height
pub const HEADER_HEIGHT: f32 = 24.0;

/// Default line height (fallback when no font metrics available)
pub const LINE_HEIGHT: f32 = 20.0;

/// Menu bar height
pub const MENU_BAR_HEIGHT: f32 = 24.0;

/// Menu item height
pub const MENU_ITEM_HEIGHT: f32 = 24.0;

/// Toolbar height
pub const TOOLBAR_HEIGHT: f32 = 40.0;

/// Toolbar button size
pub const TOOLBAR_BUTTON_SIZE: f32 = 32.0;

/// Resize handle hit area size
pub const RESIZE_HANDLE_SIZE: f32 = 4.0;

/// Minimum panel size when resizing
pub const MIN_PANEL_SIZE: f32 = 100.0;

/// Default panel width for left/right docked panels
pub const DEFAULT_PANEL_WIDTH: f32 = 250.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constants_positive() {
        assert!(PADDING > 0.0);
        assert!(HEADER_HEIGHT > 0.0);
        assert!(LINE_HEIGHT > 0.0);
        assert!(MIN_PANEL_SIZE > 0.0);
    }

    #[test]
    fn test_padding_hierarchy() {
        assert!(PADDING > PADDING_SMALL);
    }
}
```

**Step 2: Add module to lib.rs**

In `crates/editor/src/lib.rs`, add after line 36 (`mod viewport_input;`):

```rust
pub mod layout;
```

**Step 3: Run tests**

Run: `cargo test -p editor layout::tests`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/editor/src/layout.rs crates/editor/src/lib.rs
git commit -m "feat(editor): add layout constants module"
```

---

## Task 4: Add TextAlign Enum to UI

**Files:**
- Modify: `crates/ui/src/context.rs:1-15` (add enum)
- Modify: `crates/ui/src/lib.rs` (export)

**Step 1: Write the failing test**

Add to `crates/ui/src/context.rs` in `mod tests`:

```rust
#[test]
fn test_text_align_default() {
    let align = TextAlign::default();
    assert_eq!(align, TextAlign::Left);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui context::tests::test_text_align_default`
Expected: FAIL with "cannot find type `TextAlign`"

**Step 3: Write minimal implementation**

Add at the top of `crates/ui/src/context.rs` (after imports, before UIContext struct):

```rust
/// Text alignment within a bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// Align text to the left edge
    #[default]
    Left,
    /// Center text horizontally
    Center,
    /// Align text to the right edge
    Right,
}
```

**Step 4: Export from lib.rs**

Add to `crates/ui/src/lib.rs` line 47:
```rust
pub use context::{UIContext, TextAlign};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p ui context::tests::test_text_align_default`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/ui/src/context.rs crates/ui/src/lib.rs
git commit -m "feat(ui): add TextAlign enum"
```

---

## Task 5: Add UIContext::font_metrics() Helper

**Files:**
- Modify: `crates/ui/src/context.rs` (add method to UIContext impl)

**Step 1: Write the failing test**

Add to `crates/ui/src/context.rs` in `mod tests`:

```rust
#[test]
fn test_ui_context_font_metrics_none_without_font() {
    let ui = UIContext::new();
    // No font loaded, should return None
    assert!(ui.font_metrics(16.0).is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui context::tests::test_ui_context_font_metrics_none_without_font`
Expected: FAIL with "no method named `font_metrics`"

**Step 3: Write minimal implementation**

Add to UIContext impl in `crates/ui/src/context.rs` (after `font_manager_mut`, around line 100):

```rust
/// Get font metrics for the default font at the given size.
///
/// Returns None if no font is loaded or metrics are unavailable.
/// Use this for calculating text positions with baseline alignment.
pub fn font_metrics(&self, font_size: f32) -> Option<crate::font::FontMetrics> {
    let font = self.font_manager.default_font()?;
    self.font_manager.metrics(font, font_size)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui context::tests::test_ui_context_font_metrics_none_without_font`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/src/context.rs
git commit -m "feat(ui): add UIContext::font_metrics() helper"
```

---

## Task 6: Add label_in_bounds() Method

**Files:**
- Modify: `crates/ui/src/context.rs` (add method)

**Step 1: Write the failing test**

Add to `crates/ui/src/context.rs` in `mod tests`:

```rust
#[test]
fn test_ui_context_label_in_bounds() {
    let mut ui = UIContext::new();
    let bounds = Rect::new(10.0, 10.0, 200.0, 30.0);

    // Should not panic even without font
    ui.label_in_bounds("Test", bounds, TextAlign::Center);

    // Should generate a draw command (placeholder without font)
    assert_eq!(ui.draw_list().len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui context::tests::test_ui_context_label_in_bounds`
Expected: FAIL with "no method named `label_in_bounds`"

**Step 3: Write minimal implementation**

Add to UIContext impl in `crates/ui/src/context.rs` (after `label_with_font`, around line 294):

```rust
/// Draw a label centered within bounds.
///
/// This method handles vertical centering automatically using font metrics
/// when available, falling back to approximate centering otherwise.
/// Use this for text in buttons, headers, and other bounded containers.
pub fn label_in_bounds(&mut self, text: &str, bounds: Rect, align: TextAlign) {
    let font_size = self.theme.text.font_size;
    let color = self.theme.text.color;
    let padding = 8.0; // Standard padding

    // Calculate X position based on alignment
    let x = if let Some(font_handle) = self.font_manager.default_font() {
        if let Ok(text_size) = self.font_manager.measure_text(font_handle, text, font_size) {
            match align {
                TextAlign::Left => bounds.x + padding,
                TextAlign::Center => bounds.x + (bounds.width - text_size.x) / 2.0,
                TextAlign::Right => bounds.x + bounds.width - text_size.x - padding,
            }
        } else {
            bounds.x + padding
        }
    } else {
        // Fallback: estimate text width
        let estimated_width = text.len() as f32 * font_size * 0.6;
        match align {
            TextAlign::Left => bounds.x + padding,
            TextAlign::Center => bounds.x + (bounds.width - estimated_width) / 2.0,
            TextAlign::Right => bounds.x + bounds.width - estimated_width - padding,
        }
    };

    // Calculate Y position (baseline) for vertical centering
    let baseline_y = if let Some(metrics) = self.font_metrics(font_size) {
        // Proper centering: baseline positioned so text is visually centered
        // ascent is positive (above baseline), descent is negative (below baseline)
        bounds.y + (bounds.height + metrics.ascent + metrics.descent) / 2.0
    } else {
        // Fallback: approximate center (treats position as top-left)
        bounds.y + bounds.height / 2.0 + font_size * 0.35
    };

    self.label(text, Vec2::new(x, baseline_y));
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui context::tests::test_ui_context_label_in_bounds`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/src/context.rs
git commit -m "feat(ui): add label_in_bounds() for centered text"
```

---

## Task 7: Add Clip Rect Draw Commands

**Files:**
- Modify: `crates/ui/src/draw.rs` (add enum variants and methods)

**Step 1: Write the failing test**

Add to `crates/ui/src/draw.rs` in `mod tests`:

```rust
#[test]
fn test_draw_list_clip_rect() {
    let mut list = DrawList::new();
    let bounds = Rect::new(10.0, 10.0, 100.0, 100.0);

    list.push_clip_rect(bounds);
    list.rect(Rect::new(20.0, 20.0, 50.0, 50.0), Color::RED);
    list.pop_clip_rect();

    assert_eq!(list.len(), 3);

    // First command should be PushClipRect
    if let DrawCommand::PushClipRect { bounds: clip_bounds } = &list.commands()[0] {
        assert_eq!(clip_bounds.x, 10.0);
    } else {
        panic!("Expected PushClipRect");
    }

    // Last command should be PopClipRect
    assert!(matches!(list.commands()[2], DrawCommand::PopClipRect));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui draw::tests::test_draw_list_clip_rect`
Expected: FAIL with "no variant named `PushClipRect`"

**Step 3: Write minimal implementation**

Add to `DrawCommand` enum in `crates/ui/src/draw.rs` (after `Line` variant):

```rust
    /// Begin clipping to a rectangular region.
    /// All subsequent draws are clipped to this bounds until PopClipRect.
    PushClipRect {
        bounds: Rect,
    },
    /// End the current clipping region, restore previous clip state.
    PopClipRect,
```

Update the `depth()` method in `impl DrawCommand`:

```rust
pub fn depth(&self) -> f32 {
    match self {
        DrawCommand::Rect { depth, .. } => *depth,
        DrawCommand::RectBorder { depth, .. } => *depth,
        DrawCommand::Text { depth, .. } => *depth,
        DrawCommand::TextPlaceholder { depth, .. } => *depth,
        DrawCommand::Circle { depth, .. } => *depth,
        DrawCommand::Line { depth, .. } => *depth,
        DrawCommand::PushClipRect { .. } => 0.0, // Clip commands don't have depth
        DrawCommand::PopClipRect => 0.0,
    }
}
```

Add methods to `impl DrawList` (after `line` method):

```rust
/// Begin clipping all subsequent draws to the given bounds.
/// Must be paired with `pop_clip_rect()`.
pub fn push_clip_rect(&mut self, bounds: Rect) {
    self.commands.push(DrawCommand::PushClipRect { bounds });
}

/// End the current clip region, restoring the previous clip state.
pub fn pop_clip_rect(&mut self) {
    self.commands.push(DrawCommand::PopClipRect);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui draw::tests::test_draw_list_clip_rect`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/src/draw.rs
git commit -m "feat(ui): add PushClipRect/PopClipRect draw commands"
```

---

## Task 8: Add UIContext Clip Rect Methods

**Files:**
- Modify: `crates/ui/src/context.rs` (add methods)

**Step 1: Write the failing test**

Add to `crates/ui/src/context.rs` in `mod tests`:

```rust
#[test]
fn test_ui_context_clip_rect() {
    let mut ui = UIContext::new();
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);

    ui.push_clip_rect(bounds);
    ui.rect(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED);
    ui.pop_clip_rect();

    assert_eq!(ui.draw_list().len(), 3);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui context::tests::test_ui_context_clip_rect`
Expected: FAIL with "no method named `push_clip_rect`"

**Step 3: Write minimal implementation**

Add to UIContext impl in `crates/ui/src/context.rs` (after `line` method):

```rust
/// Begin clipping all subsequent draws to the given bounds.
///
/// Use this to prevent content from rendering outside panel boundaries.
/// Must be paired with `pop_clip_rect()`. Clip regions can be nested.
pub fn push_clip_rect(&mut self, bounds: Rect) {
    self.draw_list.push_clip_rect(bounds);
}

/// End the current clip region, restoring the previous clip state.
///
/// If no clip region is active, this resets to the full viewport.
pub fn pop_clip_rect(&mut self) {
    self.draw_list.pop_clip_rect();
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui context::tests::test_ui_context_clip_rect`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/src/context.rs
git commit -m "feat(ui): add push_clip_rect/pop_clip_rect to UIContext"
```

---

## Task 9: Update dock.rs to Use label_in_bounds()

**Files:**
- Modify: `crates/editor/src/dock.rs:232-268`

**Step 1: Read current implementation**

The current `render()` method uses:
```rust
let title_pos = Vec2::new(
    header_bounds.x + 8.0,
    header_bounds.center().y,
);
ui.label(&panel.title, title_pos);
```

**Step 2: Update to use label_in_bounds()**

In `crates/editor/src/dock.rs`, update the `render()` method.

First, add import at top of file:
```rust
use ui::TextAlign;
```

Then replace lines 257-261 (the title positioning code):

```rust
// OLD:
// let title_pos = Vec2::new(
//     header_bounds.x + 8.0,
//     header_bounds.center().y,
// );
// ui.label(&panel.title, title_pos);

// NEW:
ui.label_in_bounds(&panel.title, header_bounds, TextAlign::Left);
```

**Step 3: Run editor tests**

Run: `cargo test -p editor`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/editor/src/dock.rs
git commit -m "fix(editor): use label_in_bounds for panel titles"
```

---

## Task 10: Update dock.rs to Use Layout Constants

**Files:**
- Modify: `crates/editor/src/dock.rs`

**Step 1: Add import**

At top of `crates/editor/src/dock.rs`, add:
```rust
use crate::layout::{HEADER_HEIGHT, RESIZE_HANDLE_SIZE, MIN_PANEL_SIZE, DEFAULT_PANEL_WIDTH};
```

**Step 2: Replace magic numbers**

In `DockPanel::new()` (around line 73):
```rust
// OLD:
size: 250.0,
min_size: 100.0,

// NEW:
size: DEFAULT_PANEL_WIDTH,
min_size: MIN_PANEL_SIZE,
```

In `DockPanel::content_bounds()` (around line 106):
```rust
// OLD:
const HEADER_HEIGHT: f32 = 24.0;

// NEW: (remove the const, use imported HEADER_HEIGHT)
```

In `DockArea::new()` (around line 137):
```rust
// OLD:
header_height: 24.0,
resize_handle_size: 4.0,

// NEW:
header_height: HEADER_HEIGHT,
resize_handle_size: RESIZE_HANDLE_SIZE,
```

**Step 3: Run tests**

Run: `cargo test -p editor dock::tests`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/editor/src/dock.rs
git commit -m "refactor(editor): use layout constants in dock.rs"
```

---

## Task 11: Add Clip Rect to Panel Content Rendering

**Files:**
- Modify: `crates/editor/src/dock.rs:232-268`

**Step 1: Update render() to push clip rects**

In `crates/editor/src/dock.rs`, update the `render()` method to return info about clip state.

Update the method (the full updated version):

```rust
/// Render all panels.
///
/// Returns the content bounds for each visible panel. The caller should:
/// 1. Render content within each bounds
/// 2. Call `end_panel_content(ui)` after rendering each panel's content
pub fn render(&mut self, ui: &mut UIContext) -> Vec<(PanelId, Rect)> {
    let mut content_areas = Vec::new();

    for panel in &self.panels {
        if !panel.visible {
            continue;
        }

        // Draw panel background
        ui.panel(panel.bounds);

        // Draw panel header
        let header_bounds = Rect::new(
            panel.bounds.x,
            panel.bounds.y,
            panel.bounds.width,
            self.header_height,
        );
        ui.rect_rounded(header_bounds, Color::new(0.15, 0.15, 0.15, 1.0), 0.0);

        // Draw panel title - properly centered
        ui.label_in_bounds(&panel.title, header_bounds, TextAlign::Left);

        // Get content bounds and push clip rect
        let content = panel.content_bounds();
        ui.push_clip_rect(content);

        // Track content area (caller will render content, then pop clip)
        content_areas.push((panel.id, content));
    }

    content_areas
}

/// Call after rendering content for each panel to pop the clip rect.
pub fn end_panel_content(&self, ui: &mut UIContext, panel_count: usize) {
    for _ in 0..panel_count {
        ui.pop_clip_rect();
    }
}
```

**Step 2: Update editor_demo.rs to use end_panel_content**

This will be done in a separate task after testing the dock changes.

**Step 3: Run tests**

Run: `cargo test -p editor dock::tests`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/editor/src/dock.rs
git commit -m "feat(editor): add clip rects to panel content areas"
```

---

## Task 12: Add DPI Scale Factor to WindowManager

**Files:**
- Modify: `crates/engine_core/src/window_manager.rs`

**Step 1: Write the failing test**

Add to `crates/engine_core/src/window_manager.rs` in `mod tests`:

```rust
#[test]
fn test_window_manager_scale_factor() {
    let config = WindowConfig::default();
    let mut manager = WindowManager::new(config);

    assert_eq!(manager.scale_factor(), 1.0);

    manager.set_scale_factor(2.0);
    assert_eq!(manager.scale_factor(), 2.0);
}

#[test]
fn test_window_manager_logical_physical_size() {
    let config = WindowConfig::new("Test").with_size(800, 600);
    let mut manager = WindowManager::new(config);
    manager.set_scale_factor(2.0);

    assert_eq!(manager.logical_size(), (800.0, 600.0));
    assert_eq!(manager.physical_size(), (1600, 1200));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core window_manager::tests::test_window_manager_scale_factor`
Expected: FAIL with "no method named `scale_factor`"

**Step 3: Write minimal implementation**

Update `crates/engine_core/src/window_manager.rs`:

Add field to `WindowManager` struct:
```rust
pub struct WindowManager {
    window: Option<Arc<Window>>,
    config: WindowConfig,
    scale_factor: f64,  // NEW
}
```

Update `new()`:
```rust
pub fn new(config: WindowConfig) -> Self {
    Self {
        window: None,
        config,
        scale_factor: 1.0,
    }
}
```

Add methods after `size()`:
```rust
/// Get the current DPI scale factor.
pub fn scale_factor(&self) -> f64 {
    self.scale_factor
}

/// Update the scale factor (call on ScaleFactorChanged event).
pub fn set_scale_factor(&mut self, scale: f64) {
    self.scale_factor = scale;
}

/// Get logical size (for UI layout).
pub fn logical_size(&self) -> (f32, f32) {
    (self.config.width as f32, self.config.height as f32)
}

/// Get physical size (for wgpu surface).
pub fn physical_size(&self) -> (u32, u32) {
    (
        (self.config.width as f64 * self.scale_factor) as u32,
        (self.config.height as f64 * self.scale_factor) as u32,
    )
}
```

Update `create()` to sync scale factor after window creation (after line 109):
```rust
Ok(window) => {
    let window = Arc::new(window);
    self.scale_factor = window.scale_factor();  // NEW: sync scale factor
    self.window = Some(window.clone());
    log::info!("Window created: {} (scale: {})", self.config.title, self.scale_factor);
    Ok(window)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p engine_core window_manager::tests`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/engine_core/src/window_manager.rs
git commit -m "feat(engine_core): add DPI scale factor tracking to WindowManager"
```

---

## Task 13: Handle ScaleFactorChanged Event in Game Loop

**Files:**
- Modify: `crates/engine_core/src/game.rs` (event handling section)

**Step 1: Find the event handling code**

Search for `WindowEvent::Resized` in game.rs to find the event handling section.

**Step 2: Add ScaleFactorChanged handling**

In the event handler section of `crates/engine_core/src/game.rs`, find `WindowEvent::Resized` and add nearby:

```rust
WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
    self.window_manager.set_scale_factor(scale_factor);
    log::info!("Scale factor changed to: {}", scale_factor);
}
```

**Step 3: Update Resized handler to use logical/physical correctly**

The resize handler should store logical size but pass physical to renderer:

```rust
WindowEvent::Resized(physical_size) => {
    let scale = self.window_manager.scale_factor();
    // Store logical size for UI
    let logical_width = (physical_size.width as f64 / scale) as u32;
    let logical_height = (physical_size.height as f64 / scale) as u32;
    self.window_manager.resize(logical_width, logical_height);

    // Renderer needs physical size for surface
    if self.render_manager.is_initialized() {
        self.render_manager.resize(physical_size.width, physical_size.height);
    }
    log::debug!("Window resized to {}x{} logical ({}x{} physical)",
        logical_width, logical_height, physical_size.width, physical_size.height);
}
```

**Step 4: Run tests**

Run: `cargo test -p engine_core`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/engine_core/src/game.rs
git commit -m "feat(engine_core): handle ScaleFactorChanged event for DPI support"
```

---

## Task 14: Update ui_integration.rs for Scissor Rects

**Files:**
- Modify: `crates/engine_core/src/ui_integration.rs`

**Step 1: Update render_ui_commands signature and add scissor handling**

This is the most complex change. Update `crates/engine_core/src/ui_integration.rs`:

Add helper function at end of file:
```rust
/// Convert logical rect to physical pixels for scissor rect.
fn logical_to_physical_rect(rect: &Rect, scale_factor: f64) -> (u32, u32, u32, u32) {
    (
        (rect.x as f64 * scale_factor) as u32,
        (rect.y as f64 * scale_factor) as u32,
        (rect.width as f64 * scale_factor).max(1.0) as u32,
        (rect.height as f64 * scale_factor).max(1.0) as u32,
    )
}

/// Intersect two rects, returning the overlapping region.
fn intersect_rects(a: &Rect, b: &Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);

    Rect::new(
        x,
        y,
        (right - x).max(0.0),
        (bottom - y).max(0.0),
    )
}
```

Update `render_ui_commands` to handle clip commands (add handling in the match):
```rust
DrawCommand::PushClipRect { bounds } => {
    // Clip rect commands are handled at a higher level
    // Log for debugging during development
    log::trace!("PushClipRect: {:?}", bounds);
}
DrawCommand::PopClipRect => {
    log::trace!("PopClipRect");
}
```

Note: Full scissor rect implementation requires changes to how rendering is orchestrated (flushing batches, setting scissor on render pass). This is tracked as a follow-up enhancement.

**Step 2: Run tests**

Run: `cargo test -p engine_core`
Expected: All pass

**Step 3: Commit**

```bash
git add crates/engine_core/src/ui_integration.rs
git commit -m "feat(engine_core): add clip rect command handling (foundation)"
```

---

## Task 15: Visual Testing and Polish

**Files:**
- Modify: `examples/editor_demo.rs` (update to use new APIs)

**Step 1: Update editor_demo.rs**

Update the panel content rendering to pop clip rects:

In `render_panel_content` or after the content rendering loop, ensure clip rects are popped.

Since `dock.render()` now pushes clip rects, we need to pop them after rendering content:

```rust
// In EditorDemo::update(), after render_panel_content loop:
for (panel_id, bounds) in content_areas.clone() {
    self.render_panel_content(ctx, panel_id, bounds);
}

// Pop all clip rects (one per panel rendered)
self.editor.dock_area.end_panel_content(ctx.ui, content_areas.len());
```

**Step 2: Run the editor demo**

Run: `cargo run --example editor_demo`
Expected:
- Text in panel headers should be vertically centered
- Text should not bleed outside panel boundaries

**Step 3: Commit**

```bash
git add examples/editor_demo.rs
git commit -m "fix(demo): update editor_demo to use clip rects"
```

---

## Verification Checklist

After completing all tasks, run:

```bash
# All tests pass
cargo test --workspace

# Editor demo runs without crashes
cargo run --example editor_demo

# Check for any new warnings
cargo clippy --workspace
```

## Summary of Changes

| File | Change |
|------|--------|
| `crates/ui/src/font.rs` | Add `FontMetrics` struct and `metrics()` method |
| `crates/ui/src/draw.rs` | Add `PushClipRect`/`PopClipRect` commands |
| `crates/ui/src/context.rs` | Add `TextAlign`, `font_metrics()`, `label_in_bounds()`, clip methods |
| `crates/ui/src/lib.rs` | Export new types |
| `crates/editor/src/layout.rs` | New file with layout constants |
| `crates/editor/src/dock.rs` | Use `label_in_bounds()`, layout constants, clip rects |
| `crates/engine_core/src/window_manager.rs` | Add scale factor tracking |
| `crates/engine_core/src/game.rs` | Handle `ScaleFactorChanged` event |
| `crates/engine_core/src/ui_integration.rs` | Foundation for clip rect handling |
| `examples/editor_demo.rs` | Update to use new APIs |
