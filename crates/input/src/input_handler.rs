//! Unified input handling.

use crate::gamepad::GamepadManager;
use crate::keyboard::KeyboardState;
use crate::mouse::MouseState;

/// A unified handler for all input types
#[derive(Debug, Default)]
pub struct InputHandler {
    /// Keyboard state
    keyboard: KeyboardState,
    /// Mouse state
    mouse: MouseState,
    /// Gamepad manager
    gamepads: GamepadManager,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            gamepads: GamepadManager::new(),
        }
    }

    /// Get a reference to the keyboard state
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    /// Get a mutable reference to the keyboard state
    pub fn keyboard_mut(&mut self) -> &mut KeyboardState {
        &mut self.keyboard
    }

    /// Get a reference to the mouse state
    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    /// Get a mutable reference to the mouse state
    pub fn mouse_mut(&mut self) -> &mut MouseState {
        &mut self.mouse
    }

    /// Get a reference to the gamepad manager
    pub fn gamepads(&self) -> &GamepadManager {
        &self.gamepads
    }

    /// Get a mutable reference to the gamepad manager
    pub fn gamepads_mut(&mut self) -> &mut GamepadManager {
        &mut self.gamepads
    }

    /// Update all input states for the next frame
    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
        self.gamepads.update();
    }
}
