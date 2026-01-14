# UI Interaction Bug Analysis and Fix

## Problem Identified

The UI button and slider click detection is failing because of a timing issue in the `InteractionManager::begin_frame()` method. 

### Root Cause

In `crates/ui/src/interaction.rs`, lines 193-195:

```rust
// Handle mouse release - deactivate widget
if self.input.mouse_just_released {
    self.active_widget = None;
}
```

This code clears the `active_widget` BEFORE widgets have a chance to check if they were clicked. The click detection logic in `interact()` method (line 287) requires both:

1. `is_active` (widget is the active widget)
2. `self.input.mouse_just_released` (mouse was just released)

But since `active_widget` is cleared in `begin_frame()`, `is_active` becomes false when `interact()` is called.

### The Fix

We need to defer clearing the `active_widget` until after widgets have had a chance to detect clicks. The mouse release handling should happen in `end_frame()` instead of `begin_frame()`.

Here's the corrected logic:

```rust
pub fn begin_frame(&mut self, input: &InputHandler) {
    self.input = InputState::from_input_handler(input);

    // Clear hot widget at start of frame (will be set by widgets that are hovered)
    self.hot_widget = None;

    // DON'T clear active_widget here - let widgets check for clicks first
    // The active_widget will be cleared in end_frame() after click detection

    // Mark all persistent state as not seen
    for state in self.persistent_state.values_mut() {
        state.seen_this_frame = false;
    }
}

pub fn end_frame(&mut self) {
    // Clear active widget if mouse was just released (after click detection)
    if self.input.mouse_just_released {
        self.active_widget = None;
    }

    // Garbage collect persistent state for widgets not seen this frame
    self.persistent_state.retain(|_, state| state.seen_this_frame);
}
```

### Alternative Fix

Another approach is to check for clicks in `begin_frame()` before clearing the active widget:

```rust
pub fn begin_frame(&mut self, input: &InputHandler) {
    self.input = InputState::from_input_handler(input);

    // Clear hot widget at start of frame (will be set by widgets that are hovered)
    self.hot_widget = None;

    // Check for clicks before clearing active widget
    // Store the fact that mouse was released this frame for click detection
    let mouse_was_released = self.input.mouse_just_released;

    // Handle mouse release - deactivate widget
    if mouse_was_released {
        self.active_widget = None;
    }

    // Mark all persistent state as not seen
    for state in self.persistent_state.values_mut() {
        state.seen_this_frame = false;
    }
}
```

But this approach is more complex. The first fix (moving the clearing to `end_frame()`) is cleaner.

## Test Results

The current tests fail because:

1. `test_ui_button_click_detection` - Button click not detected due to timing issue
2. `test_interaction_manager_click_logic` - Same timing issue
3. `test_input_state_from_input_handler` - Mouse state not being cleared properly

After applying the fix, all tests should pass.

## Implementation Steps

1. Move the `active_widget` clearing logic from `begin_frame()` to `end_frame()`
2. Run the tests to verify the fix works
3. Test with the actual game engine to ensure no regressions