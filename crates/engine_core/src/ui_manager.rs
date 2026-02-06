//! UI Manager - extracts UI responsibilities from GameRunner
//!
//! Handles UI frame lifecycle, draw command collection, and event coordination.

use ui::{UIContext, DrawCommand};
use input::InputHandler;

/// Manages UI lifecycle and draw command collection
pub struct UIManager {
    ui_context: UIContext,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new() -> Self {
        Self {
            ui_context: UIContext::new(),
        }
    }

    /// Begin a new UI frame
    pub fn begin_frame(&mut self, input: &InputHandler, window_size: glam::Vec2) {
        self.ui_context.begin_frame(input, window_size);
    }

    /// Get mutable access to the UI context
    pub fn ui_context(&mut self) -> &mut UIContext {
        &mut self.ui_context
    }

    /// End the UI frame and collect draw commands
    pub fn end_frame(&mut self) -> Vec<DrawCommand> {
        self.ui_context.end_frame();
        self.ui_context.draw_list().commands().to_vec()
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_manager_creation() {
        let _manager = UIManager::new();
        // Should create without errors
    }

    #[test]
    fn test_ui_manager_frame_lifecycle() {
        let input = InputHandler::new();
        let mut manager = UIManager::new();
        let window_size = glam::Vec2::new(800.0, 600.0);

        manager.begin_frame(&input, window_size);
        let ctx = manager.ui_context();
        ctx.label("Test", glam::Vec2::new(10.0, 10.0));
        let commands = manager.end_frame();

        assert!(!commands.is_empty(), "Should have UI commands after frame");
    }
}