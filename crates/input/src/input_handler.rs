//! Unified input handling.

use crate::gamepad::GamepadManager;
use crate::keyboard::{KeyboardState, convert_physical_key};
use crate::mouse::MouseState;
use crate::input_mapping::{InputMapping, InputSource, GameAction};
use winit::event::{WindowEvent, ElementState};
use std::collections::VecDeque;

/// Input events that can be queued for processing
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

    /// Check if a game action is currently active (any bound input is pressed)
    pub fn is_action_active(&self, action: &GameAction) -> bool {
        let bindings = self.input_mapping.get_bindings(action);
        for binding in bindings {
            match binding {
                InputSource::Keyboard(key) => {
                    if self.keyboard.is_key_pressed(*key) {
                        return true;
                    }
                }
                InputSource::Mouse(button) => {
                    if self.mouse.is_button_pressed(*button) {
                        return true;
                    }
                }
                InputSource::Gamepad(id, button) => {
                    if let Some(gamepad) = self.gamepads.get_gamepad(*id) {
                        if gamepad.is_button_pressed(*button) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a game action was just activated this frame
    pub fn is_action_just_activated(&self, action: &GameAction) -> bool {
        let bindings = self.input_mapping.get_bindings(action);
        for binding in bindings {
            match binding {
                InputSource::Keyboard(key) => {
                    if self.keyboard.is_key_just_pressed(*key) {
                        return true;
                    }
                }
                InputSource::Mouse(button) => {
                    if self.mouse.is_button_just_pressed(*button) {
                        return true;
                    }
                }
                InputSource::Gamepad(id, button) => {
                    if let Some(gamepad) = self.gamepads.get_gamepad(*id) {
                        if gamepad.is_button_just_pressed(*button) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a game action was just deactivated this frame
    pub fn is_action_just_deactivated(&self, action: &GameAction) -> bool {
        let bindings = self.input_mapping.get_bindings(action);
        for binding in bindings {
            match binding {
                InputSource::Keyboard(key) => {
                    if self.keyboard.is_key_just_released(*key) {
                        return true;
                    }
                }
                InputSource::Mouse(button) => {
                    if self.mouse.is_button_just_released(*button) {
                        return true;
                    }
                }
                InputSource::Gamepad(id, button) => {
                    if let Some(gamepad) = self.gamepads.get_gamepad(*id) {
                        if gamepad.is_button_just_released(*button) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Queue an input event for later processing
    pub fn queue_event(&mut self, event: InputEvent) {
        self.event_queue.push_back(event);
    }

    /// Process all queued input events
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

    /// Update all input states for the next frame (processes queued events then clears just pressed/released states)
    pub fn update(&mut self) {
        // Process any queued input events first
        self.process_queued_events();

        // Then update all input states (clears just pressed/released states)
        self.keyboard.update();
        self.mouse.update();
        self.gamepads.update();
    }

    /// End the frame by clearing just pressed/released states
    ///
    /// Call this at the end of each frame after game logic has run.
    /// This clears the "just pressed" and "just released" flags without processing events.
    /// Use this when you've already called `process_queued_events()` earlier in the frame.
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