//! Editor input mapping and action definitions.
//!
//! Defines editor-specific actions and provides default key bindings.
//! Uses the engine's input mapping system for configurable bindings.

use input::{InputHandler, InputSource};
use std::hash::Hash;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Editor-specific input actions.
///
/// These actions are used throughout the editor for navigation, selection,
/// and manipulation. Default bindings can be customized via EditorInputMapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorAction {
    // ================== Viewport Navigation ==================
    /// Pan the viewport (typically middle mouse or Space+drag)
    Pan,
    /// Zoom in
    ZoomIn,
    /// Zoom out
    ZoomOut,
    /// Focus camera on selection
    FocusSelection,
    /// Reset camera to origin
    ResetCamera,

    // ================== Selection ==================
    /// Primary select (click to select)
    Select,
    /// Add to selection modifier
    AddToSelection,
    /// Toggle selection modifier
    ToggleSelection,
    /// Select all entities
    SelectAll,
    /// Deselect all
    DeselectAll,

    // ================== Tools ==================
    /// Select tool (no gizmo)
    ToolSelect,
    /// Move/translate tool
    ToolMove,
    /// Rotate tool
    ToolRotate,
    /// Scale tool
    ToolScale,

    // ================== Edit ==================
    /// Delete selected entities
    Delete,
    /// Duplicate selected entities
    Duplicate,
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,
    /// Copy selection
    Copy,
    /// Paste copied entities
    Paste,
    /// Cut selection
    Cut,

    // ================== Grid ==================
    /// Toggle grid visibility
    ToggleGrid,
    /// Toggle snap to grid
    ToggleSnap,

    // ================== View ==================
    /// Toggle play mode
    TogglePlayMode,
}

/// Input state for editor actions.
#[derive(Debug, Clone, Default)]
pub struct EditorInputState {
    /// Whether pan modifier is active (Space key held)
    pub pan_modifier: bool,
    /// Whether add-to-selection modifier is active (Shift held)
    pub add_modifier: bool,
    /// Whether toggle-selection modifier is active (Ctrl held)
    pub toggle_modifier: bool,
    /// Current mouse position (screen coords)
    pub mouse_position: glam::Vec2,
    /// Mouse movement delta
    pub mouse_delta: glam::Vec2,
    /// Mouse scroll delta
    pub scroll_delta: f32,
    /// Primary mouse button (left) state
    pub primary_button: ButtonState,
    /// Secondary mouse button (right) state
    pub secondary_button: ButtonState,
    /// Middle mouse button state
    pub middle_button: ButtonState,
}

/// State of a mouse button.
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    /// Currently held down
    pub pressed: bool,
    /// Just pressed this frame
    pub just_pressed: bool,
    /// Just released this frame
    pub just_released: bool,
}

/// Manages editor input bindings and state.
#[derive(Debug)]
pub struct EditorInputMapping {
    /// Key bindings for actions
    bindings: std::collections::HashMap<EditorAction, Vec<InputSource>>,
}

impl Default for EditorInputMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorInputMapping {
    /// Create a new editor input mapping with default bindings.
    pub fn new() -> Self {
        let mut mapping = Self {
            bindings: std::collections::HashMap::new(),
        };
        mapping.set_default_bindings();
        mapping
    }

    /// Set default key bindings.
    fn set_default_bindings(&mut self) {
        // Viewport navigation
        self.bind(EditorAction::FocusSelection, InputSource::Keyboard(KeyCode::KeyF));
        self.bind(EditorAction::ResetCamera, InputSource::Keyboard(KeyCode::Home));
        self.bind(EditorAction::Pan, InputSource::Keyboard(KeyCode::Space));
        self.bind(EditorAction::Pan, InputSource::Mouse(MouseButton::Middle));

        // Selection modifiers
        self.bind(EditorAction::AddToSelection, InputSource::Keyboard(KeyCode::ShiftLeft));
        self.bind(EditorAction::AddToSelection, InputSource::Keyboard(KeyCode::ShiftRight));
        self.bind(EditorAction::ToggleSelection, InputSource::Keyboard(KeyCode::ControlLeft));
        self.bind(EditorAction::ToggleSelection, InputSource::Keyboard(KeyCode::ControlRight));
        self.bind(EditorAction::SelectAll, InputSource::Keyboard(KeyCode::KeyA)); // Ctrl+A handled separately
        self.bind(EditorAction::DeselectAll, InputSource::Keyboard(KeyCode::Escape));

        // Primary select
        self.bind(EditorAction::Select, InputSource::Mouse(MouseButton::Left));

        // Tools (Q, W, E, R like most editors)
        self.bind(EditorAction::ToolSelect, InputSource::Keyboard(KeyCode::KeyQ));
        self.bind(EditorAction::ToolMove, InputSource::Keyboard(KeyCode::KeyW));
        self.bind(EditorAction::ToolRotate, InputSource::Keyboard(KeyCode::KeyE));
        self.bind(EditorAction::ToolScale, InputSource::Keyboard(KeyCode::KeyR));

        // Edit operations
        self.bind(EditorAction::Delete, InputSource::Keyboard(KeyCode::Delete));
        self.bind(EditorAction::Delete, InputSource::Keyboard(KeyCode::Backspace));
        self.bind(EditorAction::Duplicate, InputSource::Keyboard(KeyCode::KeyD)); // Ctrl+D
        self.bind(EditorAction::Undo, InputSource::Keyboard(KeyCode::KeyZ)); // Ctrl+Z
        self.bind(EditorAction::Redo, InputSource::Keyboard(KeyCode::KeyY)); // Ctrl+Y or Ctrl+Shift+Z
        self.bind(EditorAction::Copy, InputSource::Keyboard(KeyCode::KeyC)); // Ctrl+C
        self.bind(EditorAction::Paste, InputSource::Keyboard(KeyCode::KeyV)); // Ctrl+V
        self.bind(EditorAction::Cut, InputSource::Keyboard(KeyCode::KeyX)); // Ctrl+X

        // Grid
        self.bind(EditorAction::ToggleGrid, InputSource::Keyboard(KeyCode::KeyG));
        self.bind(EditorAction::ToggleSnap, InputSource::Keyboard(KeyCode::KeyS)); // Ctrl+S handled separately for save

        // View
        self.bind(EditorAction::TogglePlayMode, InputSource::Keyboard(KeyCode::F5));
    }

