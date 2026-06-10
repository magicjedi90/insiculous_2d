//! Prelude module for the input crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    button_tracker::ButtonTracker,
    gamepad::{GamepadAxis, GamepadButton, GamepadManager, GamepadState},
    input_mapping::{GameAction, InputMapping, InputSource},
    keyboard::KeyboardState,
    mouse::{MousePosition, MouseState},
    InputEvent, InputHandler,
};
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
