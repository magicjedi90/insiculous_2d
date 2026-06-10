//! Keyboard input handling.

use crate::button_tracker::ButtonTracker;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Represents the state of a keyboard
#[derive(Debug, Default, Clone)]
pub struct KeyboardState {
    keys: ButtonTracker<KeyCode>,
}

impl KeyboardState {
    /// Create a new keyboard state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the keyboard state with a key press event
    pub fn handle_key_press(&mut self, key: KeyCode) {
        self.keys.press(key);
    }

    /// Update the keyboard state with a key release event
    pub fn handle_key_release(&mut self, key: KeyCode) {
        self.keys.release(key);
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys.is_pressed(key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys.is_just_pressed(key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys.is_just_released(key)
    }

    /// Clear the just pressed and just released sets for the next frame
    pub fn clear_frame_state(&mut self) {
        self.keys.clear_frame_state();
    }
}

/// Convert a winit physical key to a key code
pub fn convert_physical_key(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(code) => Some(code),
        _ => None,
    }
}
