//! Input mapping system for binding inputs to game actions.

use std::collections::HashMap;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;
use crate::gamepad::GamepadButton;

/// Represents different types of input sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    /// Keyboard key
    Keyboard(KeyCode),
    /// Mouse button
    Mouse(MouseButton),
    /// Gamepad button (with gamepad ID)
    Gamepad(u32, GamepadButton),
}

/// Represents a game action that can be bound to inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    /// Movement actions
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    /// Action buttons
    Action1, // Typically primary action (e.g., A button, Space key)
    Action2, // Typically secondary action (e.g., B button, Enter key)
    Action3, // Typically tertiary action (e.g., X button, Shift key)
    Action4, // Typically quaternary action (e.g., Y button, Ctrl key)
    /// UI actions
    Menu,
    Cancel,
    Select,
    /// Custom action with ID
    Custom(u32),
}

/// Input mapping configuration
#[derive(Debug, Clone)]
pub struct InputMapping {
    /// Maps input sources to game actions
    bindings: HashMap<InputSource, GameAction>,
    /// Maps game actions to input sources (for reverse lookup)
    action_bindings: HashMap<GameAction, Vec<InputSource>>,
}

impl Default for InputMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl InputMapping {
    /// Create a new input mapping with default bindings
    pub fn new() -> Self {
        let mut mapping = Self {
            bindings: HashMap::new(),
            action_bindings: HashMap::new(),
        };
        
        // Set up default bindings
        mapping.set_default_bindings();
        mapping
    }

    /// Set default input bindings for common actions
    pub fn set_default_bindings(&mut self) {
        // Movement
        self.bind_input(InputSource::Keyboard(KeyCode::KeyW), GameAction::MoveUp);
        self.bind_input(InputSource::Keyboard(KeyCode::ArrowUp), GameAction::MoveUp);
        self.bind_input(InputSource::Keyboard(KeyCode::KeyS), GameAction::MoveDown);
        self.bind_input(InputSource::Keyboard(KeyCode::ArrowDown), GameAction::MoveDown);
        self.bind_input(InputSource::Keyboard(KeyCode::KeyA), GameAction::MoveLeft);
        self.bind_input(InputSource::Keyboard(KeyCode::ArrowLeft), GameAction::MoveLeft);
        self.bind_input(InputSource::Keyboard(KeyCode::KeyD), GameAction::MoveRight);
        self.bind_input(InputSource::Keyboard(KeyCode::ArrowRight), GameAction::MoveRight);

        // Actions
        self.bind_input(InputSource::Keyboard(KeyCode::Space), GameAction::Action1);
        self.bind_input(InputSource::Mouse(MouseButton::Left), GameAction::Action1);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::A), GameAction::Action1);
        
        self.bind_input(InputSource::Keyboard(KeyCode::Enter), GameAction::Action2);
        self.bind_input(InputSource::Mouse(MouseButton::Right), GameAction::Action2);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::B), GameAction::Action2);
        
        self.bind_input(InputSource::Keyboard(KeyCode::ShiftLeft), GameAction::Action3);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::X), GameAction::Action3);
        
        self.bind_input(InputSource::Keyboard(KeyCode::ControlLeft), GameAction::Action4);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::Y), GameAction::Action4);

        // UI
        self.bind_input(InputSource::Keyboard(KeyCode::Escape), GameAction::Menu);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::Start), GameAction::Menu);
        
        self.bind_input(InputSource::Keyboard(KeyCode::Tab), GameAction::Select);
        self.bind_input(InputSource::Gamepad(0, GamepadButton::Select), GameAction::Select);
    }

    /// Bind an input source to a game action
    pub fn bind_input(&mut self, input: InputSource, action: GameAction) {
        // Remove any existing binding for this input
        if let Some(old_action) = self.bindings.remove(&input) {
            if let Some(sources) = self.action_bindings.get_mut(&old_action) {
                sources.retain(|&s| s != input);
                // If the action has no more bindings, remove it entirely
                if sources.is_empty() {
                    self.action_bindings.remove(&old_action);
                }
            }
        }

        // Add the new binding
        self.bindings.insert(input, action);
        self.action_bindings.entry(action).or_default().push(input);
    }

    /// Bind an input source to multiple game actions (allows one input to trigger multiple actions)
    pub fn bind_input_to_multiple_actions(&mut self, input: InputSource, actions: Vec<GameAction>) {
        // Remove any existing binding for this input
        if let Some(old_action) = self.bindings.remove(&input) {
            if let Some(sources) = self.action_bindings.get_mut(&old_action) {
                sources.retain(|&s| s != input);
            }
        }

        // Bind to the first action for the reverse lookup
        if let Some(first_action) = actions.first() {
            self.bindings.insert(input, *first_action);
        }

        // Add bindings for all actions
        for action in actions {
            self.action_bindings.entry(action).or_default().push(input);
        }
    }

    /// Unbind an input source
    pub fn unbind_input(&mut self, input: &InputSource) {
        if let Some(action) = self.bindings.remove(input) {
            if let Some(sources) = self.action_bindings.get_mut(&action) {
                sources.retain(|&s| &s != input);
                // If the action has no more bindings, remove it entirely
                if sources.is_empty() {
                    self.action_bindings.remove(&action);
                }
            }
        }
    }

    /// Unbind all inputs for a specific action
    pub fn unbind_action(&mut self, action: &GameAction) {
        if let Some(sources) = self.action_bindings.remove(action) {
            for source in sources {
                self.bindings.remove(&source);
            }
        }
    }

    /// Get the action bound to an input source
    pub fn get_action(&self, input: &InputSource) -> Option<&GameAction> {
        self.bindings.get(input)
    }

    /// Get all input sources bound to an action
    pub fn get_bindings(&self, action: &GameAction) -> &[InputSource] {
        self.action_bindings.get(action).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if an action is currently bound to any input
    pub fn has_binding(&self, action: &GameAction) -> bool {
        self.action_bindings.contains_key(action)
    }

    /// Clear all bindings
    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
        self.action_bindings.clear();
    }
}