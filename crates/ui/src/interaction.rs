//! Widget interaction and state management.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use glam::Vec2;
use input::prelude::{InputHandler, MouseButton};

use crate::Rect;

/// Unique identifier for a widget.
/// Can be created from strings, integers, or tuples for hierarchical IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a widget ID from a hash value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Create a widget ID from a string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Create a widget ID from a string and index (for lists).
    pub fn from_str_index(s: &str, index: usize) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        index.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Get the raw ID value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<&str> for WidgetId {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<u64> for WidgetId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<(&str, usize)> for WidgetId {
    fn from((s, index): (&str, usize)) -> Self {
        Self::from_str_index(s, index)
    }
}

/// State of a widget in the current frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    /// Widget is not interacted with
    Normal,
    /// Mouse is hovering over the widget
    Hovered,
    /// Widget is being pressed/dragged
    Active,
    /// Widget is disabled and cannot be interacted with
    Disabled,
}

/// Result of a widget interaction.
#[derive(Debug, Clone, Copy)]
pub struct InteractionResult {
    /// Current state of the widget
    pub state: WidgetState,
    /// True if the widget was clicked (mouse released over it while active)
    pub clicked: bool,
    /// True if the widget is currently being dragged
    pub dragging: bool,
    /// Mouse position relative to widget bounds
    pub local_mouse: Vec2,
}

impl Default for InteractionResult {
    fn default() -> Self {
        Self {
            state: WidgetState::Normal,
            clicked: false,
            dragging: false,
            local_mouse: Vec2::ZERO,
        }
    }
}

/// Persistent state for widgets that need to track data across frames.
#[derive(Debug, Clone, Default)]
pub struct WidgetPersistentState {
    /// Whether the widget was seen this frame (for garbage collection)
    pub seen_this_frame: bool,
    /// Custom float value (e.g., slider position, scroll offset)
    pub float_value: f32,
    /// Custom boolean value (e.g., checkbox state, expanded state)
    pub bool_value: bool,
    /// Custom string value (e.g., text input content)
    pub string_value: String,
}

/// Input state snapshot for UI interaction.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current mouse position in screen coordinates
    pub mouse_pos: Vec2,
    /// Whether left mouse button is pressed
    pub mouse_down: bool,
    /// Whether left mouse button was just pressed this frame
    pub mouse_just_pressed: bool,
    /// Whether left mouse button was just released this frame
    pub mouse_just_released: bool,
    /// Mouse scroll delta
    pub scroll_delta: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_pos: Vec2::ZERO,
            mouse_down: false,
            mouse_just_pressed: false,
            mouse_just_released: false,
            scroll_delta: 0.0,
        }
    }
}

impl InputState {
    /// Create input state from an InputHandler.
    pub fn from_input_handler(input: &InputHandler) -> Self {
        let mouse = input.mouse();
        let pos = mouse.position();
        Self {
            mouse_pos: Vec2::new(pos.x, pos.y),
            mouse_down: mouse.is_button_pressed(MouseButton::Left),
            mouse_just_pressed: mouse.is_button_just_pressed(MouseButton::Left),
            mouse_just_released: mouse.is_button_just_released(MouseButton::Left),
            scroll_delta: mouse.wheel_delta(),
        }
    }
}

/// Tracks interaction state for all widgets in the UI.
pub struct InteractionManager {
    /// Currently hot widget (mouse hovering)
    hot_widget: Option<WidgetId>,
    /// Currently active widget (being pressed/dragged)
    active_widget: Option<WidgetId>,
    /// Input state snapshot for this frame
    input: InputState,
    /// Persistent state storage for widgets
    persistent_state: HashMap<WidgetId, WidgetPersistentState>,
    /// Widget that had keyboard focus
    focus_widget: Option<WidgetId>,
}

impl Default for InteractionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionManager {
    /// Create a new interaction manager.
    pub fn new() -> Self {
        Self {
            hot_widget: None,
            active_widget: None,
            input: InputState::default(),
            persistent_state: HashMap::new(),
            focus_widget: None,
        }
    }

    /// Begin a new frame, updating input state.
    pub fn begin_frame(&mut self, input: &InputHandler) {
        self.input = InputState::from_input_handler(input);

        // Clear hot widget at start of frame (will be set by widgets that are hovered)
        self.hot_widget = None;

        // Handle mouse release - deactivate widget
        if self.input.mouse_just_released {
            self.active_widget = None;
        }

        // Mark all persistent state as not seen
        for state in self.persistent_state.values_mut() {
            state.seen_this_frame = false;
        }
    }

    /// End a frame, cleaning up stale state.
    pub fn end_frame(&mut self) {
        // Garbage collect persistent state for widgets not seen this frame
        // Only remove after several frames of not being seen (allows for animation, etc.)
        self.persistent_state.retain(|_, state| state.seen_this_frame);
    }

    /// Get the current input state.
    pub fn input(&self) -> &InputState {
        &self.input
    }

    /// Get the current mouse position.
    pub fn mouse_pos(&self) -> Vec2 {
        self.input.mouse_pos
    }

