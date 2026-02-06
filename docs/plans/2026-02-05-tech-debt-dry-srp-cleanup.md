# Tech Debt DRY/SRP Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Clean up medium-effort DRY/SRP violations across engine_core, editor, ui, and input crates.

**Architecture:** Extract repeated patterns into focused helper functions/methods, following existing crate conventions. Each task is a self-contained refactor within a single file, with no cross-crate dependencies between tasks.

**Tech Stack:** Rust, existing crate APIs (no new dependencies)

---

## Task 1: Extract `next_depth()` helper in ui DrawList

**Files:**
- Modify: `crates/ui/src/draw.rs` (lines 162-249)

**Step 1: Add the helper method to DrawList impl**

Add this method to the `impl DrawList` block:

```rust
/// Calculate the depth for the next draw command.
/// Each command gets slightly increasing depth to maintain draw order.
#[inline]
fn next_depth(&self) -> f32 {
    self.base_depth + self.commands.len() as f32 * 0.001
}
```

**Step 2: Replace all 8 occurrences**

Replace every instance of `self.base_depth + self.commands.len() as f32 * 0.001` with `self.next_depth()` in methods: `rect_rounded()`, `rect_border_rounded()`, `text_placeholder()`, `text()`, `text_simple()`, `circle()`, `line()`.

**Step 3: Run tests**

Run: `cargo test -p ui`
Expected: All tests pass (no behavior change)

**Step 4: Commit**

```
refactor(ui): extract next_depth() helper in DrawList
```

---

## Task 2: Extract `unbind_input_internal()` in input mapping

**Files:**
- Modify: `crates/input/src/input_mapping.rs` (lines 145-205)

**Step 1: Add the helper method**

Add this private method to `InputMapping`:

```rust
/// Remove an existing binding for the given input source, cleaning up action_bindings.
fn remove_existing_binding(&mut self, input: &InputSource) {
    if let Some(old_action) = self.bindings.remove(input) {
        if let Some(sources) = self.action_bindings.get_mut(&old_action) {
            sources.retain(|&s| s != *input);
            if sources.is_empty() {
                self.action_bindings.remove(&old_action);
            }
        }
    }
}
```

**Step 2: Use in `bind_input()`**

Replace the unbind block in `bind_input()` with:
```rust
self.remove_existing_binding(&input);
```

**Step 3: Use in `bind_input_to_multiple_actions()`**

Replace the unbind block in `bind_input_to_multiple_actions()` with:
```rust
self.remove_existing_binding(&input);
```

Note: This also fixes the inconsistency where `bind_input_to_multiple_actions` didn't clean up empty action entries.

**Step 4: Run tests**

Run: `cargo test -p input`
Expected: All tests pass

**Step 5: Commit**

```
refactor(input): extract remove_existing_binding() helper in InputMapping
```

---

## Task 3: Extract `has_keys()` helper for vec-like detection in editor inspector

**Files:**
- Modify: `crates/editor/src/inspector.rs` (lines 177-202)

**Step 1: Add the helper function**

Add above `is_vec_like`:

```rust
/// Check if a JSON map has exactly the given keys (no more, no less).
fn has_exact_keys(map: &serde_json::Map<String, Value>, keys: &[&str]) -> bool {
    map.len() == keys.len() && keys.iter().all(|k| map.contains_key(*k))
}
```

**Step 2: Simplify `is_vec_like()`**

Replace the three if-statements with:

```rust
fn is_vec_like(map: &serde_json::Map<String, Value>) -> bool {
    has_exact_keys(map, &["x", "y"])
        || has_exact_keys(map, &["x", "y", "z"])
        || has_exact_keys(map, &["x", "y", "z", "w"])
}
```

**Step 3: Run tests**

Run: `cargo test -p editor`
Expected: All tests pass

**Step 4: Commit**

```
refactor(editor): extract has_exact_keys() helper for vec detection in inspector
```

---

## Task 4: Extract `check_action_with()` in editor input

**Files:**
- Modify: `crates/editor/src/editor_input.rs` (lines 201-259)

**Step 1: Add the generic helper method**

Add this private method to `EditorInputMapping`:

```rust
/// Check if any binding for an action satisfies the given predicates.
fn check_action_with(
    &self,
    action: EditorAction,
    input: &InputHandler,
    key_check: impl Fn(&InputHandler, KeyCode) -> bool,
    mouse_check: impl Fn(&InputHandler, MouseButton) -> bool,
) -> bool {
    self.get_bindings(action).iter().any(|source| match source {
        InputSource::Keyboard(key) => key_check(input, *key),
        InputSource::Mouse(button) => mouse_check(input, *button),
        InputSource::Gamepad(_, _) => false,
    })
}
```

**Step 2: Simplify the three methods**

Replace `is_action_pressed`, `is_action_just_pressed`, `is_action_just_released` with:

