# Input Crate — Agent Context

Event-based input system with action mapping. Thread-safe via Arc<Mutex<>>.

## Key Types
- `InputHandler` — main state: keyboard, mouse, gamepad
- `InputMapping` — action bindings: `bind_action(action, InputSource::Key(KeyCode::Space))`
- `ThreadSafeInput` — Arc<Mutex<InputHandler>> wrapper

## Per-Frame State
- `is_key_pressed(key)` — held this frame
- `is_key_just_pressed(key)` — pressed this frame (one-shot)
- `is_key_just_released(key)` — released this frame (one-shot)
- `is_action_active(action)` — any bound input active

## Known Tech Debt
- Missing dead zone normalization tests for gamepad
- Missing frame-accurate event timing tests

## Testing
- 56 tests, run with `cargo test -p input`