    /// Bind an input source to an action.
    pub fn bind(&mut self, action: EditorAction, source: InputSource) {
        self.bindings.entry(action).or_default().push(source);
    }

    /// Remove all bindings for an action.
    pub fn unbind(&mut self, action: EditorAction) {
        self.bindings.remove(&action);
    }

    /// Clear a specific binding from an action.
    pub fn unbind_source(&mut self, action: EditorAction, source: &InputSource) {
        if let Some(sources) = self.bindings.get_mut(&action) {
            sources.retain(|s| s != source);
        }
    }

    /// Get all bindings for an action.
    pub fn get_bindings(&self, action: EditorAction) -> &[InputSource] {
        self.bindings.get(&action).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if any binding for an action satisfies the given predicates.
    fn check_action_with(
        &self,
        action: EditorAction,
        input: &InputHandler,
        key_check: impl Fn(&InputHandler, KeyCode) -> bool,
        mouse_check: impl Fn(&InputHandler, MouseButton) -> bool,
    ) -> bool {
        self.get_bindings(action).iter().any(|source| match source {
            InputSource::Keyboard(key) => key_check(input, *key),
            InputSource::Mouse(button) => mouse_check(input, *button),
            InputSource::Gamepad(_, _) => false,
        })
    }

    /// Check if an action is currently active (any bound input pressed).
    pub fn is_action_pressed(&self, action: EditorAction, input: &InputHandler) -> bool {
        self.check_action_with(
            action,
            input,
            |i, key| i.is_key_pressed(key),
            |i, btn| i.is_mouse_button_pressed(btn),
        )
    }

    /// Check if an action was just pressed this frame.
    pub fn is_action_just_pressed(&self, action: EditorAction, input: &InputHandler) -> bool {
        self.check_action_with(
            action,
            input,
            |i, key| i.is_key_just_pressed(key),
            |i, btn| i.is_mouse_button_just_pressed(btn),
        )
    }

    /// Check if an action was just released this frame.
    pub fn is_action_just_released(&self, action: EditorAction, input: &InputHandler) -> bool {
        self.check_action_with(
            action,
            input,
            |i, key| i.is_key_just_released(key),
            |i, btn| i.mouse().is_button_just_released(btn),
        )
    }

    /// Update input state from InputHandler.
    pub fn update_state(&self, input: &InputHandler) -> EditorInputState {
        let mouse_pos = input.mouse_position();
        let mouse_delta = input.mouse_movement_delta();

        EditorInputState {
            pan_modifier: self.is_action_pressed(EditorAction::Pan, input),
            add_modifier: self.is_action_pressed(EditorAction::AddToSelection, input),
            toggle_modifier: self.is_action_pressed(EditorAction::ToggleSelection, input),
            mouse_position: glam::Vec2::new(mouse_pos.x, mouse_pos.y),
            mouse_delta: glam::Vec2::new(mouse_delta.0, mouse_delta.1),
            scroll_delta: input.mouse_wheel_delta(),
            primary_button: ButtonState {
                pressed: input.is_mouse_button_pressed(MouseButton::Left),
                just_pressed: input.is_mouse_button_just_pressed(MouseButton::Left),
                just_released: input.mouse().is_button_just_released(MouseButton::Left),
            },
            secondary_button: ButtonState {
                pressed: input.is_mouse_button_pressed(MouseButton::Right),
                just_pressed: input.is_mouse_button_just_pressed(MouseButton::Right),
                just_released: input.mouse().is_button_just_released(MouseButton::Right),
            },
            middle_button: ButtonState {
                pressed: input.is_mouse_button_pressed(MouseButton::Middle),
                just_pressed: input.is_mouse_button_just_pressed(MouseButton::Middle),
                just_released: input.mouse().is_button_just_released(MouseButton::Middle),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let mapping = EditorInputMapping::new();

        // Check some default bindings exist
        assert!(!mapping.get_bindings(EditorAction::FocusSelection).is_empty());
        assert!(!mapping.get_bindings(EditorAction::ToolMove).is_empty());
        assert!(!mapping.get_bindings(EditorAction::Pan).is_empty());
    }

    #[test]
    fn test_bind_action() {
        let mut mapping = EditorInputMapping::new();
        let original_count = mapping.get_bindings(EditorAction::ZoomIn).len();

        mapping.bind(EditorAction::ZoomIn, InputSource::Keyboard(KeyCode::Equal));

        assert_eq!(
            mapping.get_bindings(EditorAction::ZoomIn).len(),
            original_count + 1
        );
    }

    #[test]
    fn test_unbind_action() {
        let mut mapping = EditorInputMapping::new();
        mapping.bind(EditorAction::ZoomIn, InputSource::Keyboard(KeyCode::Equal));

        mapping.unbind(EditorAction::ZoomIn);

        assert!(mapping.get_bindings(EditorAction::ZoomIn).is_empty());
    }

    #[test]
    fn test_pan_has_multiple_bindings() {
        let mapping = EditorInputMapping::new();
        let bindings = mapping.get_bindings(EditorAction::Pan);

        // Should have both Space key and Middle mouse
        assert!(bindings.len() >= 2);
    }
}
