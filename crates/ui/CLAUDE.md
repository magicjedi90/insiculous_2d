# UI Crate — Agent Context

Immediate-mode UI framework with fontdue text rendering.

## Pattern
```rust
ui.begin_frame(&input, window_size);
ui.panel(rect);
ui.label("text", pos);
if ui.button("id", "label", rect) { /* clicked */ }
let val = ui.slider("id", current, rect);
ui.end_frame(); // collects draw commands
```

## File Map
- `context.rs` — UIContext, widget methods (button, slider, label, panel, text_input, float_input)
- `font.rs` — FontManager (loading, rasterization, glyph cache, layout)
- `draw.rs` — Draw command generation
- `interaction.rs` — Widget state, mouse hit detection
- `rect.rs` — UIRect layout utilities
- `style.rs` — Color, Theme definitions

## Known Tech Debt
- FontManager has too many responsibilities (load + raster + cache + layout)

## Testing
- 60 tests, run with `cargo test -p ui`

## Godot Oracle
- Immediate-mode patterns: Godot doesn't use immediate-mode, but see `scene/gui/control.cpp` for widget lifecycle
- Font rendering: `modules/text_server_advanced/text_server_adv.cpp`
