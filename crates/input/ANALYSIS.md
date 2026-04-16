# Input System Analysis

## Audit Note (April 15, 2026)
Pruned after audit: removed the "TODO comments in tests" issue (fixed long ago
by commit `dbc42c4`), the obsolete `EngineApplication` wiring diagram (that
type no longer exists — `GameRunner` in `engine_core/src/game.rs` owns the
window-event and per-frame plumbing now), and the grab-bag "Future
Enhancements" laundry list (haptics, voice, motion, recording/playback, etc.)
that was aspirational rather than analytical. Kept: the Winit coupling
rationale, the frame lifecycle contract, dead-zone and timing gaps (still
real), the `bind_input_to_multiple_actions` reverse-lookup asymmetry, and the
thread-safe wrapper design. Test count corrected from 60 to the verified **56
passing** (`cargo test -p input`).

---

## Summary
- Input handling built around `InputHandler`, a per-frame event queue, and the
  `InputEvent` enum.
- Covers keyboard, mouse, and multi-gamepad input; action-layer mapping sits
  on top of device state.
- Winit `WindowEvent`s are the sole ingress point; a `ThreadSafeInputHandler`
  wraps everything in `Arc<Mutex<>>` for cross-thread access.

## Current State (April 2026)
**Test Count**: 56 passing, 0 failing, 0 ignored (doc-tests are `ignore`'d by
design — they contain pseudocode snippets).

---

## Architecture

### Module Layout
```
crates/input/src/
├── lib.rs              — re-exports + init() + InputError
├── input_handler.rs    — InputHandler, InputEvent, frame lifecycle
├── input_mapping.rs    — InputMapping, InputSource, GameAction
├── keyboard.rs         — KeyboardState + winit physical-key conversion
├── mouse.rs            — MouseState (position, buttons, wheel, delta)
├── gamepad.rs          — GamepadState, GamepadManager (per-id)
├── thread_safe.rs      — Arc<Mutex<InputHandler>> wrapper
└── prelude.rs          — curated public surface
```

### Event Flow
1. Host calls `input.handle_window_event(&winit_event)` — enqueued, not
   applied.
2. At frame start, host calls `input.process_queued_events()` — drains the
   `VecDeque<InputEvent>` into device state (handles just-pressed/just-released
   transitions).
3. Game logic reads state: `is_key_pressed`, `is_key_just_pressed`,
   `is_action_active`, etc.
4. At frame end, host calls `input.end_frame()` — clears the `just_*` sets on
   every device.
5. `update()` is a convenience that does steps 2 and 4 together; the split
   pair is preferred so `just_*` flags are observable for an entire frame.

Current host wiring lives in `engine_core/src/game.rs` (`GameRunner`):
`handle_window_event` at line 513, `process_queued_events` at line 328. There
is no `EngineApplication` class — the older ANALYSIS text that described one
was stale.

### API Quality
- Consistent naming: `is_*_pressed` / `is_*_just_pressed` / `is_*_just_released`.
- Strong enum typing throughout (`GameAction`, `InputSource`, `GamepadButton`,
  `GamepadAxis`).
- Thread-safe wrapper returns `Result<T, InputThreadError>` on every call so
  callers see mutex poisoning rather than panicking.

---

## Design Notes & Tradeoffs

### Winit Coupling Is Intentional
`InputEvent` carries `winit::keyboard::KeyCode` and `winit::event::MouseButton`
directly (see `input_handler.rs` doc comment above the enum). The reasoning:

- Winit is the de-facto Rust windowing crate; replacing it is unlikely.
- Skipping an abstraction layer means no conversion overhead and no
  duplicate enum maintenance.
- If a non-winit backend ever appears, the conversion can be added at the
  boundary inside `handle_window_event` without changing the public API.

This is a deliberate simplicity-over-portability tradeoff. Document the choice
when porting is discussed.

### One-Input-Many-Actions Asymmetry
`InputMapping` supports binding a single `InputSource` to multiple
`GameAction`s via `bind_input_to_multiple_actions`. Because the reverse
lookup (`get_action`) is a single-valued `HashMap<InputSource, GameAction>`,
it only returns the **first** action for a multi-bound input. Forward lookup
(`get_bindings(action)`) and `is_action_active()` are correct for all actions.

Recommendation baked into the doc comments: prefer
`InputHandler::is_action_active()` over `get_action()` when you need to know
whether an action fires.

### Frame Lifecycle Contract
The module-level doc on `input_handler.rs` is the canonical description.
Diverging from it (e.g., calling `end_frame` before reading `just_pressed`)
silently breaks one-shot detection. The `update()` shortcut hides this
footgun by fusing the two steps; keep it available for trivial use cases but
steer non-trivial callers to the split form.

### Thread-Safe Wrapper Scope
`ThreadSafeInputHandler` mirrors the full `InputHandler` API behind a
`Mutex`. It returns owned clones for state snapshots (`keyboard_state()`,
`mouse_state()`, `gamepad_manager()`) rather than guard references — callers
can release the lock immediately and work on the snapshot. This is the right
shape for a read-mostly integration, but it means each snapshot is an
allocation; if a hot loop ever reads state from another thread, reconsider.

---

## Known Gaps

### Gamepad Analog Dead Zones
`GamepadState::axis_value` returns the raw axis value verbatim. There is no
configurable dead-zone threshold, no radial-vs-axial normalization, and no
tests covering either. Games that need clean stick input currently have to
filter the raw value themselves. Adding a `DeadZoneConfig` on
`GamepadManager` (with per-axis thresholds) plus a `normalized_axis_value`
accessor would be the minimal fix.

### Event Timing / Frame-Accurate Tests
No test asserts ordering guarantees when multiple events of different kinds
arrive in one frame (e.g., press + move + release of the same mouse button).
The queue is a `VecDeque` so FIFO is structurally guaranteed, but a test
that locks that in would catch future regressions.

### Keyboard Layout Normalization
`convert_physical_key` maps winit physical keys directly; there's no
accommodation for logical-key remapping (AZERTY, Dvorak, etc.). Games that
care currently have to wire `WindowEvent::KeyboardInput.logical_key`
themselves, which isn't exposed through our `InputEvent`. Not urgent — most
2D game input is positional — but worth noting if the engine grows a text
input story.

---

## Future Directions (if/when needed)
These are not debt, just features that don't exist yet:

- **Dead-zone configuration** (see gap above — smallest concrete next step).
- **Gesture recognition** (double-click, drag, pinch). Would live as a layer
  above `InputEvent`, not inside it.
- **Chord detection** (Ctrl+Shift+Key). `InputMapping` doesn't model
  modifier+key as a unit today.
- **Context stack** (modal overlays that consume input before gameplay sees
  it). Currently every consumer has to check its own "am I active" flag.
- **Touch / multi-touch** if the engine ever targets mobile.
- **Haptic output** (controller rumble) — would need a new trait since
  `InputHandler` is strictly read-side today.

---

## Production Readiness
Stable for keyboard / mouse / gamepad in desktop games:
- Core event pipeline and per-frame state transitions are covered by 56
  tests.
- Thread-safe wrapper has dedicated concurrency tests.
- Action mapping supports the common one-action-many-inputs case cleanly.

The gaps above (dead zones, multi-event ordering, layout normalization) are
all additive — none require breaking changes to close.