```rust
pub fn is_action_pressed(&self, action: EditorAction, input: &InputHandler) -> bool {
    self.check_action_with(
        action, input,
        |i, key| i.is_key_pressed(key),
        |i, btn| i.is_mouse_button_pressed(btn),
    )
}

pub fn is_action_just_pressed(&self, action: EditorAction, input: &InputHandler) -> bool {
    self.check_action_with(
        action, input,
        |i, key| i.is_key_just_pressed(key),
        |i, btn| i.is_mouse_button_just_pressed(btn),
    )
}

pub fn is_action_just_released(&self, action: EditorAction, input: &InputHandler) -> bool {
    self.check_action_with(
        action, input,
        |i, key| i.is_key_just_released(key),
        |i, _btn| false, // Mouse button release not yet supported
    )
}
```

**Step 3: Run tests**

Run: `cargo test -p editor`
Expected: All tests pass

**Step 4: Commit**

```
refactor(editor): extract check_action_with() to deduplicate input checking
```

---

## Task 5: Extract `centered_rect()` in editor gizmo

**Files:**
- Modify: `crates/editor/src/gizmo.rs` (lines 206-244, 399-404)

**Step 1: Add the helper method to Gizmo**

```rust
/// Create a square rect centered at the given position, sized to handle_size.
fn centered_handle_rect(&self, center: Vec2) -> Rect {
    Rect::new(
        center.x - self.handle_size / 2.0,
        center.y - self.handle_size / 2.0,
        self.handle_size,
        self.handle_size,
    )
}
```

**Step 2: Replace all occurrences**

In `render_translate()`: Replace x_arrow_bounds, y_arrow_bounds, and center_bounds creation.
In `render_scale()`: Replace handle_bounds creation in the corners loop.

Each `Rect::new(pos.x - self.handle_size / 2.0, pos.y - self.handle_size / 2.0, self.handle_size, self.handle_size)` becomes `self.centered_handle_rect(pos)`.

**Step 3: Run tests**

Run: `cargo test -p editor`
Expected: All tests pass

**Step 4: Commit**

```
refactor(editor): extract centered_handle_rect() to deduplicate gizmo bounds
```

---

## Task 6: Extract `screen_to_world` helpers in engine_core ui_integration

**Files:**
- Modify: `crates/engine_core/src/ui_integration.rs` (lines 38-210)

**Step 1: Add two helper functions at the top of the file**

```rust
/// Convert a screen-space rect center to world coordinates.
/// Screen: (0,0) = top-left. World: (0,0) = center.
fn screen_rect_center_to_world(bounds: &Rect, window_size: Vec2) -> Vec2 {
    Vec2::new(
        bounds.x + bounds.width / 2.0 - window_size.x / 2.0,
        window_size.y / 2.0 - (bounds.y + bounds.height / 2.0),
    )
}

/// Convert a screen-space point to world coordinates.
fn screen_point_to_world(x: f32, y: f32, window_size: Vec2) -> Vec2 {
    Vec2::new(
        x - window_size.x / 2.0,
        window_size.y / 2.0 - y,
    )
}
```

**Step 2: Replace all coordinate transformation instances**

Replace the 7 instances of inline coordinate math:
- `DrawCommand::Rect` handler → use `screen_rect_center_to_world`
- Text placeholder handler → use `screen_rect_center_to_world` variant
- Circle handler → use `screen_point_to_world`
- Line handler → use `screen_point_to_world` for midpoint
- `render_ui_rect` helper → use `screen_rect_center_to_world`
- Glyph positioning → use `screen_point_to_world` variant with offset
- TextPlaceholder handler → compute bounds, then use helper

**Step 3: Run tests**

Run: `cargo test -p engine_core`
Expected: All tests pass

**Step 4: Commit**

```
refactor(engine_core): extract screen-to-world coordinate helpers in ui_integration
```

---

## Task 7: Extract `handle_render_error()` in engine_core render_manager

**Files:**
- Modify: `crates/engine_core/src/render_manager.rs` (lines 117-181)

**Step 1: Add the helper method to RenderManager**

```rust
/// Handle a render error, attempting surface recreation on surface loss.
fn handle_render_error(renderer: &mut Renderer, error: RendererError) -> Result<(), RendererError> {
    match error {
        RendererError::SurfaceError(_) => {
            if let Err(e) = renderer.recreate_surface() {
                log::error!("Failed to recreate surface: {}", e);
                return Err(e);
            }
            log::debug!("Surface recreated after loss");
            Ok(())
        }
        e => {
            log::error!("Render error: {}", e);
            Err(e)
        }
    }
}
```

**Step 2: Use in `render()` and `render_basic()`**

Replace both match blocks with:
```rust
match renderer.render_with_sprites(...) {
    Ok(_) => Ok(()),
    Err(e) => Self::handle_render_error(renderer, e),
}
```

```rust
match renderer.render() {
    Ok(_) => Ok(()),
    Err(e) => Self::handle_render_error(renderer, e),
}
```

**Step 3: Run tests**

Run: `cargo test -p engine_core`
Expected: All tests pass

**Step 4: Commit**

```
refactor(engine_core): extract handle_render_error() in RenderManager
```

---

## Task 8: Extract `parse_hex_byte()` in engine_core scene_loader

