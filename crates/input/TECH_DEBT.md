# Technical Debt: input

Last audited: June 2026 (full audit + structural refactor)

## Summary
- Behavioral bugs: 3 found, 3 fixed
- DRY violations: 2 found, 2 fixed
- SRP/architecture issues: 3 found, 3 fixed
- Dead code: ~250 lines removed
- Open items: 4 (all feature gaps or low-priority polish)

**Overall Assessment:** The June 2026 audit found the crate structurally sound
but with two real behavioral bugs, a non-reusable action model, and significant
dead weight. All were fixed in the same pass (see "Resolved" below). The crate
now has: a generic `InputMapping<A>` so games define their own actions, a
shared `ButtonTracker<T>` replacing three copies of the press-state pattern,
and an `InputHandler` reduced to pure device state. Remaining items are
feature gaps, not debt.

---

## Resolved (June 2026 Audit)

### [BUG-001] Stale mouse movement delta ✅ FIXED
- **Was:** `MouseState::update()` never reset `previous_position`, so after the
  mouse stopped moving, `movement_delta()` returned the last delta **forever**
  (continuous camera/drag drift). Multiple `CursorMoved` events per frame also
  collapsed to only the last segment.
- **Fix:** `MouseState` now accumulates `frame_delta` across all move events in
  a frame and zeroes it in `clear_frame_state()`. Wheel delta also accumulates
  (was: overwritten per event). `previous_position()` removed.
- **Regression tests:** `test_movement_delta_resets_each_frame`,
  `test_movement_delta_accumulates_within_frame` (tests/mouse.rs)

### [BUG-002] Multi-action bindings leaked on unbind/rebind ✅ FIXED
- **Was:** `InputMapping` kept two HashMaps (input→action and action→inputs)
  whose invariants desynced: `bind_input_to_multiple_actions` recorded only the
  first action in the forward map, so `unbind_input`/rebinding left stale
  bindings on the other actions (e.g. Space kept triggering `Select` after
  being unbound).
- **Fix:** Single source of truth — `HashMap<A, Vec<InputSource>>`. One source
  bound to many actions is now just multiple `bind()` calls; `unbind_source()`
  removes a source from *all* actions. `bind_input_to_multiple_actions` and
  `get_action` (and their documented asymmetry) are gone.
- **Regression test:** `test_unbind_source_removes_from_all_actions`

### [BUG-003] Incorrect action edge semantics ✅ FIXED
- **Was:** `is_action_just_deactivated` returned true if *any* bound source was
  just released — an action could be "just deactivated" while still active
  (release W while ArrowUp held). Same class of issue for re-triggering
  `just_activated` on a second key press mid-hold.
- **Fix:** `InputMapping::just_activated/just_deactivated` compare against
  reconstructed previous-frame state (`was_active`): activation fires only on
  the inactive→active edge, deactivation only on active→inactive.
- **Regression tests:** `test_second_source_does_not_retrigger_activation`,
  `test_releasing_one_source_keeps_action_active`

### [ARCH-003] Engine-owned action enum (reusability) ✅ FIXED
- **Was:** `GameAction` was a fixed engine enum (4 movement + 4 numbered
  actions + UI); games could only extend via `Custom(u32)` magic numbers.
  training.md documented a generic API that didn't exist. The editor crate had
  already been forced to hand-roll its own `EditorInputMapping`.
- **Fix:** `InputMapping<A: Copy + Eq + Hash>` is generic — games define their
  own action enums. `GameAction` survives as an *optional preset* via
  `InputMapping::with_default_bindings()`, used by `BehaviorRunner` for
  scene-defined behaviors (rebindable via `BehaviorRunner::actions_mut()`).
  `EditorInputMapping` now delegates to `InputMapping<EditorAction>`.

### [ARCH-004] Implicit default bindings in `new()` ✅ FIXED
- **Was:** `InputMapping::new()` silently bound WASD, left-click→Action1,
  Escape→Menu — least-surprise violation for any game wanting its own scheme.
- **Fix:** `new()` is empty; defaults are opt-in via `with_default_bindings()`.

### [SRP-002] InputHandler owned the action mapping ✅ FIXED
- **Was:** `InputHandler` mixed device state with one hardcoded
  `InputMapping` + action query methods.
- **Fix:** `InputHandler` is device state + event queue only. It exposes
  `is_source_pressed/just_pressed/just_released(&InputSource)`; action
  evaluation lives on `InputMapping` (`is_active(action, &input)`).

