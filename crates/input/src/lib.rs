//! Input abstraction for the insiculous_2d game engine.
//!
//! This crate provides abstractions for keyboard, mouse, and gamepad input.

mod gamepad;
mod input_handler;
mod input_mapping;
mod keyboard;
mod mouse;
mod thread_safe;

pub mod prelude;

// Re-export for convenience
pub use gamepad::*;
pub use input_handler::*;
pub use input_mapping::*;
pub use keyboard::*;
pub use mouse::*;
pub use thread_safe::*;

// Re-export input events
pub use input_handler::InputEvent;

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

    #[error("Thread-safe input error: {0}")]
    ThreadError(#[from] InputThreadError),
}
