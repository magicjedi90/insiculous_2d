//! Widget interaction and state management.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use glam::Vec2;
use input::prelude::{InputHandler, KeyCode, MouseButton};

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
    /// Characters typed this frame (for text input widgets)
    pub typed_chars: Vec<char>,
    /// Whether Enter/Return was just pressed
    pub enter_pressed: bool,
    /// Whether Escape was just pressed
    pub escape_pressed: bool,
    /// Whether Backspace was just pressed
    pub backspace_pressed: bool,
    /// Whether Tab was just pressed
    pub tab_pressed: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_pos: Vec2::ZERO,
            mouse_down: false,
            mouse_just_pressed: false,
            mouse_just_released: false,
            scroll_delta: 0.0,
            typed_chars: Vec::new(),
            enter_pressed: false,
            escape_pressed: false,
            backspace_pressed: false,
            tab_pressed: false,
        }
    }
}

/// Map a physical KeyCode to a character for text input.
/// Returns None for non-character keys. Only maps keys useful for numeric input.
fn keycode_to_char(key: KeyCode, shift: bool) -> Option<char> {
    use KeyCode::*;
    match key {
        // Numpad always maps to digits regardless of shift
        Numpad0 => Some('0'),
        Numpad1 => Some('1'),
        Numpad2 => Some('2'),
        Numpad3 => Some('3'),
        Numpad4 => Some('4'),
        Numpad5 => Some('5'),
        Numpad6 => Some('6'),
        Numpad7 => Some('7'),
        Numpad8 => Some('8'),
        Numpad9 => Some('9'),
        NumpadDecimal => Some('.'),
        NumpadSubtract => Some('-'),
        // Top-row digits only when shift is not held
        Digit0 if !shift => Some('0'),
        Digit1 if !shift => Some('1'),
        Digit2 if !shift => Some('2'),
        Digit3 if !shift => Some('3'),
        Digit4 if !shift => Some('4'),
        Digit5 if !shift => Some('5'),
        Digit6 if !shift => Some('6'),
        Digit7 if !shift => Some('7'),
        Digit8 if !shift => Some('8'),
        Digit9 if !shift => Some('9'),
        Period if !shift => Some('.'),
        Minus if !shift => Some('-'),
        _ => None,
    }
}

impl InputState {
    /// Create input state from an InputHandler.
    pub fn from_input_handler(input: &InputHandler) -> Self {
        let mouse = input.mouse();
        let pos = mouse.position();
        let kb = input.keyboard();

        let shift = kb.is_key_pressed(KeyCode::ShiftLeft)
            || kb.is_key_pressed(KeyCode::ShiftRight);

        // Collect typed characters from just-pressed keys
        let typed_keys = [
            KeyCode::Digit0, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
            KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7,
            KeyCode::Digit8, KeyCode::Digit9,
            KeyCode::Numpad0, KeyCode::Numpad1, KeyCode::Numpad2, KeyCode::Numpad3,
            KeyCode::Numpad4, KeyCode::Numpad5, KeyCode::Numpad6, KeyCode::Numpad7,
            KeyCode::Numpad8, KeyCode::Numpad9,
            KeyCode::Period, KeyCode::NumpadDecimal,
            KeyCode::Minus, KeyCode::NumpadSubtract,
        ];

        let mut typed_chars = Vec::new();
        for &key in &typed_keys {
            if kb.is_key_just_pressed(key) {
                if let Some(ch) = keycode_to_char(key, shift) {
                    typed_chars.push(ch);
                }
            }
        }

        Self {
            mouse_pos: Vec2::new(pos.x, pos.y),
            mouse_down: mouse.is_button_pressed(MouseButton::Left),
            mouse_just_pressed: mouse.is_button_just_pressed(MouseButton::Left),
            mouse_just_released: mouse.is_button_just_released(MouseButton::Left),
            scroll_delta: mouse.wheel_delta(),
            typed_chars,
            enter_pressed: kb.is_key_just_pressed(KeyCode::Enter)
                || kb.is_key_just_pressed(KeyCode::NumpadEnter),
            escape_pressed: kb.is_key_just_pressed(KeyCode::Escape),
            backspace_pressed: kb.is_key_just_pressed(KeyCode::Backspace),
            tab_pressed: kb.is_key_just_pressed(KeyCode::Tab),
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
    /// Regions (e.g. open dropdowns) that swallow mouse input for all
    /// widgets outside the overlay scope. Cleared each frame.
    blocking_rects: Vec<Rect>,
    /// Whether interact() calls are currently inside an overlay (exempt
    /// from blocking rects). Cleared each frame.
    overlay_scope: bool,
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
            blocking_rects: Vec::new(),
            overlay_scope: false,
        }
    }

