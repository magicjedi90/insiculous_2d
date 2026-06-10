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
- `context.rs` — UIContext, widget methods (button, slider, label, panel, checkbox, float_input)
- `font.rs` — FontManager (loading, rasterization, glyph cache, layout); bitmaps shared via `Arc<[u8]>`
- `draw.rs` — Draw command generation (`Rect` re-exported from `common`)
- `interaction.rs` — Widget state, mouse hit detection, focus, per-widget persistent state
- `style.rs` — Theme definitions (`Color` re-exported from `common`), private palette consts

## Known Tech Debt
- See `TECH_DEBT.md` — open: FontManager responsibilities (SRP-001), context.rs over 600-line rule (SRP-002), numeric-only text input (JUN-T1)

## Testing
- 68 tests, run with `cargo test -p ui`

## Godot Oracle
- Immediate-mode patterns: Godot doesn't use immediate-mode, but see `scene/gui/control.cpp` for widget lifecycle
- Font rendering: `modules/text_server_advanced/text_server_adv.cpp`
