//! Thread-safe wrapper for input handling.

use crate::{
    input_handler::{InputHandler, InputEvent},
    input_mapping::GameAction,
    keyboard::KeyboardState,
    mouse::MouseState,
    gamepad::GamepadManager,
};
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper for InputHandler
#[derive(Debug, Clone)]
pub struct ThreadSafeInputHandler {
    inner: Arc<Mutex<InputHandler>>,
}

impl Default for ThreadSafeInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadSafeInputHandler {
    /// Create a new thread-safe input handler
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InputHandler::new())),
        }
    }

    /// Helper function to lock the inner input handler
    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, InputHandler>, InputThreadError> {
        self.inner.lock()
            .map_err(|e| InputThreadError::LockError(format!("Mutex poisoned: {}", e)))
    }

    /// Queue an input event for processing (thread-safe)
    pub fn queue_event(&self, event: InputEvent) -> Result<(), InputThreadError> {
        let mut handler = self.lock_inner()?;
        handler.queue_event(event);
        Ok(())
    }

    /// Handle a window event by queuing it for processing (thread-safe)
    pub fn handle_window_event(&self, event: &winit::event::WindowEvent) -> Result<(), InputThreadError> {
        let mut handler = self.lock_inner()?;
        handler.handle_window_event(event);
        Ok(())
    }

    /// Process all queued input events (thread-safe)
    pub fn process_queued_events(&self) -> Result<(), InputThreadError> {
        let mut handler = self.lock_inner()?;
        handler.process_queued_events();
        Ok(())
    }

    /// Update all input states for the next frame (thread-safe)
    pub fn update(&self) -> Result<(), InputThreadError> {
        let mut handler = self.lock_inner()?;
        handler.update();
        Ok(())
    }

    /// Check if a specific key is currently pressed (thread-safe)
    pub fn is_key_pressed(&self, key: winit::keyboard::KeyCode) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_key_pressed(key))
    }

    /// Check if a specific key was just pressed this frame (thread-safe)
    pub fn is_key_just_pressed(&self, key: winit::keyboard::KeyCode) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_key_just_pressed(key))
    }

    /// Check if a specific key was just released this frame (thread-safe)
    pub fn is_key_just_released(&self, key: winit::keyboard::KeyCode) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_key_just_released(key))
    }

    /// Check if a mouse button is currently pressed (thread-safe)
    pub fn is_mouse_button_pressed(&self, button: winit::event::MouseButton) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_mouse_button_pressed(button))
    }

    /// Check if a mouse button was just pressed this frame (thread-safe)
    pub fn is_mouse_button_just_pressed(&self, button: winit::event::MouseButton) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_mouse_button_just_pressed(button))
    }

    /// Get current mouse position (thread-safe)
    pub fn mouse_position(&self) -> Result<crate::mouse::MousePosition, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.mouse_position())
    }

    /// Get mouse movement delta since last frame (thread-safe)
    pub fn mouse_movement_delta(&self) -> Result<(f32, f32), InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.mouse_movement_delta())
    }

    /// Get mouse wheel scroll delta (thread-safe)
    pub fn mouse_wheel_delta(&self) -> Result<f32, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.mouse_wheel_delta())
    }

    /// Check if a game action is currently active (thread-safe)
    pub fn is_action_active(&self, action: &GameAction) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_action_active(action))
    }

    /// Check if a game action was just activated this frame (thread-safe)
    pub fn is_action_just_activated(&self, action: &GameAction) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_action_just_activated(action))
    }

    /// Check if a game action was just deactivated this frame (thread-safe)
    pub fn is_action_just_deactivated(&self, action: &GameAction) -> Result<bool, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.is_action_just_deactivated(action))
    }

    /// Get a copy of the current keyboard state (thread-safe)
    pub fn keyboard_state(&self) -> Result<KeyboardState, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.keyboard().clone())
    }

    /// Get a copy of the current mouse state (thread-safe)
    pub fn mouse_state(&self) -> Result<MouseState, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.mouse().clone())
    }

    /// Get a copy of the current gamepad manager (thread-safe)
    pub fn gamepad_manager(&self) -> Result<GamepadManager, InputThreadError> {
        let handler = self.lock_inner()?;
        Ok(handler.gamepads().clone())
    }
}

/// Errors that can occur in the thread-safe input system
#[derive(Debug, thiserror::Error)]
pub enum InputThreadError {
    #[error("Failed to lock input handler: {0}")]
    LockError(String),

    #[error("Input operation error: {0}")]
    OperationError(String),
}