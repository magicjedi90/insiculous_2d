# Technical Debt: input

Last audited: January 2026

## Summary
- DRY violations: 3
- SRP violations: 1
- KISS violations: 1
- Architecture issues: 2

**Overall Assessment:** The input crate is well-designed with clean architecture. Most issues are minor DRY violations from the necessary structural similarity between input device types.

---

## DRY Violations

### [DRY-001] Repeated input state tracking pattern across device types
- **Files:** `keyboard.rs:7-15`, `mouse.rs:22-36`, `gamepad.rs:37-48`
- **Issue:** All three device state structs use identical pattern for tracking input:
  ```rust
  pressed_keys/pressed_buttons: HashSet<...>,
  just_pressed: HashSet<...>,
  just_released: HashSet<...>,
  ```
  With identical methods: `handle_*_press()`, `handle_*_release()`, `is_*_pressed()`, `is_*_just_pressed()`, `is_*_just_released()`, `update()`.
- **Suggested fix:** Consider a generic `InputStateTracker<T>` struct:
  ```rust
  pub struct InputStateTracker<T: Eq + Hash + Copy> {
      pressed: HashSet<T>,
      just_pressed: HashSet<T>,
      just_released: HashSet<T>,
  }
  ```
- **Priority:** Low (working pattern, abstraction may add complexity)

### [DRY-002] Repeated action checking pattern in input_handler.rs
- **File:** `input_handler.rs`
- **Lines:** 71-149
- **Issue:** `is_action_active()`, `is_action_just_activated()`, and `is_action_just_deactivated()` have identical structure:
  ```rust
  let bindings = self.input_mapping.get_bindings(action);
  for binding in bindings {
      match binding {
          InputSource::Keyboard(key) => { if self.keyboard.is_key_XXX(*key) { return true; } }
          InputSource::Mouse(button) => { if self.mouse.is_button_XXX(*button) { return true; } }
          InputSource::Gamepad(id, button) => { ... }
      }
  }
  false
  ```
- **Suggested fix:** Extract a helper method that takes a closure for the check:
  ```rust
  fn check_bindings<F>(&self, action: &GameAction, check: F) -> bool
  where F: Fn(&InputSource) -> bool
  ```
- **Priority:** Medium

### [DRY-003] Repeated unbind logic in input_mapping.rs
- **File:** `input_mapping.rs`
- **Lines:** 104-118, 122-138
- **Issue:** The logic to remove an existing binding and clean up `action_bindings` is duplicated in `bind_input()` and `bind_input_to_multiple_actions()`:
  ```rust
  if let Some(old_action) = self.bindings.remove(&input) {
      if let Some(sources) = self.action_bindings.get_mut(&old_action) {
          sources.retain(|&s| s != input);
          ...
      }
  }
  ```
- **Suggested fix:** Extract to `fn remove_existing_binding(&mut self, input: &InputSource)`.
- **Priority:** Low

---

## SRP Violations

### [SRP-001] InputHandler has dual update methods
- **File:** `input_handler.rs`
- **Lines:** 238-258
- **Issue:** Two similar methods exist for frame-end processing:
  - `update()` - processes queued events AND clears just_pressed/released
  - `end_frame()` - ONLY clears just_pressed/released

  The documentation says use `end_frame()` "when you've already called `process_queued_events()` earlier in the frame" but this creates confusion about the correct API to call.
- **Suggested fix:** Either:
  1. Remove `end_frame()` and always use `update()`
  2. Rename to make distinction clear: `process_and_end_frame()` vs `end_frame()`
  3. Document more clearly which to use when
- **Priority:** Medium (API confusion)

---

## KISS Violations

### [KISS-001] bind_input_to_multiple_actions has inconsistent behavior
- **File:** `input_mapping.rs`
- **Lines:** 122-139
- **Issue:** The method `bind_input_to_multiple_actions()` only stores the first action in `bindings` HashMap, but adds the input to all action_bindings. This creates asymmetric behavior:
  - `get_bindings(action)` returns correct inputs for ALL actions
  - `get_action(input)` only returns the FIRST action

  The comment says "Bind to the first action for the reverse lookup" but this is confusing and potentially buggy.
