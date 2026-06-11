//! Interactive widgets for [`UIContext`]: buttons, sliders, checkboxes,
//! float inputs, plus panel and progress-bar containers.

use glam::Vec2;

use crate::{Color, Rect, WidgetId, WidgetState};

use super::{TextAlign, UIContext};

impl UIContext {
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
        let border = style.border;
        let border_width = style.border_width;
        let corner_radius = style.corner_radius;

        // Draw button background
        self.draw_list.rect_rounded(bounds, background, corner_radius);

        // Draw border
        if border_width > 0.0 {
            self.draw_list
                .rect_border_rounded(bounds, border, border_width, corner_radius);
        }

        // Draw label centered in the button
        let font_size = self.theme.text.font_size;
        let text_pos = self.text_pos_in_bounds(label, bounds, TextAlign::Center, font_size, 0.0);
        self.draw_text_at_baseline(label, text_pos, text_color, font_size);

        result.clicked
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

    /// Create a float text input field.
    ///
    /// Displays the value as text in a box. Click to focus and type a new value.
    /// Accepts digits, period, and minus sign. Enter/Tab commits, Escape cancels.
    /// The committed value is clamped to the min/max range.
    ///
    /// Returns the current value (unchanged while editing, new value on commit).
    pub fn float_input(
        &mut self,
        id: impl Into<WidgetId>,
        value: f32,
        min: f32,
        max: f32,
        bounds: Rect,
    ) -> f32 {
        let id = id.into();
        let result = self.interaction.interact(id, bounds, true);
        let is_focused = self.interaction.is_focused(id);

        // Snapshot keyboard state before mutating persistent state
        let typed_chars = self.interaction.input().typed_chars.clone();
        let enter = self.interaction.input().enter_pressed;
        let escape = self.interaction.input().escape_pressed;
        let backspace = self.interaction.input().backspace_pressed;
        let tab = self.interaction.input().tab_pressed;
        let mouse_just_pressed = self.interaction.input().mouse_just_pressed;
        let mouse_in_bounds = bounds.contains(self.interaction.input().mouse_pos);

        if result.clicked && !is_focused {
            // Enter edit mode
            self.interaction.set_focus(id);
            let state = self.interaction.get_state(id);
            state.string_value = format!("{:.2}", value);
        }

        if self.interaction.is_focused(id) {
            // Process keyboard input
            let state = self.interaction.get_state(id);

            for ch in &typed_chars {
                state.string_value.push(*ch);
            }

            if backspace {
                state.string_value.pop();
            }

            // Commit on Enter, Tab, or click outside
            if enter || tab || (mouse_just_pressed && !mouse_in_bounds) {
                return self.commit_float_input(id, value, min, max, bounds);
            }

            // Cancel on Escape
            if escape {
                self.interaction.clear_focus();
                return self.draw_float_value(bounds, value, false);
            }

            // Draw focused state with editing text
            let edit_text = self.interaction.get_state(id).string_value.clone();
            self.draw_float_input_box(bounds, &edit_text, true);
            return value; // Return original while editing
        }

        // Not focused — draw display value
        let hovered = result.state == WidgetState::Hovered;
        self.draw_float_value(bounds, value, hovered)
    }

    /// Commit the edit buffer of a float input: parse (falling back to the
    /// pre-edit value), clamp, unfocus, and draw the committed value.
    fn commit_float_input(&mut self, id: WidgetId, fallback: f32, min: f32, max: f32, bounds: Rect) -> f32 {
        let new_value = self.interaction.get_state(id).string_value
            .parse::<f32>()
            .unwrap_or(fallback)
            .clamp(min, max);
        self.interaction.clear_focus();
        self.draw_float_value(bounds, new_value, false)
    }

    /// Draw a float input showing a numeric value; returns the value for
    /// tail-call convenience.
    fn draw_float_value(&mut self, bounds: Rect, value: f32, highlighted: bool) -> f32 {
        self.draw_float_input_box(bounds, &format!("{:.2}", value), highlighted);
        value
    }

    /// Draw a float input text box (shared by focused and unfocused states).
    fn draw_float_input_box(&mut self, bounds: Rect, text: &str, highlighted: bool) {
        let style = self.theme.text_input.clone();
        let bg = if highlighted { style.background_focused } else { style.background };
        let border = if highlighted { style.border_focused } else { style.border };

        self.draw_list.rect_rounded(bounds, bg, style.corner_radius);
        self.draw_list
            .rect_border_rounded(bounds, border, style.border_width, style.corner_radius);

        let text_pos =
            self.text_pos_in_bounds(text, bounds, TextAlign::Left, style.font_size, style.padding);
        self.draw_text_at_baseline(text, text_pos, style.text_color, style.font_size);
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
            position.x + checkbox_size + self.theme.button.padding,
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
}
