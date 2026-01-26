# UI Overhaul Design

**Date:** 2026-01-26
**Status:** Draft
**Author:** Claude + Human collaboration

## Problem Statement

The editor UI has several rendering issues:
1. **Text positioning bug**: Text renders below its containers (buttons show blank, text appears underneath)
2. **No clipping**: Content bleeds across panel boundaries
3. **No DPI scaling**: Will break on HiDPI displays (Mac Retina, scaled Linux)
4. **Inconsistent spacing**: Magic numbers scattered throughout code

## Design

### 1. Baseline Text Positioning

**Change:** Text `position.y` represents the baseline (where letters sit), not top-left.

#### Font Metrics API

```rust
// crates/ui/src/font.rs

/// Font measurement information for layout calculations
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Distance from baseline to top of tallest glyph (positive)
    pub ascent: f32,
    /// Distance from baseline to bottom of descenders (negative)
    pub descent: f32,
    /// Recommended distance between baselines
    pub line_height: f32,
}

impl FontManager {
    /// Get font metrics for layout calculations
    pub fn metrics(&self, font: FontHandle, font_size: f32) -> Option<FontMetrics> {
        let font = self.fonts.get(&font)?;
        let scale = font.horizontal_line_metrics(font_size)?;
        Some(FontMetrics {
            ascent: scale.ascent,
            descent: scale.descent,  // Usually negative
            line_height: scale.ascent - scale.descent + scale.line_gap,
        })
    }
}
```

#### Glyph Layout Changes

```rust
// In layout_text(), glyph Y positioning changes:
// OLD: y offset from top-left of text bounds
// NEW: y offset from baseline

for character in text.chars() {
    let (metrics, bitmap) = font.rasterize(character, font_size);

    // Baseline-relative positioning
    // metrics.ymin is distance from baseline to bottom of glyph (usually negative)
    // So glyph top = baseline - (glyph_height + ymin)
    let glyph_y = -(metrics.height as f32) - metrics.ymin;

    glyphs.push(LayoutGlyph {
        x: cursor_x + metrics.xmin as f32,
        y: glyph_y,
        // ...
    });
}
```

#### UI Context Updates

```rust
// crates/ui/src/context.rs

/// Text alignment within bounds
#[derive(Debug, Clone, Copy, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl UIContext {
    /// Get font metrics for the default font
    pub fn font_metrics(&self, font_size: f32) -> Option<FontMetrics> {
        let font = self.font_manager.default_font()?;
        self.font_manager.metrics(font, font_size)
    }

    /// Draw label centered within bounds (most common use case)
    pub fn label_in_bounds(&mut self, text: &str, bounds: Rect, align: TextAlign) {
        let font_size = self.theme.text.font_size;
        let color = self.theme.text.color;

        if let Some(metrics) = self.font_metrics(font_size) {
            // Calculate baseline for vertical centering
            // Visual center considers ascent (above baseline) and descent (below)
            let baseline_y = bounds.y + (bounds.height + metrics.ascent + metrics.descent) / 2.0;

            // Calculate X based on alignment
            let x = match align {
                TextAlign::Left => bounds.x + PADDING,
                TextAlign::Center => {
                    if let Ok(size) = self.measure_text(text, font_size) {
                        bounds.x + (bounds.width - size.x) / 2.0
                    } else {
                        bounds.x + PADDING
                    }
                }
                TextAlign::Right => {
                    if let Ok(size) = self.measure_text(text, font_size) {
                        bounds.x + bounds.width - size.x - PADDING
                    } else {
                        bounds.x + PADDING
                    }
                }
            };

            self.label(text, Vec2::new(x, baseline_y));
        } else {
            // Fallback: approximate center
            self.label(text, bounds.center());
        }
    }
}
```

#### Button Text Fix

```rust
// In button_styled(), replace the fudge factor with proper centering:

let font_size = self.theme.text.font_size;
if let Some(font_handle) = self.font_manager.default_font() {
    if let Some(metrics) = self.font_manager.metrics(font_handle, font_size) {
        if let Ok(text_width) = self.font_manager.measure_text(font_handle, label, font_size) {
            let baseline_y = bounds.y + (bounds.height + metrics.ascent + metrics.descent) / 2.0;
            let text_x = bounds.x + (bounds.width - text_width.x) / 2.0;
            let text_pos = Vec2::new(text_x, baseline_y);
            // ... render text at text_pos
        }
    }
}
```

### 2. Scissor Rect Clipping

#### Draw Commands

