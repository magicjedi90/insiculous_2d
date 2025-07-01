//! Keyboard input handling.

use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Represents the state of a keyboard
#[derive(Debug, Default)]
pub struct KeyboardState {
    /// Currently pressed keys
    pressed_keys: HashSet<KeyCode>,
    /// Keys that were just pressed this frame
    just_pressed: HashSet<KeyCode>,
    /// Keys that were just released this frame
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    /// Create a new keyboard state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the keyboard state with a key press event
    pub fn handle_key_press(&mut self, key: KeyCode) {
        if !self.pressed_keys.contains(&key) {
            self.just_pressed.insert(key);
        }
        self.pressed_keys.insert(key);
    }

    /// Update the keyboard state with a key release event
    pub fn handle_key_release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
        self.just_released.insert(key);
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Clear the just pressed and just released sets for the next frame
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

/// Convert a winit physical key to a key code
pub fn convert_physical_key(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(code) => Some(code),
        _ => None,
    }
}
