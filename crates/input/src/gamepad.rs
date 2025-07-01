//! Gamepad input handling.

use std::collections::{HashMap, HashSet};

/// Represents a gamepad button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// Represents a gamepad axis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

/// Represents the state of a single gamepad
#[derive(Debug, Default)]
pub struct GamepadState {
    /// Currently pressed buttons
    pressed_buttons: HashSet<GamepadButton>,
    /// Buttons that were just pressed this frame
    just_pressed: HashSet<GamepadButton>,
    /// Buttons that were just released this frame
    just_released: HashSet<GamepadButton>,
    /// Current axis values
    axis_values: HashMap<GamepadAxis, f32>,
}

impl GamepadState {
    /// Create a new gamepad state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the gamepad state with a button press event
    pub fn handle_button_press(&mut self, button: GamepadButton) {
        if !self.pressed_buttons.contains(&button) {
            self.just_pressed.insert(button);
        }
        self.pressed_buttons.insert(button);
    }

    /// Update the gamepad state with a button release event
    pub fn handle_button_release(&mut self, button: GamepadButton) {
        self.pressed_buttons.remove(&button);
        self.just_released.insert(button);
    }

    /// Update an axis value
    pub fn update_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.axis_values.insert(axis, value);
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Get the value of an axis
    pub fn axis_value(&self, axis: GamepadAxis) -> f32 {
        *self.axis_values.get(&axis).unwrap_or(&0.0)
    }

    /// Clear the just pressed and just released sets for the next frame
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

/// Manages all connected gamepads
#[derive(Debug, Default)]
pub struct GamepadManager {
    /// States for all connected gamepads
    gamepad_states: HashMap<u32, GamepadState>,
}

impl GamepadManager {
    /// Create a new gamepad manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new gamepad
    pub fn register_gamepad(&mut self, id: u32) {
        self.gamepad_states.insert(id, GamepadState::new());
    }

    /// Unregister a gamepad
    pub fn unregister_gamepad(&mut self, id: u32) {
        self.gamepad_states.remove(&id);
    }

    /// Get a reference to a gamepad state
    pub fn get_gamepad(&self, id: u32) -> Option<&GamepadState> {
        self.gamepad_states.get(&id)
    }

    /// Get a mutable reference to a gamepad state
    pub fn get_gamepad_mut(&mut self, id: u32) -> Option<&mut GamepadState> {
        self.gamepad_states.get_mut(&id)
    }

    /// Update all gamepad states
    pub fn update(&mut self) {
        for state in self.gamepad_states.values_mut() {
            state.update();
        }
    }
}
