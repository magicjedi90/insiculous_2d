//! Gamepad input handling.
//!
//! # Backend Status
//!
//! This module tracks gamepad *state* but the engine currently has no gamepad
//! backend (e.g. gilrs) producing [`crate::InputEvent`] gamepad events. Until a
//! backend is wired up, gamepad state only changes if events are queued manually
//! via [`crate::InputHandler::queue_event`]. Gamepads are auto-registered when
//! their first event is processed.

use crate::button_tracker::ButtonTracker;
use std::collections::HashMap;

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
#[derive(Debug, Default, Clone)]
pub struct GamepadState {
    /// Button press state
    buttons: ButtonTracker<GamepadButton>,
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
        self.buttons.press(button);
    }

    /// Update the gamepad state with a button release event
    pub fn handle_button_release(&mut self, button: GamepadButton) {
        self.buttons.release(button);
    }

    /// Update an axis value
    pub fn update_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.axis_values.insert(axis, value);
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.is_pressed(button)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.is_just_pressed(button)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.buttons.is_just_released(button)
    }

    /// Get the value of an axis
    pub fn axis_value(&self, axis: GamepadAxis) -> f32 {
        *self.axis_values.get(&axis).unwrap_or(&0.0)
    }

    /// Clear the just pressed and just released sets for the next frame
    pub fn clear_frame_state(&mut self) {
        self.buttons.clear_frame_state();
    }
}

/// Manages all connected gamepads
#[derive(Debug, Default, Clone)]
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

    /// Get a gamepad state, registering it if not yet known.
    ///
    /// Used by event processing so events for a new gamepad are never
    /// silently dropped.
    pub fn get_or_register(&mut self, id: u32) -> &mut GamepadState {
        self.gamepad_states.entry(id).or_default()
    }

    /// Clear per-frame state on all gamepads
    pub fn clear_frame_state(&mut self) {
        for state in self.gamepad_states.values_mut() {
            state.clear_frame_state();
        }
    }
}
