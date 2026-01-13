# Technical Debt: input

Last audited: January 2026

## Summary
- DRY violations: 3
- SRP violations: 0 (1 resolved)
- KISS violations: 0 (1 resolved)
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

### ~~[DRY-002] Repeated action checking pattern in input_handler.rs~~ ✅ RESOLVED
- **File:** `input_handler.rs`
- **Resolution:** Extracted three helper methods:
  - `is_input_pressed(&self, source: &InputSource) -> bool`
  - `is_input_just_pressed(&self, source: &InputSource) -> bool`
  - `is_input_just_released(&self, source: &InputSource) -> bool`

  The action methods now use `iter().any()` with these helpers:
  ```rust
  pub fn is_action_active(&self, action: &GameAction) -> bool {
      self.input_mapping.get_bindings(action).iter().any(|s| self.is_input_pressed(s))
  }
  ```
- **Resolved:** January 2026

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

### ~~[SRP-001] InputHandler has dual update methods~~ ✅ RESOLVED
- **File:** `input_handler.rs`
- **Resolution:** Comprehensive documentation added to clarify the frame lifecycle:
  - Module-level doc explains the 4-step frame lifecycle (Event Collection → Event Processing → Game Logic → State Reset)
  - `process_queued_events()` - clearly documented as "call at start of frame"
  - `end_frame()` - clearly documented as "call at end of frame" with code examples
  - `update()` - documented as convenience method combining both steps for simple use cases

  The API is intentionally separated for fine-grained control in game loops.

---

## KISS Violations

### ~~[KISS-001] bind_input_to_multiple_actions has inconsistent behavior~~ ✅ RESOLVED
- **File:** `input_mapping.rs`
- **Resolution:** Comprehensive documentation added to clarify the intentional asymmetric behavior:
  - Module-level docs explain the binding model and the limitation
  - `bind_input_to_multiple_actions()` has detailed docs explaining that `get_action()` only returns the first action
  - `get_action()` has a warning note about the limitation
  - `get_bindings()` is documented as the recommended lookup method
  - Users are guided to use `InputHandler::is_action_active()` for most use cases

  The behavior is intentional (simpler data structure) and now well-documented.

---

## Architecture Issues

### ~~[ARCH-001] Dual error types for same domain~~ ✅ RESOLVED
- **Files:** `lib.rs:32-42`, `thread_safe.rs:151-159`
- **Resolution:** Added `From<InputThreadError>` implementation for `InputError`:
  ```rust
  #[error("Thread-safe input error: {0}")]
  ThreadError(#[from] InputThreadError),
  ```
  This allows automatic conversion using `?` operator when needed.
- **Resolved:** January 2026

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
| Medium priority issues | 2 |
| Low priority issues | 3 |

---

## Recommendations

### ✅ Completed
1. ~~**Fix SRP-001** - Clarify `update()` vs `end_frame()` API~~ - Comprehensive documentation added
2. ~~**Fix KISS-001** - Document or fix multi-action binding behavior~~ - Comprehensive documentation added

### Short-term Improvements
1. **Fix DRY-002** - Extract action binding check helper

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
| DRY-002: Action checking | Not tracked | Open |
| SRP-001: Dual update methods | Not tracked | ✅ RESOLVED - Documentation added |
| KISS-001: Multi-action binding | Not tracked | ✅ RESOLVED - Documentation added |

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