    /// Begin a new frame, updating input state.
    pub fn begin_frame(&mut self, input: &InputHandler) {
        self.input = InputState::from_input_handler(input);

        // Clear hot widget at start of frame (will be set by widgets that are hovered)
        self.hot_widget = None;

        // Blocking regions are re-registered each frame by whatever overlay is open
        self.blocking_rects.clear();
        self.overlay_scope = false;

        // Don't clear active_widget here - let widgets check for clicks first
        // The active_widget will be cleared in end_frame() after click detection

        // Mark all persistent state as not seen
        for state in self.persistent_state.values_mut() {
            state.seen_this_frame = false;
        }
    }

    /// End a frame, cleaning up stale state.
    pub fn end_frame(&mut self) {
        // Clear active widget if mouse was just released (after click detection)
        if self.input.mouse_just_released {
            self.active_widget = None;
        }

        // Garbage collect persistent state for widgets not submitted this frame.
        // The focused widget's state is kept even when unseen so a text input
        // doesn't lose its edit buffer if its panel skips a frame.
        let focus = self.focus_widget;
        self.persistent_state
            .retain(|id, state| state.seen_this_frame || focus == Some(*id));
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

    /// Check if any widget has keyboard focus (e.g. a text input being edited).
    pub fn has_focus(&self) -> bool {
        self.focus_widget.is_some()
    }

    /// Register a region that swallows mouse input for all widgets outside
    /// the overlay scope (used by dropdown menus and popups). Cleared each frame.
    pub fn push_blocking_rect(&mut self, rect: Rect) {
        self.blocking_rects.push(rect);
    }

    /// Set whether subsequent interact() calls belong to an overlay and are
    /// therefore exempt from blocking rects.
    pub fn set_overlay_scope(&mut self, overlay: bool) {
        self.overlay_scope = overlay;
    }

    /// Check if mouse input at the given position is swallowed by a blocking
    /// region (an open dropdown or popup).
    pub fn is_blocked_at(&self, pos: Vec2) -> bool {
        self.blocking_rects.iter().any(|r| r.contains(pos))
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

        // Widgets outside an overlay are inert while the mouse is over a
        // blocking region (open dropdown/popup): no hover, no click, no
        // activation. An already-active widget keeps its slot — end_frame
        // clears it on mouse release.
        if !self.overlay_scope && self.is_blocked_at(self.input.mouse_pos) {
            return InteractionResult::default();
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

        let state = if is_active && !self.input.mouse_just_released {
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

    /// Build an InputHandler with the mouse at `pos`, optionally pressed.
    fn input_with_mouse(pos: Vec2, pressed: bool) -> InputHandler {
        let mut input = InputHandler::new();
        input.mouse_mut().update_position(pos.x, pos.y);
        if pressed {
            input.mouse_mut().handle_button_press(MouseButton::Left);
        }
        input
    }

    #[test]
    fn test_blocking_rect_makes_outside_widget_inert() {
        let mut manager = InteractionManager::new();
        let input = input_with_mouse(Vec2::new(50.0, 50.0), true);
        manager.begin_frame(&input);

        // A dropdown covers the widget's area
        manager.push_blocking_rect(Rect::new(0.0, 0.0, 100.0, 100.0));

        let id = WidgetId::from_str("widget_under_dropdown");
        let result = manager.interact(id, Rect::new(40.0, 40.0, 50.0, 50.0), true);

        assert_eq!(result.state, WidgetState::Normal, "no hover under a blocking rect");
        assert!(!result.clicked);
        assert!(!result.dragging);
        assert!(manager.active_widget.is_none(), "press must not activate a blocked widget");
        assert!(manager.hot_widget.is_none());
    }

    #[test]
    fn test_overlay_scope_widget_stays_interactive_over_blocking_rect() {
        let mut manager = InteractionManager::new();
        let input = input_with_mouse(Vec2::new(50.0, 50.0), true);
        manager.begin_frame(&input);

        manager.push_blocking_rect(Rect::new(0.0, 0.0, 100.0, 100.0));
        manager.set_overlay_scope(true);

        let id = WidgetId::from_str("dropdown_item");
        let result = manager.interact(id, Rect::new(40.0, 40.0, 50.0, 50.0), true);

        assert_eq!(result.state, WidgetState::Active, "overlay widget receives the press");
        assert!(result.dragging);
    }

    #[test]
    fn test_widget_outside_blocking_rect_unaffected() {
        let mut manager = InteractionManager::new();
        let input = input_with_mouse(Vec2::new(300.0, 300.0), false);
        manager.begin_frame(&input);

        manager.push_blocking_rect(Rect::new(0.0, 0.0, 100.0, 100.0));

        let id = WidgetId::from_str("far_widget");
        let result = manager.interact(id, Rect::new(280.0, 280.0, 50.0, 50.0), true);
        assert_eq!(result.state, WidgetState::Hovered, "blocking only applies under the rect");
    }

    #[test]
    fn test_blocked_widget_persistent_state_survives_frame() {
        let mut manager = InteractionManager::new();
        let id = WidgetId::from_str("blocked_text_input");
        manager.get_state(id).string_value = "edit buffer".to_string();

        let input = input_with_mouse(Vec2::new(50.0, 50.0), false);
        manager.begin_frame(&input);
        manager.push_blocking_rect(Rect::new(0.0, 0.0, 100.0, 100.0));
        manager.interact(id, Rect::new(40.0, 40.0, 20.0, 20.0), true);
        manager.end_frame();

        let state = manager.get_state_if_exists(id).expect("blocked widget state retained");
        assert_eq!(state.string_value, "edit buffer");
    }

    #[test]
    fn test_begin_frame_clears_blocking_state() {
        let mut manager = InteractionManager::new();
        manager.push_blocking_rect(Rect::new(0.0, 0.0, 100.0, 100.0));
        manager.set_overlay_scope(true);
        assert!(manager.is_blocked_at(Vec2::new(50.0, 50.0)));

        manager.begin_frame(&InputHandler::new());
        assert!(!manager.is_blocked_at(Vec2::new(50.0, 50.0)));
        assert!(!manager.overlay_scope);
    }

    #[test]
    fn test_has_focus_tracks_any_focused_widget() {
        let mut manager = InteractionManager::new();
        assert!(!manager.has_focus());

        manager.set_focus(WidgetId::from_str("field"));
        assert!(manager.has_focus());

        manager.clear_focus();
        assert!(!manager.has_focus());
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
        state.string_value = "hello".to_string();

        let state = manager.get_state_if_exists(id).unwrap();
        assert!(state.seen_this_frame);
        assert_eq!(state.string_value, "hello");
    }

    #[test]
    fn test_unseen_widget_state_is_garbage_collected() {
        let mut manager = InteractionManager::new();
        let id = WidgetId::from_str("transient");

        manager.get_state(id).string_value = "data".to_string();
        manager.end_frame();
        assert!(manager.get_state_if_exists(id).is_some(), "seen state survives the frame");

        // Next frame: widget never submitted
        manager.begin_frame(&InputHandler::new());
        manager.end_frame();
        assert!(manager.get_state_if_exists(id).is_none(), "unseen state is collected");
    }

    #[test]
    fn test_focused_widget_state_survives_unseen_frame() {
        let mut manager = InteractionManager::new();
        let id = WidgetId::from_str("text_input");

        manager.get_state(id).string_value = "editing".to_string();
        manager.set_focus(id);
        manager.end_frame();

        // Next frame: widget not submitted (e.g., panel skipped a frame),
        // but it holds focus so its edit buffer must be retained.
        manager.begin_frame(&InputHandler::new());
        manager.end_frame();

        let state = manager.get_state_if_exists(id).expect("focused state retained");
        assert_eq!(state.string_value, "editing");
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
        assert!(input.typed_chars.is_empty());
        assert!(!input.enter_pressed);
        assert!(!input.escape_pressed);
        assert!(!input.backspace_pressed);
        assert!(!input.tab_pressed);
    }

    #[test]
    fn test_keycode_to_char_digits() {
        assert_eq!(keycode_to_char(KeyCode::Digit0, false), Some('0'));
        assert_eq!(keycode_to_char(KeyCode::Digit9, false), Some('9'));
        assert_eq!(keycode_to_char(KeyCode::Numpad5, false), Some('5'));
        assert_eq!(keycode_to_char(KeyCode::Numpad5, true), Some('5')); // numpad ignores shift
    }

    #[test]
    fn test_keycode_to_char_special() {
        assert_eq!(keycode_to_char(KeyCode::Period, false), Some('.'));
        assert_eq!(keycode_to_char(KeyCode::Minus, false), Some('-'));
        assert_eq!(keycode_to_char(KeyCode::NumpadDecimal, false), Some('.'));
        assert_eq!(keycode_to_char(KeyCode::NumpadSubtract, true), Some('-'));
    }

    #[test]
    fn test_keycode_to_char_shift_blocks_top_row() {
        assert_eq!(keycode_to_char(KeyCode::Digit0, true), None); // Shift+0 = ')'
        assert_eq!(keycode_to_char(KeyCode::Period, true), None); // Shift+. = '>'
        assert_eq!(keycode_to_char(KeyCode::Minus, true), None); // Shift+- = '_'
    }

    #[test]
    fn test_keycode_to_char_non_numeric() {
        assert_eq!(keycode_to_char(KeyCode::KeyA, false), None);
        assert_eq!(keycode_to_char(KeyCode::Space, false), None);
        assert_eq!(keycode_to_char(KeyCode::Enter, false), None);
    }

    #[test]
    fn test_focus_management() {
        let mut manager = InteractionManager::new();
        let id = WidgetId::from_str("text_input");

        assert!(!manager.is_focused(id));

        manager.set_focus(id);
        assert!(manager.is_focused(id));

        manager.clear_focus();
        assert!(!manager.is_focused(id));
    }
}