**Files:**
- Modify: `crates/engine_core/src/scene_loader.rs` (lines 528-545)

**Step 1: Add the helper function**

Add to the impl block or as a free function:

```rust
/// Parse a 2-character hex byte from a string at the given offset.
fn parse_hex_byte(hex: &str, start: usize) -> Result<u8, SceneLoadError> {
    u8::from_str_radix(&hex[start..start + 2], 16)
        .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))
}
```

**Step 2: Simplify `parse_hex_color()`**

Replace the repeated `u8::from_str_radix` calls:

```rust
if hex.len() == 6 {
    Ok([parse_hex_byte(hex, 0)?, parse_hex_byte(hex, 2)?, parse_hex_byte(hex, 4)?, 255])
} else if hex.len() == 8 {
    Ok([parse_hex_byte(hex, 0)?, parse_hex_byte(hex, 2)?, parse_hex_byte(hex, 4)?, parse_hex_byte(hex, 6)?])
}
```

**Step 3: Run tests**

Run: `cargo test -p engine_core`
Expected: All tests pass

**Step 4: Commit**

```
refactor(engine_core): extract parse_hex_byte() to deduplicate hex color parsing
```

---

## Task 9: Extract menu layout constants in editor

**Files:**
- Modify: `crates/editor/src/menu.rs` (lines 305-378)

**Step 1: Move constants to module level**

Replace the local constants inside `render_dropdown_static` with module-level constants:

```rust
/// Menu dropdown layout constants
const DROPDOWN_ITEM_HEIGHT: f32 = 24.0;
const DROPDOWN_ITEM_PADDING: f32 = 8.0;
const DROPDOWN_WIDTH: f32 = 200.0;
```

**Step 2: Update `render_dropdown_static` to use module constants**

Remove the three `const` lines from inside the function and reference the module-level constants instead.

**Step 3: Run tests**

Run: `cargo test -p editor`
Expected: All tests pass

**Step 4: Commit**

```
refactor(editor): promote menu dropdown constants to module level
```

---

## Task 10: Extract `calculate_baseline_y()` in ui context

**Files:**
- Modify: `crates/ui/src/context.rs` (lines 252-265, 362-366)

**Step 1: Add the helper method to UIContext**

```rust
/// Calculate the baseline Y position for vertically centered text.
fn baseline_y(&self, text_top: f32, font_size: f32, font_handle: FontHandle) -> f32 {
    let ascent = self.font_manager.metrics(font_handle, font_size)
        .map(|m| m.ascent)
        .unwrap_or(font_size * 0.8);
    text_top + ascent
}
```

**Step 2: Use in `button_styled()`**

Replace the ascent/baseline calculation with:
```rust
let baseline_y = self.baseline_y(text_top, font_size, font_handle);
```

**Step 3: Use in `label_in_bounds()`**

Replace the ascent/baseline calculation with the same helper call.

**Step 4: Run tests**

Run: `cargo test -p ui`
Expected: All tests pass

**Step 5: Commit**

```
refactor(ui): extract baseline_y() helper to deduplicate text positioning
```

---

## Task 11: Fix `measure_text()` cache bypass in ui font

**Files:**
- Modify: `crates/ui/src/font.rs` (lines 332-352)
- Modify: `crates/ui/src/context.rs` (callers of measure_text)

**Step 1: Change `measure_text` signature from `&self` to `&mut self`**

**Step 2: Replace direct rasterize with cached version**

Change:
```rust
let (metrics, _) = font.rasterize(character, font_size);
width += metrics.advance_width;
```

To use `rasterize_glyph()` which populates the cache. Note: since `rasterize_glyph` takes `&mut self`, this requires the signature change.

If `rasterize_glyph` returns a type with an `advance` field, use that. Otherwise, we may need to use `font.metrics()` instead of full rasterize - check the fontdue API for a metrics-only call that doesn't rasterize.

**Step 3: Update callers**

Any callers of `measure_text` that pass `&self` need to be updated to `&mut self`. Check `context.rs` call sites.

**Step 4: Run tests**

Run: `cargo test -p ui`
Expected: All tests pass

**Step 5: Commit**

```
perf(ui): use glyph cache in measure_text() to avoid redundant rasterization
```

---

## Task 12: Run full workspace tests and update TECH_DEBT.md files

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass (338+ passing)

**Step 2: Update TECH_DEBT.md files**

Mark resolved items in each crate's TECH_DEBT.md:
- `crates/ui/TECH_DEBT.md`: Mark DRY-005, DRY-006, PERF-001 as resolved
- `crates/input/TECH_DEBT.md`: Mark DRY-003 as resolved
- `crates/editor/TECH_DEBT.md`: Mark DRY-001, DRY-002/SRP-001, DRY-003, DRY-004 as resolved
- `crates/engine_core/TECH_DEBT.md`: Mark DRY-002, DRY-004, DRY-005 as resolved
- `crates/ui/TECH_DEBT.md`: Mark DRY-004 as resolved (render_text_with_fallback not done but baseline_y is)

**Step 3: Commit**

```
docs: update TECH_DEBT.md files with resolved items from cleanup
```
