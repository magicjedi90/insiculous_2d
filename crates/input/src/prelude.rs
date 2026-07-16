//! Prelude module for the input crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    button_tracker::ButtonTracker,
    gamepad::{AxisDirection, GamepadAxis, GamepadButton, GamepadManager, GamepadState},
    input_mapping::{GameAction, InputMapping, InputSource, AXIS_ACTIVATION_THRESHOLD},
    keyboard::KeyboardState,
    mouse::{MousePosition, MouseState},
    player::{InputSettings, PlayerBindings, PlayerId, PlayerSource},
    InputEvent, InputHandler,
};
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
