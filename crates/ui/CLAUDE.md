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
- `context/` — UIContext: `mod.rs` (struct, lifecycle incl. `begin_frame_dt`, fonts, primitives incl. `image`/`rect_border`), `text.rs` (label/measure), `widgets.rs` (button, slider, checkbox), `text_input.rs` (float_input: select-all-on-focus, cursor, selection, arrows/Home/End, key repeat), `tests.rs`
- `font/` — `mod.rs` (FontManager facade: loading/storage), `glyph_cache.rs` (GlyphCache; bitmaps shared via `Arc<[u8]>`), `layout.rs` (text layout/measurement)
- `draw.rs` — Draw command generation (`Rect` re-exported from `common`)
- `interaction.rs` — Widget state, mouse hit detection, focus, per-widget persistent state (`edit: TextEditState`)
- `input_state.rs` — per-frame `InputState` snapshot + `KeyRepeat` (dt-driven hold repeat)
- `text_edit.rs` — pure `TextEditState` (buffer/cursor/selection editing model)
- `style.rs` — Theme definitions (`Color` re-exported from `common`), private palette consts

## Known Tech Debt
- See `TECH_DEBT.md` — open: JUN-T1 narrowed (cursor/selection/repeat DONE Jul 2026; still numeric-only by design); Low: TextDrawData redundancy (ARCH-003), unused scroll_delta (JUN-T2), no layout helpers (JUN-T3)

## Testing
- 102 tests (incl. 2 doc), run with `cargo test -p ui`

## Godot Oracle
- Immediate-mode patterns: Godot doesn't use immediate-mode, but see `scene/gui/control.cpp` for widget lifecycle
- Font rendering: `modules/text_server_advanced/text_server_adv.cpp`