```rust
// crates/ui/src/draw.rs

pub enum DrawCommand {
    // ... existing variants ...

    /// Begin clipping to a rectangular region
    PushClipRect { bounds: Rect },

    /// End clipping, restore previous clip region
    PopClipRect,
}

impl DrawList {
    pub fn push_clip_rect(&mut self, bounds: Rect) {
        self.commands.push(DrawCommand::PushClipRect { bounds });
    }

    pub fn pop_clip_rect(&mut self) {
        self.commands.push(DrawCommand::PopClipRect);
    }
}
```

#### UI Context API

```rust
// crates/ui/src/context.rs

impl UIContext {
    /// Begin a clipped region. All subsequent draws are clipped to bounds.
    /// Must be paired with pop_clip_rect().
    pub fn push_clip_rect(&mut self, bounds: Rect) {
        self.draw_list.push_clip_rect(bounds);
    }

    /// End the current clipped region, restore previous clip.
    pub fn pop_clip_rect(&mut self) {
        self.draw_list.pop_clip_rect();
    }
}
```

#### Renderer Integration

```rust
// crates/engine_core/src/ui_integration.rs

pub fn render_ui_commands(
    sprites: &mut SpriteBatcher,
    commands: &[DrawCommand],
    window_size: Vec2,
    scale_factor: f64,  // NEW: for scissor rect conversion
    glyph_textures: &HashMap<GlyphCacheKey, TextureHandle>,
    render_pass: &mut wgpu::RenderPass,  // NEW: needed for scissor
) {
    let mut clip_stack: Vec<Rect> = Vec::new();

    for cmd in commands {
        match cmd {
            DrawCommand::PushClipRect { bounds } => {
                // Flush current batch before changing scissor
                sprites.flush(render_pass);

                // Convert logical to physical pixels
                let physical_bounds = logical_to_physical(*bounds, scale_factor);

                // Intersect with current clip (if any) for nested clips
                let effective_bounds = if let Some(current) = clip_stack.last() {
                    intersect_rects(current, &physical_bounds)
                } else {
                    physical_bounds
                };

                clip_stack.push(effective_bounds);

                render_pass.set_scissor_rect(
                    effective_bounds.x as u32,
                    effective_bounds.y as u32,
                    effective_bounds.width as u32,
                    effective_bounds.height as u32,
                );
            }

            DrawCommand::PopClipRect => {
                sprites.flush(render_pass);
                clip_stack.pop();

                if let Some(bounds) = clip_stack.last() {
                    render_pass.set_scissor_rect(
                        bounds.x as u32,
                        bounds.y as u32,
                        bounds.width as u32,
                        bounds.height as u32,
                    );
                } else {
                    // Reset to full viewport
                    let (w, h) = physical_window_size(window_size, scale_factor);
                    render_pass.set_scissor_rect(0, 0, w, h);
                }
            }

            // ... handle other commands as before ...
        }
    }
}

fn logical_to_physical(rect: Rect, scale_factor: f64) -> Rect {
    Rect::new(
        (rect.x as f64 * scale_factor) as f32,
        (rect.y as f64 * scale_factor) as f32,
        (rect.width as f64 * scale_factor) as f32,
        (rect.height as f64 * scale_factor) as f32,
    )
}
```

### 3. DPI Scaling

#### Window Manager Changes

```rust
// crates/engine_core/src/window_manager.rs

pub struct WindowManager {
    window: Option<Arc<Window>>,
    config: WindowConfig,
    scale_factor: f64,  // NEW
}

impl WindowManager {
    pub fn new(config: WindowConfig) -> Self {
        Self {
            window: None,
            config,
            scale_factor: 1.0,
        }
    }

    /// Get logical size (for UI layout)
    pub fn logical_size(&self) -> (f32, f32) {
        (self.config.width as f32, self.config.height as f32)
    }

    /// Get physical size (for wgpu surface)
    pub fn physical_size(&self) -> (u32, u32) {
        (
            (self.config.width as f64 * self.scale_factor) as u32,
            (self.config.height as f64 * self.scale_factor) as u32,
        )
    }

    /// Get current scale factor
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Update scale factor (call on ScaleFactorChanged event)
    pub fn set_scale_factor(&mut self, scale: f64) {
        self.scale_factor = scale;
    }

    /// Update after window creation to get actual scale factor
    pub fn sync_scale_factor(&mut self) {
        if let Some(window) = &self.window {
            self.scale_factor = window.scale_factor();
        }
    }
}
```

#### Event Handling

