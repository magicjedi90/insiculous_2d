//! Prelude module for the input crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    gamepad::{GamepadAxis, GamepadButton, GamepadManager, GamepadState},
    init,
    keyboard::KeyboardState,
    mouse::{MousePosition, MouseState},
    input_mapping::{InputMapping, InputSource, GameAction},
    thread_safe::{ThreadSafeInputHandler, InputThreadError},
    InputError, InputHandler, InputEvent,
};
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
