//! Input abstraction for the insiculous_2d game engine.
//!
//! This crate provides abstractions for keyboard, mouse, and gamepad input.

mod gamepad;
mod input_handler;
mod keyboard;
mod mouse;

pub mod prelude;

// Re-export for convenience
pub use gamepad::*;
pub use input_handler::*;
pub use keyboard::*;
pub use mouse::*;

/// Initialize the input system
pub fn init() -> Result<InputHandler, InputError> {
    log::info!("Input system initialized");
    Ok(InputHandler::new())
}

/// Errors that can occur in the input system
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("Failed to initialize input system: {0}")]
    InitializationError(String),

    #[error("Input device error: {0}")]
    DeviceError(String),
}
