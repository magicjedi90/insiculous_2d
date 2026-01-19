//! Editor toolbar with tool selection.
//!
//! The toolbar provides buttons for switching between editor tools
//! (Select, Move, Rotate, Scale) and displays the current tool state.

use glam::Vec2;
use ui::{Color, Rect, UIContext};

/// Available editor tools for manipulating entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorTool {
    /// Select and click entities
    #[default]
    Select,
    /// Move/translate entities
    Move,
    /// Rotate entities
    Rotate,
    /// Scale entities uniformly or non-uniformly
    Scale,
}

impl EditorTool {
    /// Get the display name for this tool.
    pub fn name(&self) -> &'static str {
        match self {
            EditorTool::Select => "Select",
            EditorTool::Move => "Move",
            EditorTool::Rotate => "Rotate",
            EditorTool::Scale => "Scale",
        }
    }

    /// Get the keyboard shortcut hint for this tool.
    pub fn shortcut(&self) -> &'static str {
        match self {
            EditorTool::Select => "Q",
            EditorTool::Move => "W",
            EditorTool::Rotate => "E",
            EditorTool::Scale => "R",
        }
    }

    /// Get all available tools.
    pub fn all() -> &'static [EditorTool] {
        &[
            EditorTool::Select,
            EditorTool::Move,
            EditorTool::Rotate,
            EditorTool::Scale,
        ]
    }
}

/// Editor toolbar widget.
///
/// Renders a horizontal bar with tool selection buttons.
#[derive(Debug, Clone)]
pub struct Toolbar {
    /// Current selected tool
    current_tool: EditorTool,
    /// Position of the toolbar
    position: Vec2,
    /// Button size
    button_size: f32,
    /// Spacing between buttons
    spacing: f32,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Toolbar {
    /// Create a new toolbar with default settings.
    pub fn new() -> Self {
        Self {
            current_tool: EditorTool::Select,
            position: Vec2::new(10.0, 10.0),
            button_size: 40.0,
            spacing: 4.0,
        }
    }

    /// Set the toolbar position.
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Set the button size.
    pub fn with_button_size(mut self, size: f32) -> Self {
        self.button_size = size;
        self
    }

    /// Get the currently selected tool.
    pub fn current_tool(&self) -> EditorTool {
        self.current_tool
    }

    /// Set the current tool.
    pub fn set_tool(&mut self, tool: EditorTool) {
        self.current_tool = tool;
    }

    /// Get the toolbar bounds (for layout purposes).
    pub fn bounds(&self) -> Rect {
        let tools = EditorTool::all();
        let width = tools.len() as f32 * (self.button_size + self.spacing) - self.spacing;
        Rect::new(self.position.x, self.position.y, width, self.button_size)
    }

    /// Render the toolbar and handle tool selection.
    ///
    /// Returns the newly selected tool if changed.
    pub fn render(&mut self, ui: &mut UIContext) -> Option<EditorTool> {
        let tools = EditorTool::all();
        let mut new_tool = None;

        // Draw toolbar background
        let bounds = self.bounds();
        let bg_bounds = bounds.expand(4.0);
        ui.panel(bg_bounds);

        // Draw tool buttons
        for (i, &tool) in tools.iter().enumerate() {
            let x = self.position.x + i as f32 * (self.button_size + self.spacing);
            let button_bounds = Rect::new(x, self.position.y, self.button_size, self.button_size);

            let is_selected = tool == self.current_tool;

            // Use different styling for selected vs unselected
            if is_selected {
                // Draw selected indicator
                ui.rect_rounded(button_bounds, Color::new(0.3, 0.5, 0.8, 1.0), 4.0);
            }

            // Draw button (will use default styling with hover effect)
            let id = format!("toolbar_{}", tool.name());
            if ui.button(id.as_str(), tool.name(), button_bounds) {
                self.current_tool = tool;
                new_tool = Some(tool);
            }

            // Draw shortcut hint below button
            let hint_pos = Vec2::new(
                button_bounds.center().x,
                button_bounds.y + button_bounds.height + 2.0,
            );
            ui.label_styled(tool.shortcut(), hint_pos, Color::new(0.5, 0.5, 0.5, 1.0), 10.0);
        }

        new_tool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_tool_default() {
        let tool = EditorTool::default();
        assert_eq!(tool, EditorTool::Select);
    }

    #[test]
    fn test_editor_tool_names() {
        assert_eq!(EditorTool::Select.name(), "Select");
        assert_eq!(EditorTool::Move.name(), "Move");
        assert_eq!(EditorTool::Rotate.name(), "Rotate");
        assert_eq!(EditorTool::Scale.name(), "Scale");
    }

    #[test]
    fn test_editor_tool_shortcuts() {
        assert_eq!(EditorTool::Select.shortcut(), "Q");
        assert_eq!(EditorTool::Move.shortcut(), "W");
        assert_eq!(EditorTool::Rotate.shortcut(), "E");
        assert_eq!(EditorTool::Scale.shortcut(), "R");
    }

    #[test]
    fn test_editor_tool_all() {
        let tools = EditorTool::all();
        assert_eq!(tools.len(), 4);
        assert!(tools.contains(&EditorTool::Select));
        assert!(tools.contains(&EditorTool::Move));
        assert!(tools.contains(&EditorTool::Rotate));
        assert!(tools.contains(&EditorTool::Scale));
    }

    #[test]
    fn test_toolbar_new() {
        let toolbar = Toolbar::new();
        assert_eq!(toolbar.current_tool(), EditorTool::Select);
    }

    #[test]
    fn test_toolbar_with_position() {
        let toolbar = Toolbar::new().with_position(Vec2::new(100.0, 50.0));
        assert_eq!(toolbar.position, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_toolbar_set_tool() {
        let mut toolbar = Toolbar::new();
        toolbar.set_tool(EditorTool::Move);
        assert_eq!(toolbar.current_tool(), EditorTool::Move);
    }

    #[test]
    fn test_toolbar_bounds() {
        let toolbar = Toolbar::new();
        let bounds = toolbar.bounds();

        // 4 tools * (40 + 4) - 4 = 172
        assert_eq!(bounds.width, 172.0);
        assert_eq!(bounds.height, 40.0);
    }
}