- **Suggested fix:** Either:
  1. Don't support multiple actions per input (simpler)
  2. Change `bindings` to `HashMap<InputSource, Vec<GameAction>>`
  3. Document the limitation clearly
- **Priority:** Medium (semantic confusion)

---

## Architecture Issues

### [ARCH-001] Dual error types for same domain
- **Files:** `lib.rs:32-39`, `thread_safe.rs:151-159`
- **Issue:** Two separate error enums exist:
  - `InputError` - For initialization and device errors
  - `InputThreadError` - For thread-safe wrapper errors

  These aren't unified, so different APIs return different error types.
- **Suggested fix:** Either:
  1. Have `InputThreadError` wrap `InputError`
  2. Unify into a single `InputError` with thread-related variants
- **Priority:** Low (working, just inconsistent)

### [ARCH-002] InputEvent uses winit types directly
- **File:** `input_handler.rs`
- **Lines:** 11-31
- **Issue:** `InputEvent` variants use winit types directly:
  ```rust
  KeyPressed(winit::keyboard::KeyCode),
  MouseButtonPressed(winit::event::MouseButton),
  ```
  This couples the event system to winit. If the engine ever needs to support other windowing systems or abstract input sources, this will need refactoring.
- **Suggested fix:** Consider internal key/button enums that are converted from winit types at the boundary. Low priority since winit is the standard for Rust game engines.
- **Priority:** Low (acceptable coupling for now)

---

## Previously Resolved (Reference)

These issues from ANALYSIS.md have been resolved:

| Issue | Resolution |
|-------|------------|
| TODO comments in tests | FIXED: All 36 TODO comments replaced with assertions |
| Event integration | FIXED: Properly forwarded in window event loop |
| Thread safety | FIXED: ThreadSafeInputHandler wrapper |

---

## Remaining Gaps (from ANALYSIS.md)

These are **feature gaps**, not technical debt:
- No gamepad analog stick dead zones
- No input event timing tests
- No gesture recognition (double-click, drag)
- No touch input support
- No haptic feedback

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 8 |
| Total lines | ~650 |
| Test coverage | 60 tests (all passing) |
| Error types | 2 (could be unified) |
| High priority issues | 0 |
| Medium priority issues | 4 |
| Low priority issues | 3 |

---

## Recommendations

### Immediate Actions
None required - the crate is production-ready.

### Short-term Improvements
1. **Fix SRP-001** - Clarify `update()` vs `end_frame()` API
2. **Fix DRY-002** - Extract action binding check helper
3. **Fix KISS-001** - Document or fix multi-action binding behavior

### Technical Debt Backlog
- DRY-001: Consider generic InputStateTracker (optional, may over-abstract)
- ARCH-001: Unify error types
- ARCH-002: Abstract winit types (only if multi-platform windowing needed)

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| TODO comments | ✅ COMPLETED - All replaced | Resolved |
| Dead zone tests | "No Gamepad Analog Stick Dead Zone Tests" | Feature gap (not debt) |
| DRY-002: Action checking | Not tracked | New finding |
| SRP-001: Dual update methods | Not tracked | New finding |
| KISS-001: Multi-action binding | Not tracked | New finding |

**New issues to add to PROJECT_ROADMAP.md:**
- SRP-001: InputHandler has confusing dual update methods (`update()` vs `end_frame()`)
- KISS-001: `bind_input_to_multiple_actions()` has asymmetric behavior

---

## Code Quality Notes

### Strengths
1. **Clean module separation** - Each device type has its own module
2. **Good abstraction** - Action-based input system is well-designed
3. **Thread safety** - Proper `Arc<Mutex<>>` wrapper for concurrent access
4. **Comprehensive API** - Full keyboard, mouse, gamepad support
5. **Good test coverage** - 60 tests covering core functionality
6. **Event queue system** - Proper event buffering for frame-based processing

### Minor Observations
- The crate correctly re-exports winit types in prelude for convenience
- Default bindings are sensible and well-organized
- Event flow is well-documented in ANALYSIS.md
