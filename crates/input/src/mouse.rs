//! Mouse input handling.

use crate::button_tracker::ButtonTracker;
use winit::event::MouseButton;

/// Represents the position of the mouse
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MousePosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// Represents the state of the mouse
#[derive(Debug, Default, Clone)]
pub struct MouseState {
    /// Current position of the mouse
    position: MousePosition,
    /// Movement accumulated over the current frame (sum of all move events)
    frame_delta: (f32, f32),
    /// Button press state
    buttons: ButtonTracker<MouseButton>,
    /// Mouse wheel delta accumulated over the current frame
    wheel_delta: f32,
}

impl MouseState {
    /// Create a new mouse state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the mouse position, accumulating the movement delta for this frame.
    ///
    /// Multiple position updates within one frame (common at high polling rates)
    /// are summed, so `movement_delta()` reflects the full frame movement.
    pub fn update_position(&mut self, x: f32, y: f32) {
        self.frame_delta.0 += x - self.position.x;
        self.frame_delta.1 += y - self.position.y;
        self.position = MousePosition { x, y };
    }

    /// Update the mouse state with a button press event
    pub fn handle_button_press(&mut self, button: MouseButton) {
        self.buttons.press(button);
    }

    /// Update the mouse state with a button release event
    pub fn handle_button_release(&mut self, button: MouseButton) {
        self.buttons.release(button);
    }

    /// Accumulate a mouse wheel scroll delta for this frame
    pub fn update_wheel_delta(&mut self, delta: f32) {
        self.wheel_delta += delta;
    }

    /// Get the current mouse position
    pub fn position(&self) -> MousePosition {
        self.position
    }

    /// Get the mouse movement accumulated this frame.
    ///
    /// Returns `(0.0, 0.0)` on frames where the mouse did not move.
    pub fn movement_delta(&self) -> (f32, f32) {
        self.frame_delta
    }

    /// Get the mouse wheel delta accumulated this frame
    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons.is_pressed(button)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.buttons.is_just_pressed(button)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.buttons.is_just_released(button)
    }

    /// Clear per-frame state (just pressed/released, movement and wheel deltas)
    pub fn clear_frame_state(&mut self) {
        self.buttons.clear_frame_state();
        self.frame_delta = (0.0, 0.0);
        self.wheel_delta = 0.0;
    }
}
