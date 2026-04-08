//! Input mapping system for binding inputs to game actions.
//!
//! # Binding Model
//!
//! The input mapping system supports two lookup directions:
//!
//! - **Action → Inputs**: "What inputs trigger this action?" (`get_bindings()`)
//! - **Input → Action**: "What action does this input trigger?" (`get_action()`)
//!
//! ## One-to-Many: Multiple Inputs per Action (Recommended)
//!
//! The common case is binding multiple inputs to a single action:
//! ```ignore
//! // Both W and Up arrow trigger MoveUp
//! mapping.bind_input(InputSource::Keyboard(KeyCode::KeyW), GameAction::MoveUp);
//! mapping.bind_input(InputSource::Keyboard(KeyCode::ArrowUp), GameAction::MoveUp);
//!
//! // get_bindings returns both inputs
//! mapping.get_bindings(&GameAction::MoveUp); // [KeyW, ArrowUp]
//! ```
//!
//! ## One-to-Many: Multiple Actions per Input (Advanced)
//!
//! You can bind one input to multiple actions via `bind_input_to_multiple_actions()`.
//! **Note**: The reverse lookup `get_action()` only returns the *first* action:
//!
//! ```ignore
//! // Space triggers both Jump and Confirm
//! mapping.bind_input_to_multiple_actions(
//!     InputSource::Keyboard(KeyCode::Space),
//!     vec![GameAction::Action1, GameAction::Select]
//! );
//!
//! // Forward lookup: Both actions recognize Space as a trigger
//! mapping.get_bindings(&GameAction::Action1); // [Space]
//! mapping.get_bindings(&GameAction::Select);  // [Space]
//!
//! // Reverse lookup: Only returns first action
//! mapping.get_action(&InputSource::Keyboard(KeyCode::Space)); // Some(Action1)
//! ```
//!
//! For most games, checking `is_action_active()` is preferred over `get_action()`.

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

    /// Remove an existing binding for the given input source, cleaning up action_bindings.
    fn remove_existing_binding(&mut self, input: &InputSource) {
        if let Some(old_action) = self.bindings.remove(input) {
            if let Some(sources) = self.action_bindings.get_mut(&old_action) {
                sources.retain(|&s| s != *input);
                if sources.is_empty() {
                    self.action_bindings.remove(&old_action);
                }
            }
        }
    }

    /// Bind an input source to a game action
    pub fn bind_input(&mut self, input: InputSource, action: GameAction) {
        // Remove any existing binding for this input
        self.remove_existing_binding(&input);

        // Add the new binding
        self.bindings.insert(input, action);
        self.action_bindings.entry(action).or_default().push(input);
    }

    /// Bind an input source to multiple game actions.
    ///
    /// This allows one input to trigger multiple actions simultaneously.
    /// For example, you might want the Space key to trigger both "Jump" and "Confirm".
    ///
    /// # Lookup Behavior
    ///
    /// - `get_bindings(action)` will return the input for ALL specified actions
    /// - `get_action(input)` will only return the **first** action in the list
    ///
    /// This asymmetry exists because the reverse lookup uses a single-value HashMap.
    /// For most use cases, you should check actions via `InputHandler::is_action_active()`
    /// rather than using `get_action()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// mapping.bind_input_to_multiple_actions(
    ///     InputSource::Keyboard(KeyCode::Space),
    ///     vec![GameAction::Action1, GameAction::Select]
    /// );
    ///
    /// // Both actions now respond to Space
    /// assert!(input.is_action_active(&GameAction::Action1)); // true when Space pressed
    /// assert!(input.is_action_active(&GameAction::Select));  // true when Space pressed
    /// ```
    pub fn bind_input_to_multiple_actions(&mut self, input: InputSource, actions: Vec<GameAction>) {
        // Remove any existing binding for this input
        self.remove_existing_binding(&input);

        // Bind to the first action for the reverse lookup (limitation: only first action returned by get_action)
        if let Some(first_action) = actions.first() {
            self.bindings.insert(input, *first_action);
        }

        // Add bindings for all actions (all actions will respond to this input)
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

    /// Get the action bound to an input source.
    ///
    /// **Note**: If the input was bound to multiple actions via
    /// `bind_input_to_multiple_actions()`, this only returns the *first* action.
    /// For checking if an action is triggered, prefer `InputHandler::is_action_active()`.
    pub fn get_action(&self, input: &InputSource) -> Option<&GameAction> {
        self.bindings.get(input)
    }

    /// Get all input sources bound to an action.
    ///
    /// This is the recommended way to query bindings, as it correctly returns
    /// all inputs that trigger the action, including those bound via
    /// `bind_input_to_multiple_actions()`.
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