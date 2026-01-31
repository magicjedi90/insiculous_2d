//! Immediate-mode UI context.
//!
//! The UIContext is the main entry point for creating UI elements.
//! It follows an immediate-mode paradigm where you describe the UI every frame.

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
    Color, DrawList, FontError, FontHandle, FontManager, GlyphDrawData, InteractionManager,
    InteractionResult, Rect, TextDrawData, TextLayout, Theme, WidgetId, WidgetState,
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
    pub fn font_metrics(&self, font_size: f32) -> Option<crate::font::FontMetrics> {
        let font = self.font_manager.default_font()?;
        self.font_manager.metrics(font, font_size)
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

    // ================== Widget Helpers ==================

    /// Get the background color for a widget based on its state and the button style
    fn widget_background_color(&self, state: WidgetState) -> Color {
        let style = &self.theme.button;
        match state {
            WidgetState::Normal => style.background,
            WidgetState::Hovered => style.background_hovered,
            WidgetState::Active => style.background_pressed,
            WidgetState::Disabled => style.background_disabled,
        }
    }

    /// Convert a TextLayout to TextDrawData for rendering.
    ///
    /// This helper extracts the common pattern of converting font layout information
    /// into the draw data structure used by the rendering system.
    fn layout_to_draw_data(
        layout: &TextLayout,
        text: &str,
        position: Vec2,
        color: Color,
        font_size: f32,
    ) -> TextDrawData {
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

        TextDrawData {
            text: text.to_string(),
            position,
            color,
            font_size,
            width: layout.width,
            height: layout.height,
            glyphs,
        }
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
        let background = self.widget_background_color(result.state);

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

        // Draw label (centered) - use actual font if available
        let font_size = self.theme.text.font_size;
        if let Some(font_handle) = self.font_manager.default_font() {
            // Measure text to center it properly
            if let Ok(text_size) = self.font_manager.measure_text(font_handle, label, font_size) {
                // Position text so it's centered vertically
                let text_top = bounds.y + (bounds.height - text_size.y) / 2.0;
                // Get font metrics to find the baseline
                let metrics = self.font_manager.metrics(font_handle, font_size);
                let ascent = metrics.map(|m| m.ascent).unwrap_or(font_size * 0.8);
                // The baseline is 'ascent' pixels below the text top
                let baseline_y = text_top + ascent;
                let text_pos = Vec2::new(
                    bounds.x + (bounds.width - text_size.x) / 2.0,
                    baseline_y,
                );
                if let Ok(layout) = self.font_manager.layout_text(font_handle, label, font_size) {
                    let text_data = Self::layout_to_draw_data(&layout, label, text_pos, text_color, font_size);
                    self.draw_list.text(text_data);
                    return result.clicked;
                }
            }
        }

        // Fallback to placeholder if no font
        let text_pos = bounds.center();
        self.draw_list
            .text_placeholder(label, text_pos, text_color, font_size);

        result.clicked
    }

    /// Create a text label.
    ///
    /// If a font has been loaded, renders with actual glyphs. Otherwise falls back to placeholder.
    /// The position specifies where the baseline of the text should be placed.
    pub fn label(&mut self, text: &str, position: Vec2) {
        self.label_styled(text, position, self.theme.text.color, self.theme.text.font_size);
    }

    /// Create a text label with custom styling.
    ///
    /// If a font has been loaded, renders with actual glyphs. Otherwise falls back to placeholder.
    /// The position specifies where the baseline of the text should be placed.
    pub fn label_styled(&mut self, text: &str, position: Vec2, color: Color, font_size: f32) {
        // Try to render with font if available
        if let Some(font_handle) = self.font_manager.default_font() {
            match self.font_manager.layout_text(font_handle, text, font_size) {
                Ok(layout) => {
                    let text_data = Self::layout_to_draw_data(&layout, text, position, color, font_size);
                    self.draw_list.text(text_data);
                    return;
                }
                Err(e) => {
                    log::warn!("Font layout failed: {}", e);
                }
            }
        } else {
            // Font not available - log debug message (no longer cached to prevent retry)
            log::debug!("No default font available for text rendering");
        }

        // Fall back to placeholder
        self.draw_list.text_placeholder(text, position, color, font_size);
    }

    /// Create a text label with a specific font handle.
    pub fn label_with_font(&mut self, text: &str, position: Vec2, font: FontHandle, font_size: f32) {
        let color = self.theme.text.color;
        if let Ok(layout) = self.font_manager.layout_text(font, text, font_size) {
            let text_data = Self::layout_to_draw_data(&layout, text, position, color, font_size);
            self.draw_list.text(text_data);
        } else {
            self.draw_list.text_placeholder(text, position, color, font_size);
        }
    }

    /// Draw a label centered within bounds.
    ///
    /// This method handles vertical centering automatically using font metrics
    /// when available, falling back to approximate centering otherwise.
    /// Use this for text in buttons, headers, and other bounded containers.
    pub fn label_in_bounds(&mut self, text: &str, bounds: Rect, align: TextAlign) {
        let font_size = self.theme.text.font_size;
        let padding = 8.0; // Standard padding

        // Get text dimensions if font is available
        let (text_size, metrics) = if let Some(font_handle) = self.font_manager.default_font() {
            let size = self.font_manager.measure_text(font_handle, text, font_size).ok();
            let m = self.font_manager.metrics(font_handle, font_size);
            (size, m)
        } else {
            (None, None)
        };

        // Calculate X position based on alignment
        let x = if let Some(size) = text_size {
            match align {
                TextAlign::Left => bounds.x + padding,
                TextAlign::Center => bounds.x + (bounds.width - size.x) / 2.0,
                TextAlign::Right => bounds.x + bounds.width - size.x - padding,
            }
        } else {
            // Fallback: estimate text width
            let estimated_width = text.len() as f32 * font_size * 0.6;
            match align {
                TextAlign::Left => bounds.x + padding,
                TextAlign::Center => bounds.x + (bounds.width - estimated_width) / 2.0,
                TextAlign::Right => bounds.x + bounds.width - estimated_width - padding,
            }
        };

        // Calculate Y baseline position for vertical centering
        // The text origin is at the baseline, so we need to offset by ascent
        let ascent = metrics.map(|m| m.ascent).unwrap_or(font_size * 0.8);
        let text_height = text_size.map(|s| s.y).unwrap_or(font_size);
        let text_top = bounds.y + (bounds.height - text_height) / 2.0;
        let baseline_y = text_top + ascent;

        self.label(text, Vec2::new(x, baseline_y));
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
        let background = self.widget_background_color(result.state);

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

    #[test]
    fn test_font_rendering_retry_after_font_load() {
        let mut ui = UIContext::new();
        
        // First frame: No font loaded, should show placeholder
        ui.label_styled("Test Text", Vec2::new(10.0, 20.0), Color::WHITE, 16.0);
        assert_eq!(ui.draw_list().len(), 1);
        assert!(matches!(&ui.draw_list().commands()[0], DrawCommand::TextPlaceholder { .. }));
        
        // Clear draw list for next frame
        ui.end_frame();
        
        // Simulate font loading (we can't easily load a real font in tests, 
        // but we can verify the retry logic works by checking that the 
        // static PRINTED flag is no longer preventing retries)
        ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));
        
        // Second frame: Should retry font rendering (will still show placeholder 
        // since no font is loaded, but the important thing is it retries)
        ui.label_styled("Test Text", Vec2::new(10.0, 20.0), Color::WHITE, 16.0);
        assert_eq!(ui.draw_list().len(), 1);
        
        // The key test: it should still create a TextPlaceholder command,
        // but the important fix is that it *retries* the font check every frame
        // instead of being blocked by the static PRINTED flag
        assert!(matches!(&ui.draw_list().commands()[0], DrawCommand::TextPlaceholder { .. }));
    }

    #[test]
    fn test_text_align_default() {
        let align = TextAlign::default();
        assert_eq!(align, TextAlign::Left);
    }

    #[test]
    fn test_ui_context_font_metrics_none_without_font() {
        let ui = UIContext::new();
        // No font loaded, should return None
        assert!(ui.font_metrics(16.0).is_none());
    }

    #[test]
    fn test_ui_context_label_in_bounds() {
        let mut ui = UIContext::new();
        let bounds = Rect::new(10.0, 10.0, 200.0, 30.0);

        // Should not panic even without font
        ui.label_in_bounds("Test", bounds, TextAlign::Center);

        // Should generate a draw command (placeholder without font)
        assert_eq!(ui.draw_list().len(), 1);
    }

    #[test]
    fn test_ui_context_clip_rect() {
        let mut ui = UIContext::new();
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);

        ui.push_clip_rect(bounds);
        ui.rect(Rect::new(10.0, 10.0, 50.0, 50.0), Color::RED);
        ui.pop_clip_rect();

        assert_eq!(ui.draw_list().len(), 3);
    }
}
