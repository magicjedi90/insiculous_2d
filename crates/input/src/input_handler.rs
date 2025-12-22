//! Unified input handling.

use crate::gamepad::GamepadManager;
use crate::keyboard::{KeyboardState, convert_physical_key};
use crate::mouse::MouseState;
use winit::event::{WindowEvent, ElementState};

/// A unified handler for all input types
#[derive(Debug, Default)]
pub struct InputHandler {
    /// Keyboard state
    keyboard: KeyboardState,
    /// Mouse state
    mouse: MouseState,
    /// Gamepad manager
    gamepads: GamepadManager,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            gamepads: GamepadManager::new(),
        }
    }

    /// Process a window event and update input state
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key_code) = convert_physical_key(event.physical_key) {
                    match event.state {
                        ElementState::Pressed => {
                            self.keyboard.handle_key_press(key_code);
                        }
                        ElementState::Released => {
                            self.keyboard.handle_key_release(key_code);
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        self.mouse.handle_button_press(*button);
                    }
                    ElementState::Released => {
                        self.mouse.handle_button_release(*button);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.update_position(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Convert scroll delta to a simple float
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                self.mouse.update_wheel_delta(scroll_delta);
            }
            _ => {
                // Other events can be ignored for now
            }
        }
    }

    /// Update all input states for the next frame (clears just pressed/released states)
    pub fn update(&mut self) {
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