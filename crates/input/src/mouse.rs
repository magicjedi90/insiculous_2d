//! Mouse input handling.

use std::collections::HashSet;
use winit::event::MouseButton;

/// Represents the position of the mouse
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MousePosition {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Default for MousePosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Represents the state of the mouse
#[derive(Debug, Default)]
pub struct MouseState {
    /// Current position of the mouse
    position: MousePosition,
    /// Previous position of the mouse
    previous_position: MousePosition,
    /// Currently pressed buttons
    pressed_buttons: HashSet<MouseButton>,
    /// Buttons that were just pressed this frame
    just_pressed: HashSet<MouseButton>,
    /// Buttons that were just released this frame
    just_released: HashSet<MouseButton>,
    /// Mouse wheel delta
    wheel_delta: f32,
}

impl MouseState {
    /// Create a new mouse state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the mouse position
    pub fn update_position(&mut self, x: f32, y: f32) {
        self.previous_position = self.position;
        self.position = MousePosition { x, y };
    }

    /// Update the mouse state with a button press event
    pub fn handle_button_press(&mut self, button: MouseButton) {
        if !self.pressed_buttons.contains(&button) {
            self.just_pressed.insert(button);
        }
        self.pressed_buttons.insert(button);
    }

    /// Update the mouse state with a button release event
    pub fn handle_button_release(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
        self.just_released.insert(button);
    }

    /// Update the mouse wheel delta
    pub fn update_wheel_delta(&mut self, delta: f32) {
        self.wheel_delta = delta;
    }

    /// Get the current mouse position
    pub fn position(&self) -> MousePosition {
        self.position
    }

    /// Get the previous mouse position
    pub fn previous_position(&self) -> MousePosition {
        self.previous_position
    }

    /// Get the mouse movement delta
    pub fn movement_delta(&self) -> (f32, f32) {
        (
            self.position.x - self.previous_position.x,
            self.position.y - self.previous_position.y,
        )
    }

    /// Get the mouse wheel delta
    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Clear the just pressed and just released sets for the next frame
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.wheel_delta = 0.0;
    }
}