### [DRY-001] Triplicated press-state tracking ✅ FIXED
- **Was:** keyboard.rs, mouse.rs, gamepad.rs each reimplemented the identical
  pressed/just_pressed/just_released HashSet pattern (~40 lines × 3).
- **Fix:** Shared `ButtonTracker<T: Copy + Eq + Hash>` (button_tracker.rs)
  composed by all three device states. Press-while-held no longer re-triggers
  "just pressed" (OS key-repeat safe), tested once in one place.

### [DRY-002 / DEAD-001/2/3] Dead weight removed ✅ FIXED
- `ThreadSafeInputHandler` (158 lines, 17 hand-written lock wrappers): **deleted**
  — no consumer existed anywhere in the workspace (engine_core holds a plain
  `InputHandler`). Recreate if a real multi-threaded consumer appears (YAGNI).
- `init()` + `InputError` (never constructed): **deleted**.
- `InputThreadError::OperationError` (never constructed): gone with the module.
- Redundant `pub use input_handler::InputEvent` re-export: removed.

### [KISS-002] Misleading `update()` naming on device states ✅ FIXED
- Device-state `update()` methods (which only cleared one-shot flags) renamed
  to `clear_frame_state()`. `InputHandler::update()` (process + end_frame
  convenience) and `end_frame()` keep their names and documented lifecycle.

### Minor fixes in the same pass
- Gamepads auto-register on first event (`GamepadManager::get_or_register`);
  events for unknown gamepad IDs are no longer silently dropped.
- Scroll `PixelDelta` normalized to lines (÷16 px) so trackpad and mouse-wheel
  scroll speeds are comparable (was: raw pixel values ~100× line values).
- 3 clippy `map_or(false, …)` warnings fixed (`is_some_and`); crate is
  clippy-clean including tests.
- `MousePosition` `Default` impl replaced with derive.
- Docs updated to match reality: training.md "Input Mapping Pattern",
  README.md input example, crates/input/CLAUDE.md (which referenced
  nonexistent `InputSource::Key` / `ThreadSafeInput` names).

---

## Open Items

### [GAP-001] No gamepad backend — Medium priority
The state model (`GamepadState`, auto-registration, `InputSource::Gamepad`
bindings) is complete and tested, but nothing produces gamepad events: there is
no gilrs (or similar) integration, and winit doesn't carry gamepad input. The
default `Gamepad(0, …)` bindings in the preset are inert in practice.
**Next step:** add a `gilrs` poll in the engine's event loop that translates to
`InputEvent::GamepadButton*/GamepadAxisUpdated` and queues them on the handler.
Dead-zone normalization for analog sticks should land with the backend.

### [GAP-002] `MousePosition` / `(f32, f32)` instead of shared Vec2 — Low
`MousePosition` duplicates a 2D vector type and `movement_delta()` returns a
bare tuple; `common`/glam exist for this. Unifying touches `ui`, `editor`, and
`editor_integration` call sites — do it as its own small cross-crate pass.

### [GAP-003] No touch / gesture support — Low (feature gap)
No tap/drag/pinch recognition, no `WindowEvent::Touch` handling. Track in
PROJECT_ROADMAP.md if mobile/web targets become real.

### [GAP-004] Binding persistence — Low (feature gap)
`InputMapping` has no save/load (e.g. serde on `InputSource` + user remapping
UI). Needed eventually for "rebind keys" settings screens; serde derives on
`InputSource`/`GamepadButton` are the first step.

---

## Metrics (post-refactor)

| Metric | Value |
|--------|-------|
| Source files | 7 (`button_tracker`, `gamepad`, `input_handler`, `input_mapping`, `keyboard`, `mouse`, `prelude` + lib) |
| Source lines | ~1,030 (was ~1,170 with less functionality) |
| Tests | 62 passing (4 unit + 54 integration + 4 doc), 0 ignored |
| Clippy warnings | 0 (including `--all-targets`) |
| Error types | 0 (none needed — no fallible APIs remain) |
| Workspace impact | engine_core `BehaviorRunner` owns its `InputMapping<GameAction>`; editor `EditorInputMapping` delegates to `InputMapping<EditorAction>`; Pong unaffected (key-level API only) |

---

## Historical (pre-June-2026 audits)

The January/February 2026 audit items (DRY-002 action-check helpers, DRY-003
unbind duplication, SRP-001 dual update methods, KISS-001 multi-action binding
asymmetry, ARCH-001 dual error types) were either resolved then, or are now
moot — the code they referred to was replaced or deleted by the June 2026
restructure above. ARCH-002 (winit type coupling) remains an intentional,
documented design choice on `InputEvent`.
