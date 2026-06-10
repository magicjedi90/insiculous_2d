# Input Crate — Agent Context

Event-based input system with a generic action-mapping layer.

## Key Types
- `InputHandler` — raw device state: keyboard, mouse, gamepads + event queue
- `InputMapping<A>` — generic action bindings; games define their own action enums:
  `mapping.bind(MyAction::Jump, InputSource::Keyboard(KeyCode::Space))`
- `GameAction` — optional engine preset enum; `InputMapping::with_default_bindings()`
  gives WASD/arrows + Space/Enter + gamepad-0 defaults (used by `BehaviorRunner`)
- `ButtonTracker<T>` — shared pressed/just_pressed/just_released tracker composed
  by `KeyboardState`, `MouseState`, `GamepadState`

## Frame Lifecycle
1. `handle_window_event()` queues events (engine does this automatically)
2. `process_queued_events()` at start of frame
3. Game logic reads state
4. `end_frame()` clears one-shot flags + mouse movement/wheel deltas

## Per-Frame Queries
- `is_key_pressed(key)` / `is_key_just_pressed(key)` / `is_key_just_released(key)`
- `is_source_pressed(&source)` family — works for any `InputSource`
- `mapping.is_active(action, &input)` — any bound source held
- `mapping.just_activated(action, &input)` — strict edge: was inactive last frame
- `mapping.just_deactivated(action, &input)` — strict edge: no source still held
- `mouse_movement_delta()` — accumulated over the frame, zero when mouse idle

## Design Notes
- `InputMapping::new()` is **empty** — no implicit default bindings
- Gamepads auto-register on first event; there is **no gamepad backend yet**
  (no gilrs) so gamepad events only come from manual `queue_event()` calls
- Winit types (`KeyCode`, `MouseButton`) used directly by design (documented
  on `InputEvent`)
- Scroll deltas normalized to lines (`PixelDelta` ÷ 16)

## Testing
- 58 tests — `cargo test -p input`
