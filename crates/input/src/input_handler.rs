//! Unified input handling.
//!
//! # Frame Lifecycle
//!
//! The input system follows a specific frame lifecycle:
//!
//! 1. **Event Collection** (automatic): Window events are queued via `handle_window_event()`
//! 2. **Event Processing** (start of frame): Call `process_queued_events()` to update input state
//! 3. **Game Logic**: Read input state via `is_key_pressed()`, `is_action_active()`, etc.
//! 4. **State Reset** (end of frame): Call `end_frame()` to clear just_pressed/just_released flags
//!
//! ```ignore
//! // Typical frame loop:
//! input.process_queued_events();  // Process this frame's input
//! // ... game logic reads input state ...
//! input.end_frame();              // Clear just_pressed/just_released for next frame
//! ```
//!
//! For simple use cases, `update()` combines steps 2 and 4 into one call.

use crate::gamepad::GamepadManager;
use crate::keyboard::{KeyboardState, convert_physical_key};
use crate::mouse::MouseState;
use crate::input_mapping::{InputMapping, InputSource, GameAction};
use winit::event::{WindowEvent, ElementState};
use std::collections::VecDeque;

/// Input events that can be queued for processing.
///
/// # Winit Coupling
///
/// This enum uses [`winit::keyboard::KeyCode`] and [`winit::event::MouseButton`] directly
/// rather than defining custom key/button types. This is an intentional design choice:
///
/// - **Winit is the standard** for Rust windowing and is unlikely to be replaced
/// - **Reduces mapping overhead** - no conversion layer needed between winit and internal types
/// - **Full compatibility** - all winit key codes and mouse buttons are supported automatically
/// - **Simpler codebase** - fewer types to maintain
///
/// If abstraction becomes necessary (e.g., for non-winit platforms), the conversion can
/// be added at the boundary in [`InputHandler::handle_window_event`] without changing
/// the public API.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Keyboard key pressed
    KeyPressed(winit::keyboard::KeyCode),
    /// Keyboard key released
    KeyReleased(winit::keyboard::KeyCode),
    /// Mouse button pressed
    MouseButtonPressed(winit::event::MouseButton),
    /// Mouse button released
    MouseButtonReleased(winit::event::MouseButton),
    /// Mouse moved to position
    MouseMoved(f32, f32),
    /// Mouse wheel scrolled
    MouseWheelScrolled(f32),
    /// Gamepad button pressed
    GamepadButtonPressed(u32, crate::gamepad::GamepadButton),
    /// Gamepad button released
    GamepadButtonReleased(u32, crate::gamepad::GamepadButton),
    /// Gamepad axis updated
    GamepadAxisUpdated(u32, crate::gamepad::GamepadAxis, f32),
}

