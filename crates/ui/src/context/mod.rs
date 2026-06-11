//! Immediate-mode UI context.
//!
//! The UIContext is the main entry point for creating UI elements.
//! It follows an immediate-mode paradigm where you describe the UI every frame.
//!
//! Split by responsibility:
//! - `mod.rs` — UIContext struct, construction, frame lifecycle, fonts, core state
//! - `text.rs` — label/measure family and shared text-drawing helpers
//! - `widgets.rs` — interactive widgets (button, slider, checkbox, float_input)
//!   and container/shape drawing

mod text;
mod widgets;

#[cfg(test)]
mod tests;

use glam::Vec2;
use input::InputHandler;

/// Text alignment within a bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// Align text to the left edge
    #[default]
    Left,
    /// Center text horizontally
    Center,
    /// Align text to the right edge
    Right,
}

use crate::{
    Color, DrawList, FontError, FontHandle, FontManager, InteractionManager, InteractionResult,
    Rect, Theme, WidgetId,
};

/// The main UI context for immediate-mode UI rendering.
///
/// # Example
/// ```
/// # use ui::{UIContext, Rect};
/// # use glam::Vec2;
/// # use input::InputHandler;
/// let mut ui = UIContext::new();
/// # let input_handler = InputHandler::new();
///
/// // In your update loop:
/// ui.begin_frame(&input_handler, Vec2::new(800.0, 600.0));
///
/// if ui.button("my_button", "Click Me!", Rect::new(10.0, 10.0, 100.0, 30.0)) {
///     println!("Button clicked!");
/// }
///
/// ui.end_frame();
///
/// // Get draw commands for rendering
/// let commands = ui.draw_list().commands();
/// # assert!(!commands.is_empty());
/// ```
pub struct UIContext {
    /// Interaction manager for widget state tracking
    interaction: InteractionManager,
    /// Draw list for collecting render commands
    draw_list: DrawList,
    /// Current theme
    theme: Theme,
    /// Window size for layout calculations
    window_size: Vec2,
    /// Font manager for text rendering
    font_manager: FontManager,
}

impl Default for UIContext {
    fn default() -> Self {
        Self::new()
    }
}

impl UIContext {
    /// Create a new UI context with default theme.
    pub fn new() -> Self {
        Self {
            interaction: InteractionManager::new(),
            draw_list: DrawList::new(),
            theme: Theme::default(),
            window_size: Vec2::new(800.0, 600.0),
            font_manager: FontManager::new(),
        }
    }

    /// Create a new UI context with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        let mut ctx = Self::new();
        ctx.theme = theme;
        ctx
    }

    // ================== Font Methods ==================

    /// Load a font from file bytes.
    pub fn load_font(&mut self, font_data: &[u8]) -> Result<FontHandle, FontError> {
        self.font_manager.load_font(font_data)
    }

    /// Load a font from a file path.
    pub fn load_font_file(&mut self, path: &str) -> Result<FontHandle, FontError> {
        self.font_manager.load_font_file(path)
    }

    /// Get the default font handle.
    pub fn default_font(&self) -> Option<FontHandle> {
        self.font_manager.default_font()
    }

    /// Set the default font.
    pub fn set_default_font(&mut self, handle: FontHandle) {
        self.font_manager.set_default_font(handle);
    }

    /// Get the font manager for advanced operations.
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }

    /// Get the font manager mutably.
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// Get font metrics for the default font at the given size.
    ///
    /// Returns None if no font is loaded or metrics are unavailable.
    /// Use this for calculating text positions with baseline alignment.
    pub fn font_metrics(&self, font_size: f32) -> Option<crate::FontMetrics> {
        let font = self.font_manager.default_font()?;
        self.font_manager.metrics(font, font_size)
    }

    // ================== Frame Lifecycle ==================

    /// Begin a new frame. Call this at the start of each frame.
    pub fn begin_frame(&mut self, input: &InputHandler, window_size: Vec2) {
        self.interaction.begin_frame(input);
        self.draw_list.clear();
        self.window_size = window_size;
    }

    /// End the frame. Call this after all UI elements have been created.
    pub fn end_frame(&mut self) {
        self.interaction.end_frame();
    }

    /// Get the draw list containing all render commands.
    pub fn draw_list(&self) -> &DrawList {
        &self.draw_list
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set a new theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Get the window size.
    pub fn window_size(&self) -> Vec2 {
        self.window_size
    }

    /// Get the current mouse position.
    pub fn mouse_pos(&self) -> Vec2 {
        self.interaction.mouse_pos()
    }

    // ================== Drawing Primitives ==================

    /// Draw a colored rectangle.
    pub fn rect(&mut self, bounds: Rect, color: Color) {
        self.draw_list.rect(bounds, color);
    }

    /// Draw a colored rectangle with rounded corners.
    pub fn rect_rounded(&mut self, bounds: Rect, color: Color, corner_radius: f32) {
        self.draw_list.rect_rounded(bounds, color, corner_radius);
    }

    /// Draw a circle.
    pub fn circle(&mut self, center: Vec2, radius: f32, color: Color) {
        self.draw_list.circle(center, radius, color);
    }

    /// Draw a line.
    pub fn line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) {
        self.draw_list.line(start, end, color, width);
    }

    /// Begin clipping all subsequent draws to the given bounds.
    ///
    /// Use this to prevent content from rendering outside panel boundaries.
    /// Must be paired with `pop_clip_rect()`. Clip regions can be nested.
    pub fn push_clip_rect(&mut self, bounds: Rect) {
        self.draw_list.push_clip_rect(bounds);
    }

    /// End the current clip region, restoring the previous clip state.
    ///
    /// If no clip region is active, this resets to the full viewport.
    pub fn pop_clip_rect(&mut self) {
        self.draw_list.pop_clip_rect();
    }

    // ================== Custom Widget Hooks ==================

    /// Check if a point is inside a rectangle (for custom hit testing).
    pub fn hit_test(&self, point: Vec2, bounds: Rect) -> bool {
        bounds.contains(point)
    }

    /// Get raw interaction result for custom widgets.
    pub fn interact(&mut self, id: impl Into<WidgetId>, bounds: Rect, enabled: bool) -> InteractionResult {
        self.interaction.interact(id.into(), bounds, enabled)
    }
}