```rust
// In game.rs event handling

WindowEvent::Resized(physical_size) => {
    // Convert physical back to logical for our tracking
    let scale = self.window_manager.scale_factor();
    let logical_width = (physical_size.width as f64 / scale) as u32;
    let logical_height = (physical_size.height as f64 / scale) as u32;
    self.window_manager.resize(logical_width, logical_height);

    // Renderer needs physical size for surface
    self.render_manager.resize(physical_size.width, physical_size.height);
}

WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
    self.window_manager.set_scale_factor(scale_factor);
    // Surface will be reconfigured on next Resized event
}
```

#### Size Usage Table

| System | Size Type | Source |
|--------|-----------|--------|
| UI layout, bounds, positions | Logical | `window_manager.logical_size()` |
| Camera projection | Logical | Same as UI |
| wgpu Surface configuration | Physical | `window_manager.physical_size()` |
| Scissor rects | Physical | Convert at render time |
| Mouse input | Logical | winit provides logical by default |

### 4. Layout Constants

```rust
// crates/editor/src/layout.rs (NEW FILE)

//! Editor layout constants for consistent spacing.

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

/// Minimum panel size
pub const MIN_PANEL_SIZE: f32 = 100.0;

/// Default panel width (for left/right docked panels)
pub const DEFAULT_PANEL_WIDTH: f32 = 250.0;
```

### 5. Editor Code Updates

#### dock.rs

```rust
use crate::layout::*;

// In render():
pub fn render(&mut self, ui: &mut UIContext) -> Vec<(PanelId, Rect)> {
    for panel in &self.panels {
        if !panel.visible {
            continue;
        }

        // Draw panel background
        ui.panel(panel.bounds);

        // Draw header
        let header_bounds = Rect::new(
            panel.bounds.x,
            panel.bounds.y,
            panel.bounds.width,
            HEADER_HEIGHT,
        );
        ui.rect_rounded(header_bounds, Color::new(0.15, 0.15, 0.15, 1.0), 0.0);

        // Draw title - properly centered in header
        ui.label_in_bounds(&panel.title, header_bounds, TextAlign::Left);

        // Clip content area
        let content = panel.content_bounds();
        ui.push_clip_rect(content);
        content_areas.push((panel.id, content));
    }

    content_areas
}

// Caller must pop clip rects after rendering content
```

#### menu.rs

```rust
use crate::layout::*;

// Menu items use label_in_bounds for proper centering:
ui.label_in_bounds(&item.label, item_bounds, TextAlign::Left);

// Shortcuts right-aligned:
if let Some(shortcut) = &item.shortcut {
    ui.label_in_bounds(shortcut, item_bounds, TextAlign::Right);
}
```

#### inspector.rs

```rust
use crate::layout::*;

// Use font metrics for line spacing:
let line_height = ui.font_metrics(font_size)
    .map(|m| m.line_height)
    .unwrap_or(LINE_HEIGHT);

// Labels still use direct positioning (baseline Y):
let baseline_y = current_y + line_height * 0.8;  // Approximate baseline within line
ui.label(&format!("{}: {}", key, value), Vec2::new(x, baseline_y));
current_y += line_height;
```

## Implementation Order

1. **Font metrics** - Foundation everything else builds on
2. **Baseline positioning in label()** - Core semantic change
3. **label_in_bounds() helper** - Makes editor updates easy
4. **Button text fix** - Uses new font metrics
5. **Layout constants** - Quick refactor
6. **Editor code updates** - Apply new APIs
7. **Scissor rect commands** - Draw command additions
8. **Scissor rect rendering** - Renderer integration
9. **DPI scaling** - Window manager + event handling
10. **Testing on HiDPI** - Verify scaling works

## Testing Strategy

- **Unit tests**: Font metrics calculations, rect intersection for nested clips
- **Visual tests**: Run editor_demo, verify text alignment in buttons/panels
- **DPI testing**: Use `GDK_SCALE=2` on Linux to simulate HiDPI
- **Regression**: Existing UI tests should still pass

## Migration Notes

All existing `label()` calls need Y coordinate adjustment:
- **Old**: Y = top of text
- **New**: Y = baseline

For code that was doing `bounds.y + some_offset`, the offset calculation changes.
For code using `label_in_bounds()`, no manual calculation needed.

## Risks

1. **Breaking change**: All `label()` positions shift. Mitigated by providing `label_in_bounds()`.
2. **Scissor performance**: Flushing batches on clip changes. Mitigated by batching draws within clip regions.
3. **Font metric availability**: fontdue provides metrics, but need fallbacks for missing fonts.
