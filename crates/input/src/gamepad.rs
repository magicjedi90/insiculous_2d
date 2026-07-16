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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

/// Represents a gamepad axis.
///
/// Stick Y axes follow the gilrs convention: **positive = up**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    /// Positive = up
    LeftStickY,
    RightStickX,
    /// Positive = up
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

/// Which half of an axis' range counts as "pressed" when an analog axis is
/// used like a digital button (e.g. stick-left bound to a MoveLeft action).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AxisDirection {
    /// Axis value at or above `+threshold` (stick right / stick up / trigger pulled)
    Positive,
    /// Axis value at or below `-threshold` (stick left / stick down)
    Negative,
}

/// Whether `value` is past `threshold` in the given direction.
fn axis_past_threshold(value: f32, direction: AxisDirection, threshold: f32) -> bool {
    match direction {
        AxisDirection::Positive => value >= threshold,
        AxisDirection::Negative => value <= -threshold,
    }
}

/// Represents the state of a single gamepad
#[derive(Debug, Default, Clone)]
pub struct GamepadState {
    /// Button press state
    buttons: ButtonTracker<GamepadButton>,
    /// Current axis values
    axis_values: HashMap<GamepadAxis, f32>,
    /// Axis values as of the end of the previous frame (snapshotted in
    /// [`GamepadState::clear_frame_state`]) — enables axis edge detection.
    prev_axis_values: HashMap<GamepadAxis, f32>,
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

    /// Get the value an axis had at the end of the previous frame
    pub fn prev_axis_value(&self, axis: GamepadAxis) -> f32 {
        *self.prev_axis_values.get(&axis).unwrap_or(&0.0)
    }

    /// Check if an axis is currently past `threshold` in `direction`
    pub fn axis_active(&self, axis: GamepadAxis, direction: AxisDirection, threshold: f32) -> bool {
        axis_past_threshold(self.axis_value(axis), direction, threshold)
    }

    /// Check if an axis crossed `threshold` in `direction` this frame
    /// (active now, was not active at the end of the previous frame)
    pub fn axis_just_activated(
        &self,
        axis: GamepadAxis,
        direction: AxisDirection,
        threshold: f32,
    ) -> bool {
        self.axis_active(axis, direction, threshold)
            && !axis_past_threshold(self.prev_axis_value(axis), direction, threshold)
    }

    /// Check if an axis dropped back inside `threshold` in `direction` this
    /// frame (inactive now, was active at the end of the previous frame)
    pub fn axis_just_deactivated(
        &self,
        axis: GamepadAxis,
        direction: AxisDirection,
        threshold: f32,
    ) -> bool {
        !self.axis_active(axis, direction, threshold)
            && axis_past_threshold(self.prev_axis_value(axis), direction, threshold)
    }

    /// Clear per-frame state for the next frame: just-pressed/just-released
    /// button sets, and the previous-frame axis snapshot.
    pub fn clear_frame_state(&mut self) {
        self.buttons.clear_frame_state();
        self.prev_axis_values.clone_from(&self.axis_values);
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

    /// Iterate over all connected gamepads as `(id, state)` pairs
    pub fn iter(&self) -> impl Iterator<Item = (u32, &GamepadState)> {
        self.gamepad_states.iter().map(|(id, state)| (*id, state))
    }

    /// Ids of all currently connected gamepads
    pub fn connected_ids(&self) -> Vec<u32> {
        self.gamepad_states.keys().copied().collect()
    }

    /// Clear per-frame state on all gamepads
    pub fn clear_frame_state(&mut self) {
        for state in self.gamepad_states.values_mut() {
            state.clear_frame_state();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_just_activated_fires_once_on_crossing_and_rearms_below_threshold() {
        let mut pad = GamepadState::new();

        // Frame 1: stick pushed past the threshold
        pad.update_axis(GamepadAxis::LeftStickX, 0.8);
        assert!(pad.axis_just_activated(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
        pad.clear_frame_state();

        // Frame 2: still held — no re-trigger
        pad.update_axis(GamepadAxis::LeftStickX, 0.9);
        assert!(pad.axis_active(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
        assert!(!pad.axis_just_activated(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
        pad.clear_frame_state();

        // Frame 3: released back inside the threshold
        pad.update_axis(GamepadAxis::LeftStickX, 0.1);
        assert!(pad.axis_just_deactivated(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
        pad.clear_frame_state();

        // Frame 4: pushed again — re-armed, fires again
        pad.update_axis(GamepadAxis::LeftStickX, 0.7);
        assert!(pad.axis_just_activated(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
    }

    #[test]
    fn negative_direction_activates_on_negative_values_only() {
        let mut pad = GamepadState::new();
        pad.update_axis(GamepadAxis::LeftStickX, -0.6);
        assert!(pad.axis_active(GamepadAxis::LeftStickX, AxisDirection::Negative, 0.5));
        assert!(!pad.axis_active(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));

        pad.update_axis(GamepadAxis::LeftStickX, 0.6);
        assert!(!pad.axis_active(GamepadAxis::LeftStickX, AxisDirection::Negative, 0.5));
        assert!(pad.axis_active(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
    }

    #[test]
    fn opposite_directions_track_edges_independently() {
        let mut pad = GamepadState::new();

        // Swing from hard left to hard right in one frame
        pad.update_axis(GamepadAxis::LeftStickX, -0.9);
        pad.clear_frame_state();
        pad.update_axis(GamepadAxis::LeftStickX, 0.9);

        assert!(pad.axis_just_activated(GamepadAxis::LeftStickX, AxisDirection::Positive, 0.5));
        assert!(pad.axis_just_deactivated(GamepadAxis::LeftStickX, AxisDirection::Negative, 0.5));
    }

    #[test]
    fn manager_iter_and_connected_ids_reflect_registration() {
        let mut manager = GamepadManager::new();
        manager.register_gamepad(0);
        manager.register_gamepad(1);

        let mut ids = manager.connected_ids();
        ids.sort_unstable();
        assert_eq!(ids, vec![0, 1]);
        assert_eq!(manager.iter().count(), 2);

        manager.unregister_gamepad(0);
        assert_eq!(manager.connected_ids(), vec![1]);
    }
}
