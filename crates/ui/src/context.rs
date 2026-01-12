//! Immediate-mode UI context.
//!
//! The UIContext is the main entry point for creating UI elements.
//! It follows an immediate-mode paradigm where you describe the UI every frame.

use glam::Vec2;
use input::InputHandler;

use crate::{
    Color, DrawList, FontError, FontHandle, FontManager, GlyphDrawData, InteractionManager,
    InteractionResult, Rect, TextDrawData, Theme, WidgetId, WidgetState,
};

/// The main UI context for immediate-mode UI rendering.
///
/// # Example
/// ```ignore
/// let mut ui = UIContext::new();
///
/// // In your update loop:
/// ui.begin_frame(&input_handler);
///
/// if ui.button("my_button", "Click Me!", Rect::new(10.0, 10.0, 100.0, 30.0)) {
///     println!("Button clicked!");
/// }
///
/// ui.end_frame();
///
/// // Get draw commands for rendering
/// let commands = ui.draw_list().commands();
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
        Self {
            interaction: InteractionManager::new(),
            draw_list: DrawList::new(),
            theme,
            window_size: Vec2::new(800.0, 600.0),
            font_manager: FontManager::new(),
        }
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

    // ================== Widget Methods ==================

    /// Create a button widget.
    ///
    /// Returns `true` if the button was clicked this frame.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this button
    /// * `label` - Text to display on the button
    /// * `bounds` - Position and size of the button
    pub fn button(&mut self, id: impl Into<WidgetId>, label: &str, bounds: Rect) -> bool {
        self.button_styled(id, label, bounds, true)
    }

    /// Create a button widget that can be disabled.
    pub fn button_styled(
        &mut self,
        id: impl Into<WidgetId>,
        label: &str,
        bounds: Rect,
        enabled: bool,
    ) -> bool {
        let id = id.into();
        let result = self.interaction.interact(id, bounds, enabled);
        let style = &self.theme.button;

        let background = match result.state {
            WidgetState::Normal => style.background,
            WidgetState::Hovered => style.background_hovered,
            WidgetState::Active => style.background_pressed,
            WidgetState::Disabled => style.background_disabled,
        };

        let text_color = if enabled {
            style.text_color
        } else {
            style.text_color_disabled
        };

        // Draw button background
        self.draw_list
            .rect_rounded(bounds, background, style.corner_radius);

        // Draw border
        if style.border_width > 0.0 {
            self.draw_list
                .rect_border_rounded(bounds, style.border, style.border_width, style.corner_radius);
        }

        // Draw label (centered)
        let text_pos = bounds.center();
        self.draw_list
            .text_placeholder(label, text_pos, text_color, self.theme.text.font_size);

        result.clicked
    }

    /// Create a text label.
    ///
    /// If a font has been loaded, renders with actual glyphs. Otherwise falls back to placeholder.
    pub fn label(&mut self, text: &str, position: Vec2) {
        self.label_styled(text, position, self.theme.text.color, self.theme.text.font_size);
    }

    /// Create a text label with custom styling.
    ///
    /// If a font has been loaded, renders with actual glyphs. Otherwise falls back to placeholder.
    pub fn label_styled(&mut self, text: &str, position: Vec2, color: Color, font_size: f32) {
        // Try to render with font if available
        if let Some(font_handle) = self.font_manager.default_font() {
            match self.font_manager.layout_text(font_handle, text, font_size) {
                Ok(layout) => {
                    let glyphs: Vec<GlyphDrawData> = layout.glyphs.iter().map(|g| {
                        GlyphDrawData {
                            bitmap: g.info.rasterized.bitmap.clone(),
                            width: g.info.rasterized.width,
                            height: g.info.rasterized.height,
                            x: g.x,
                            y: g.y,
                            character: g.character,
                        }
                    }).collect();

                    let text_data = TextDrawData {
                        text: text.to_string(),
                        position,
                        color,
                        font_size,
                        width: layout.width,
                        height: layout.height,
                        glyphs,
                    };
                    self.draw_list.text(text_data);
                    return;
                }
                Err(e) => {
                    println!("[FONT DEBUG] layout_text failed: {}", e);
                }
            }
        } else {
            // Only print once
            static PRINTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !PRINTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                println!("[FONT DEBUG] No default font loaded");
            }
        }

        // Fall back to placeholder
        self.draw_list.text_placeholder(text, position, color, font_size);
    }

    /// Create a text label with a specific font handle.
    pub fn label_with_font(&mut self, text: &str, position: Vec2, font: FontHandle, font_size: f32) {
        let color = self.theme.text.color;
        if let Ok(layout) = self.font_manager.layout_text(font, text, font_size) {
            let glyphs: Vec<GlyphDrawData> = layout.glyphs.iter().map(|g| {
                GlyphDrawData {
                    bitmap: g.info.rasterized.bitmap.clone(),
                    width: g.info.rasterized.width,
                    height: g.info.rasterized.height,
                    x: g.x,
                    y: g.y,
                    character: g.character,
                }
            }).collect();

            let text_data = TextDrawData {
                text: text.to_string(),
                position,
                color,
                font_size,
                width: layout.width,
                height: layout.height,
                glyphs,
            };
            self.draw_list.text(text_data);
        } else {
            self.draw_list.text_placeholder(text, position, color, font_size);
        }
    }

    /// Create a panel (container background).
    pub fn panel(&mut self, bounds: Rect) {
        let style = &self.theme.panel;
        self.draw_list.panel(
            bounds,
            style.background,
            style.border,
            style.border_width,
            style.corner_radius,
        );
    }

    /// Create a panel with custom styling.
    pub fn panel_styled(&mut self, bounds: Rect, background: Color, border: Color, border_width: f32) {
        let style = &self.theme.panel;
        self.draw_list.panel(
            bounds,
            background,
            border,
            border_width,
            style.corner_radius,
        );
    }

    /// Create a horizontal slider.
    ///
    /// Returns the new value if it changed.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this slider
    /// * `value` - Current value (0.0 to 1.0)
    /// * `bounds` - Position and size of the slider track
    pub fn slider(&mut self, id: impl Into<WidgetId>, value: f32, bounds: Rect) -> f32 {
        self.slider_range(id, value, 0.0, 1.0, bounds)
    }

    /// Create a horizontal slider with a custom range.
    pub fn slider_range(
        &mut self,
        id: impl Into<WidgetId>,
        value: f32,
        min: f32,
        max: f32,
        bounds: Rect,
    ) -> f32 {
        let id = id.into();
        let style = &self.theme.slider;

        // Normalize value to 0-1 range
        let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);

        // Calculate track bounds (vertically centered in the provided bounds)
        let track_y = bounds.y + (bounds.height - style.track_height) / 2.0;
        let track_bounds = Rect::new(bounds.x, track_y, bounds.width, style.track_height);

        // Calculate thumb position
        let thumb_x = bounds.x + normalized * bounds.width;
        let thumb_y = bounds.y + bounds.height / 2.0;
        let thumb_center = Vec2::new(thumb_x, thumb_y);

        // Check interaction with the entire slider area (not just thumb)
        let result = self.interaction.interact(id, bounds, true);

        let thumb_color = match result.state {
            WidgetState::Normal => style.thumb_color,
            WidgetState::Hovered => style.thumb_hovered,
            WidgetState::Active => style.thumb_pressed,
            WidgetState::Disabled => style.thumb_color,
        };

        // Draw slider
        self.draw_list.slider(
            track_bounds,
            thumb_center,
            style.thumb_radius,
            style.track_background,
            style.track_fill,
            thumb_color,
            normalized,
        );

        // Calculate new value if dragging
        if result.dragging {
            let mouse_x = self.interaction.mouse_pos().x;
            let new_normalized = ((mouse_x - bounds.x) / bounds.width).clamp(0.0, 1.0);
            min + new_normalized * (max - min)
        } else {
            value
        }
    }

    /// Create a checkbox.
    ///
    /// Returns `true` if the checkbox was toggled this frame.
    pub fn checkbox(&mut self, id: impl Into<WidgetId>, checked: bool, bounds: Rect) -> bool {
        let id = id.into();
        let result = self.interaction.interact(id, bounds, true);
        let style = &self.theme.button;

        let background = match result.state {
            WidgetState::Normal => style.background,
            WidgetState::Hovered => style.background_hovered,
            WidgetState::Active => style.background_pressed,
            WidgetState::Disabled => style.background_disabled,
        };

        // Draw checkbox background
        self.draw_list
            .rect_rounded(bounds, background, style.corner_radius);

        // Draw border
        self.draw_list
            .rect_border_rounded(bounds, style.border, style.border_width, style.corner_radius);

        // Draw check mark if checked
        if checked {
            let inner = bounds.shrink(bounds.width * 0.25);
            self.draw_list
                .rect_rounded(inner, style.text_color, style.corner_radius * 0.5);
        }

        result.clicked
    }

    /// Create a checkbox with a label.
    pub fn checkbox_labeled(
        &mut self,
        id: impl Into<WidgetId>,
        label: &str,
        checked: bool,
        position: Vec2,
    ) -> bool {
        let checkbox_size = self.theme.text.font_size * 1.2;
        let checkbox_bounds = Rect::new(position.x, position.y, checkbox_size, checkbox_size);

        let clicked = self.checkbox(id, checked, checkbox_bounds);

        // Draw label
        let label_pos = Vec2::new(
            position.x + checkbox_size + 8.0,
            position.y + checkbox_size / 2.0,
        );
        self.label(label, label_pos);

        clicked
    }

    /// Create a progress bar.
    pub fn progress_bar(&mut self, value: f32, bounds: Rect) {
        self.progress_bar_styled(value, bounds, self.theme.slider.track_background, self.theme.slider.track_fill);
    }

    /// Create a progress bar with custom colors.
    pub fn progress_bar_styled(&mut self, value: f32, bounds: Rect, background: Color, fill: Color) {
        let style = &self.theme.panel;
        let normalized = value.clamp(0.0, 1.0);

        // Draw background
        self.draw_list
            .rect_rounded(bounds, background, style.corner_radius);

        // Draw fill
        if normalized > 0.0 {
            let fill_width = bounds.width * normalized;
            let fill_bounds = Rect::new(bounds.x, bounds.y, fill_width, bounds.height);
            self.draw_list
                .rect_rounded(fill_bounds, fill, style.corner_radius);
        }
    }

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

    /// Check if a point is inside a rectangle (for custom hit testing).
    pub fn hit_test(&self, point: Vec2, bounds: Rect) -> bool {
        bounds.contains(point)
    }

    /// Get raw interaction result for custom widgets.
    pub fn interact(&mut self, id: impl Into<WidgetId>, bounds: Rect, enabled: bool) -> InteractionResult {
        self.interaction.interact(id.into(), bounds, enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DrawCommand;

    #[test]
    fn test_ui_context_new() {
        let ui = UIContext::new();
        assert!(ui.draw_list().is_empty());
    }

    #[test]
    fn test_ui_context_with_theme() {
        let theme = Theme::light();
        let ui = UIContext::with_theme(theme);
        // Light theme has different colors
        assert_ne!(ui.theme().button.background.r, Theme::default().button.background.r);
    }

    #[test]
    fn test_ui_context_set_theme() {
        let mut ui = UIContext::new();
        let original_bg = ui.theme().button.background;

        ui.set_theme(Theme::light());
        assert_ne!(ui.theme().button.background.r, original_bg.r);
    }

    #[test]
    fn test_ui_context_window_size() {
        let ui = UIContext::new();
        assert_eq!(ui.window_size(), Vec2::new(800.0, 600.0));
    }

    #[test]
    fn test_ui_context_label() {
        let mut ui = UIContext::new();
        ui.label("Test", Vec2::new(10.0, 20.0));
        assert_eq!(ui.draw_list().len(), 1);

        if let DrawCommand::TextPlaceholder { text, position, .. } = &ui.draw_list().commands()[0] {
            assert_eq!(text, "Test");
            assert_eq!(*position, Vec2::new(10.0, 20.0));
        } else {
            panic!("Expected TextPlaceholder command");
        }
    }

    #[test]
    fn test_ui_context_panel() {
        let mut ui = UIContext::new();
        ui.panel(Rect::new(0.0, 0.0, 200.0, 100.0));
        // Panel creates a rect and optionally a border
        assert!(ui.draw_list().len() >= 1);
    }

    #[test]
    fn test_ui_context_rect() {
        let mut ui = UIContext::new();
        ui.rect(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
        assert_eq!(ui.draw_list().len(), 1);
    }

    #[test]
    fn test_ui_context_circle() {
        let mut ui = UIContext::new();
        ui.circle(Vec2::new(50.0, 50.0), 25.0, Color::BLUE);
        assert_eq!(ui.draw_list().len(), 1);
    }

    #[test]
    fn test_ui_context_hit_test() {
        let ui = UIContext::new();
        let bounds = Rect::new(10.0, 10.0, 100.0, 100.0);

        assert!(ui.hit_test(Vec2::new(50.0, 50.0), bounds));
        assert!(!ui.hit_test(Vec2::new(5.0, 5.0), bounds));
    }

    #[test]
    fn test_ui_context_progress_bar() {
        let mut ui = UIContext::new();
        ui.progress_bar(0.5, Rect::new(0.0, 0.0, 200.0, 20.0));
        // Progress bar creates background and fill rects
        assert!(ui.draw_list().len() >= 1);
    }

    #[test]
    fn test_ui_context_font_manager_access() {
        let ui = UIContext::new();
        // Font manager should be accessible and have no default font initially
        assert!(ui.default_font().is_none());
        assert!(ui.font_manager().default_font().is_none());
    }

    #[test]
    fn test_ui_context_font_manager_mut_access() {
        let mut ui = UIContext::new();
        // Should be able to get mutable access to font manager
        let fm = ui.font_manager_mut();
        let (fonts, glyphs) = fm.cache_stats();
        assert_eq!(fonts, 0);
        assert_eq!(glyphs, 0);
    }

    #[test]
    fn test_ui_context_label_without_font() {
        let mut ui = UIContext::new();
        // Without a font loaded, label should fall back to TextPlaceholder
        ui.label("No Font", Vec2::new(10.0, 20.0));
        assert_eq!(ui.draw_list().len(), 1);

        if let DrawCommand::TextPlaceholder { text, .. } = &ui.draw_list().commands()[0] {
            assert_eq!(text, "No Font");
        } else {
            panic!("Expected TextPlaceholder command when no font is loaded");
        }
    }

    #[test]
    fn test_ui_context_label_styled_without_font() {
        let mut ui = UIContext::new();
        ui.label_styled("Styled Text", Vec2::new(50.0, 60.0), Color::RED, 24.0);
        assert_eq!(ui.draw_list().len(), 1);

        if let DrawCommand::TextPlaceholder { text, color, font_size, .. } = &ui.draw_list().commands()[0] {
            assert_eq!(text, "Styled Text");
            assert_eq!(*color, Color::RED);
            assert_eq!(*font_size, 24.0);
        } else {
            panic!("Expected TextPlaceholder command");
        }
    }
}