/// A unified handler for all input types
#[derive(Debug, Default)]
pub struct InputHandler {
    /// Keyboard state
    keyboard: KeyboardState,
    /// Mouse state
    mouse: MouseState,
    /// Gamepad manager
    gamepads: GamepadManager,
    /// Event queue for buffering input events
    event_queue: VecDeque<InputEvent>,
    /// Input mapping configuration
    input_mapping: InputMapping,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            gamepads: GamepadManager::new(),
            event_queue: VecDeque::new(),
            input_mapping: InputMapping::new(),
        }
    }

    /// Get a reference to the input mapping
    pub fn input_mapping(&self) -> &InputMapping {
        &self.input_mapping
    }

    /// Get a mutable reference to the input mapping
    pub fn input_mapping_mut(&mut self) -> &mut InputMapping {
        &mut self.input_mapping
    }

    // ================== Input Source Helpers ==================

    /// Check if an input source is currently pressed
    fn is_input_pressed(&self, source: &InputSource) -> bool {
        match source {
            InputSource::Keyboard(key) => self.keyboard.is_key_pressed(*key),
            InputSource::Mouse(button) => self.mouse.is_button_pressed(*button),
            InputSource::Gamepad(id, button) => self
                .gamepads
                .get_gamepad(*id)
                .map_or(false, |g| g.is_button_pressed(*button)),
        }
    }

    /// Check if an input source was just pressed this frame
    fn is_input_just_pressed(&self, source: &InputSource) -> bool {
        match source {
            InputSource::Keyboard(key) => self.keyboard.is_key_just_pressed(*key),
            InputSource::Mouse(button) => self.mouse.is_button_just_pressed(*button),
            InputSource::Gamepad(id, button) => self
                .gamepads
                .get_gamepad(*id)
                .map_or(false, |g| g.is_button_just_pressed(*button)),
        }
    }

    /// Check if an input source was just released this frame
    fn is_input_just_released(&self, source: &InputSource) -> bool {
        match source {
            InputSource::Keyboard(key) => self.keyboard.is_key_just_released(*key),
            InputSource::Mouse(button) => self.mouse.is_button_just_released(*button),
            InputSource::Gamepad(id, button) => self
                .gamepads
                .get_gamepad(*id)
                .map_or(false, |g| g.is_button_just_released(*button)),
        }
    }

    // ================== Action Checking ==================

    /// Check if a game action is currently active (any bound input is pressed)
    pub fn is_action_active(&self, action: &GameAction) -> bool {
        self.input_mapping
            .get_bindings(action)
            .iter()
            .any(|source| self.is_input_pressed(source))
    }

    /// Check if a game action was just activated this frame
    pub fn is_action_just_activated(&self, action: &GameAction) -> bool {
        self.input_mapping
            .get_bindings(action)
            .iter()
            .any(|source| self.is_input_just_pressed(source))
    }

    /// Check if a game action was just deactivated this frame
    pub fn is_action_just_deactivated(&self, action: &GameAction) -> bool {
        self.input_mapping
            .get_bindings(action)
            .iter()
            .any(|source| self.is_input_just_released(source))
    }

    /// Queue an input event for later processing
    pub fn queue_event(&mut self, event: InputEvent) {
        self.event_queue.push_back(event);
    }

    /// Process all queued input events, updating input state.
    ///
    /// Call this at the **start** of each frame, before reading input state.
    /// This processes all events that were queued via `handle_window_event()`
    /// since the last call, updating keyboard, mouse, and gamepad state.
    ///
    /// After calling this, you can read current input state via methods like
    /// `is_key_pressed()`, `is_key_just_pressed()`, `is_action_active()`, etc.
    ///
    /// At the end of the frame, call `end_frame()` to reset the "just pressed"
    /// and "just released" flags for the next frame.
    pub fn process_queued_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            self.process_event(event);
        }
    }

    /// Process a single input event
    fn process_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key) => {
                self.keyboard.handle_key_press(key);
            }
            InputEvent::KeyReleased(key) => {
                self.keyboard.handle_key_release(key);
            }
            InputEvent::MouseButtonPressed(button) => {
                self.mouse.handle_button_press(button);
            }
            InputEvent::MouseButtonReleased(button) => {
                self.mouse.handle_button_release(button);
            }
            InputEvent::MouseMoved(x, y) => {
                self.mouse.update_position(x, y);
            }
            InputEvent::MouseWheelScrolled(delta) => {
                self.mouse.update_wheel_delta(delta);
            }
            InputEvent::GamepadButtonPressed(id, button) => {
                if let Some(gamepad) = self.gamepads.get_gamepad_mut(id) {
                    gamepad.handle_button_press(button);
                }
            }
            InputEvent::GamepadButtonReleased(id, button) => {
                if let Some(gamepad) = self.gamepads.get_gamepad_mut(id) {
                    gamepad.handle_button_release(button);
                }
            }
            InputEvent::GamepadAxisUpdated(id, axis, value) => {
                if let Some(gamepad) = self.gamepads.get_gamepad_mut(id) {
                    gamepad.update_axis(axis, value);
                }
            }
        }
    }

    /// Handle a window event by queuing it for later processing
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key_code) = convert_physical_key(event.physical_key) {
                    let input_event = match event.state {
                        ElementState::Pressed => InputEvent::KeyPressed(key_code),
                        ElementState::Released => InputEvent::KeyReleased(key_code),
                    };
                    self.queue_event(input_event);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let input_event = match state {
                    ElementState::Pressed => InputEvent::MouseButtonPressed(*button),
                    ElementState::Released => InputEvent::MouseButtonReleased(*button),
                };
                self.queue_event(input_event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.queue_event(InputEvent::MouseMoved(position.x as f32, position.y as f32));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Convert scroll delta to a simple float
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                self.queue_event(InputEvent::MouseWheelScrolled(scroll_delta));
            }
            _ => {
                // Other events can be ignored for now
            }
        }
    }

    /// Convenience method that processes events and resets state in one call.
    ///
    /// This is equivalent to calling `process_queued_events()` followed by `end_frame()`.
    /// Use this for simple applications where you don't need fine-grained control over
    /// when events are processed vs when state is reset.
    ///
    /// For most game loops, prefer the two-step approach:
    /// ```ignore
    /// input.process_queued_events();  // At start of frame
    /// // ... game logic ...
    /// input.end_frame();              // At end of frame
    /// ```
    pub fn update(&mut self) {
        // Process any queued input events first
        self.process_queued_events();

        // Then update all input states (clears just pressed/released states)
        self.keyboard.update();
        self.mouse.update();
        self.gamepads.update();
    }

    /// Reset input state for the next frame.
    ///
    /// Clears "just pressed" and "just released" flags from all input devices.
    /// Call this at the **end** of each frame, after all game logic has had a
    /// chance to check input state.
    ///
    /// # When to Use
    ///
    /// Use `end_frame()` when you've separately called `process_queued_events()` at
    /// the start of the frame. This is the recommended pattern for game loops:
    ///
    /// ```ignore
    /// // Start of frame
    /// input.process_queued_events();
    ///
    /// // Game logic - can check is_key_just_pressed(), etc.
    /// if input.is_key_just_pressed(KeyCode::Space) {
    ///     player.jump();
    /// }
    ///
    /// // End of frame - reset for next frame
    /// input.end_frame();
    /// ```
    pub fn end_frame(&mut self) {
        self.keyboard.update();
        self.mouse.update();
        self.gamepads.update();
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

    /// Check if a specific key is currently pressed
    pub fn is_key_pressed(&self, key: winit::keyboard::KeyCode) -> bool {
        self.keyboard.is_key_pressed(key)
    }

    /// Check if a specific key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: winit::keyboard::KeyCode) -> bool {
        self.keyboard.is_key_just_pressed(key)
    }

    /// Check if a specific key was just released this frame
    pub fn is_key_just_released(&self, key: winit::keyboard::KeyCode) -> bool {
        self.keyboard.is_key_just_released(key)
    }

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: winit::event::MouseButton) -> bool {
        self.mouse.is_button_pressed(button)
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_mouse_button_just_pressed(&self, button: winit::event::MouseButton) -> bool {
        self.mouse.is_button_just_pressed(button)
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> crate::mouse::MousePosition {
        self.mouse.position()
    }

    /// Get mouse movement delta since last frame
    pub fn mouse_movement_delta(&self) -> (f32, f32) {
        self.mouse.movement_delta()
    }

    /// Get mouse wheel scroll delta
    pub fn mouse_wheel_delta(&self) -> f32 {
        self.mouse.wheel_delta()
    }
}