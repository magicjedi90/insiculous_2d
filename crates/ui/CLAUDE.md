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
- `context/` — UIContext: `mod.rs` (struct, lifecycle, fonts, primitives), `text.rs` (label/measure), `widgets.rs` (button, slider, checkbox, float_input), `tests.rs`
- `font/` — `mod.rs` (FontManager facade: loading/storage), `glyph_cache.rs` (GlyphCache; bitmaps shared via `Arc<[u8]>`), `layout.rs` (text layout/measurement)
- `draw.rs` — Draw command generation (`Rect` re-exported from `common`)
- `interaction.rs` — Widget state, mouse hit detection, focus, per-widget persistent state
- `style.rs` — Theme definitions (`Color` re-exported from `common`), private palette consts

## Known Tech Debt
- See `TECH_DEBT.md` — open: numeric-only text input (JUN-T1, Medium); Low: TextDrawData redundancy (ARCH-003), unused scroll_delta (JUN-T2), no layout helpers (JUN-T3)

## Testing
- 80 tests (incl. 2 doc), run with `cargo test -p ui`

## Godot Oracle
- Immediate-mode patterns: Godot doesn't use immediate-mode, but see `scene/gui/control.cpp` for widget lifecycle
- Font rendering: `modules/text_server_advanced/text_server_adv.cpp`