    /// Check if a widget is the hot (hovered) widget.
    pub fn is_hot(&self, id: WidgetId) -> bool {
        self.hot_widget == Some(id)
    }

    /// Check if a widget is the active (pressed/dragged) widget.
    pub fn is_active(&self, id: WidgetId) -> bool {
        self.active_widget == Some(id)
    }

    /// Check if a widget has keyboard focus.
    pub fn is_focused(&self, id: WidgetId) -> bool {
        self.focus_widget == Some(id)
    }

    /// Set keyboard focus to a widget.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focus_widget = Some(id);
    }

    /// Clear keyboard focus.
    pub fn clear_focus(&mut self) {
        self.focus_widget = None;
    }

    /// Get persistent state for a widget, creating default if not present.
    pub fn get_state(&mut self, id: WidgetId) -> &mut WidgetPersistentState {
        let state = self.persistent_state.entry(id).or_default();
        state.seen_this_frame = true;
        state
    }

    /// Get persistent state for a widget if it exists.
    pub fn get_state_if_exists(&self, id: WidgetId) -> Option<&WidgetPersistentState> {
        self.persistent_state.get(&id)
    }

    /// Process interaction for a widget.
    pub fn interact(&mut self, id: WidgetId, bounds: Rect, enabled: bool) -> InteractionResult {
        // Mark state as seen
        self.get_state(id).seen_this_frame = true;

        if !enabled {
            return InteractionResult {
                state: WidgetState::Disabled,
                ..Default::default()
            };
        }

        let mouse_in_bounds = bounds.contains(self.input.mouse_pos);
        let local_mouse = self.input.mouse_pos - bounds.position();

        // Check if this widget should become active
        if mouse_in_bounds && self.input.mouse_just_pressed && self.active_widget.is_none() {
            self.active_widget = Some(id);
        }

        // Update hot widget
        if mouse_in_bounds && self.active_widget.is_none() {
            self.hot_widget = Some(id);
        }

        // Determine state and interactions
        let is_active = self.active_widget == Some(id);
        let is_hot = mouse_in_bounds;

        // Click happens when mouse is released while active AND still over the widget
        let clicked = is_active && self.input.mouse_just_released && mouse_in_bounds;

        let state = if is_active {
            WidgetState::Active
        } else if is_hot {
            WidgetState::Hovered
        } else {
            WidgetState::Normal
        };

        InteractionResult {
            state,
            clicked,
            dragging: is_active && self.input.mouse_down,
            local_mouse,
        }
    }

    /// Process interaction for a draggable widget (like a slider thumb).
    /// Returns the drag delta if dragging.
    pub fn interact_draggable(&mut self, id: WidgetId, bounds: Rect, enabled: bool) -> (InteractionResult, Option<Vec2>) {
        let result = self.interact(id, bounds, enabled);

        let drag_delta = if result.dragging {
            // Calculate drag delta from previous frame position
            // For now, just use the current mouse position relative to bounds center
            Some(self.input.mouse_pos - bounds.center())
        } else {
            None
        };

        (result, drag_delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id_from_str() {
        let id1 = WidgetId::from_str("button_1");
        let id2 = WidgetId::from_str("button_1");
        let id3 = WidgetId::from_str("button_2");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_widget_id_from_str_index() {
        let id1 = WidgetId::from_str_index("item", 0);
        let id2 = WidgetId::from_str_index("item", 1);
        let id3 = WidgetId::from_str_index("item", 0);

        assert_ne!(id1, id2);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_widget_id_conversions() {
        let id1: WidgetId = "test".into();
        let id2: WidgetId = WidgetId::from_str("test");
        assert_eq!(id1, id2);

        let id3: WidgetId = 12345u64.into();
        assert_eq!(id3.value(), 12345);

        let id4: WidgetId = ("list", 5).into();
        let id5 = WidgetId::from_str_index("list", 5);
        assert_eq!(id4, id5);
    }

    #[test]
    fn test_interaction_manager_new() {
        let manager = InteractionManager::new();
        assert!(manager.hot_widget.is_none());
        assert!(manager.active_widget.is_none());
        assert!(manager.focus_widget.is_none());
    }

    #[test]
    fn test_interaction_manager_state() {
        let mut manager = InteractionManager::new();
        let id = WidgetId::from_str("test_widget");

        let state = manager.get_state(id);
        state.float_value = 42.0;
        state.bool_value = true;
        state.string_value = "hello".to_string();

        let state = manager.get_state_if_exists(id).unwrap();
        assert_eq!(state.float_value, 42.0);
        assert!(state.bool_value);
        assert_eq!(state.string_value, "hello");
    }

    #[test]
    fn test_interaction_result_default() {
        let result = InteractionResult::default();
        assert_eq!(result.state, WidgetState::Normal);
        assert!(!result.clicked);
        assert!(!result.dragging);
    }

    #[test]
    fn test_input_state_default() {
        let input = InputState::default();
        assert_eq!(input.mouse_pos, Vec2::ZERO);
        assert!(!input.mouse_down);
        assert!(!input.mouse_just_pressed);
        assert!(!input.mouse_just_released);
    }
}
