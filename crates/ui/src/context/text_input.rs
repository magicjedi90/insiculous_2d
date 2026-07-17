//! Numeric text-input widget with a real editing model: click-to-focus
//! selects the whole value, a visible cursor, arrow/Home/End navigation,
//! shift-selection, and editing at the cursor position.
//!
//! The editing rules live in [`crate::TextEditState`]; this file translates
//! input-state flags into edit calls and draws the box/selection/caret.

use crate::{Rect, TextEditState, WidgetId, WidgetState};

use super::{TextAlign, UIContext};

/// Caret width in pixels.
const CARET_WIDTH: f32 = 1.0;

impl UIContext {
    /// Create a float text input field.
    ///
    /// Click to focus: the whole value is selected so typing overwrites it.
    /// Click again to place the cursor; arrows/Home/End move it (shift
    /// extends the selection), Backspace/Delete edit at the cursor, and
    /// held keys repeat. Enter/Tab or clicking outside commits (clamped to
    /// `min..=max`, parse failures fall back to the pre-edit value);
    /// Escape cancels.
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
        let was_focused = self.interaction.is_focused(id);

        // Snapshot keyboard/mouse state before mutating persistent state
        let input = self.interaction.input().clone();
        let mouse_in_bounds = bounds.contains(input.mouse_pos);
        let padding = self.theme.text_input.padding;
        let font_size = self.theme.text_input.font_size;

        if result.clicked && !was_focused {
            // Enter edit mode with the whole value selected — typing replaces it
            self.interaction.set_focus(id);
            let text = format!("{:.2}", value);
            self.interaction.get_state(id).edit.set_text_select_all(&text);
        } else if result.clicked && was_focused {
            // Click inside while editing: place the cursor at the click
            let text = self.interaction.get_state(id).edit.text.clone();
            let widths = self.prefix_widths(&text, font_size);
            let local_x = input.mouse_pos.x - (bounds.x + padding);
            self.interaction.get_state(id).edit.cursor_from_click(&widths, local_x);
        }

        if self.interaction.is_focused(id) {
            // Cancel on Escape
            if input.escape_pressed {
                self.interaction.clear_focus();
                return self.draw_float_value(bounds, value, false);
            }

            // Commit on Enter, Tab, or click outside
            if input.enter_pressed || input.tab_pressed || (input.mouse_just_pressed && !mouse_in_bounds) {
                return self.commit_float_input(id, value, min, max, bounds);
            }

            // Navigation, deletion, then typed characters — all cursor-aware
            let state = &mut self.interaction.get_state(id).edit;
            if input.left_pressed {
                state.move_left(input.shift_down);
            }
            if input.right_pressed {
                state.move_right(input.shift_down);
            }
            if input.home_pressed {
                state.home(input.shift_down);
            }
            if input.end_pressed {
                state.end(input.shift_down);
            }
            if input.backspace_pressed {
                state.backspace();
            }
            if input.delete_pressed {
                state.delete();
            }
            for ch in &input.typed_chars {
                state.insert_char(*ch);
            }

            let edit = self.interaction.get_state(id).edit.clone();
            self.draw_float_input_editing(bounds, &edit);
            return value; // Return original while editing
        }

        // Not focused — draw display value
        let hovered = result.state == WidgetState::Hovered;
        self.draw_float_value(bounds, value, hovered)
    }

    /// Commit the edit buffer of a float input: parse (falling back to the
    /// pre-edit value), clamp, unfocus, and draw the committed value.
    fn commit_float_input(&mut self, id: WidgetId, fallback: f32, min: f32, max: f32, bounds: Rect) -> f32 {
        let new_value = self.interaction.get_state(id).edit.text
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

    /// Pixel widths of every prefix of `text` at `font_size`:
    /// `result[i]` = width of the first `i` chars (so `len + 1` entries).
    /// Used to place the caret, the selection band, and click-to-cursor.
    fn prefix_widths(&self, text: &str, font_size: f32) -> Vec<f32> {
        let mut widths = Vec::with_capacity(text.chars().count() + 1);
        widths.push(0.0);
        let mut end = 0;
        for c in text.chars() {
            end += c.len_utf8();
            widths.push(self.measure_text_styled(&text[..end], font_size).x);
        }
        widths
    }

    /// Draw a focused float input: box, selection band, text, and caret,
    /// clipped to the bounds so long edits don't overflow.
    fn draw_float_input_editing(&mut self, bounds: Rect, edit: &TextEditState) {
        let style = self.theme.text_input.clone();

        self.draw_list.rect_rounded(bounds, style.background_focused, style.corner_radius);
        self.draw_list
            .rect_border_rounded(bounds, style.border_focused, style.border_width, style.corner_radius);

        self.push_clip_rect(bounds);

        let widths = self.prefix_widths(&edit.text, style.font_size);
        let text_origin_x = bounds.x + style.padding;
        // Vertical band for selection/caret: centered, sized from the font
        let band_height = (style.font_size * 1.2).min(bounds.height - 2.0);
        let band_y = bounds.y + (bounds.height - band_height) / 2.0;

        // Selection highlight behind the text
        if let Some((start, end)) = edit.selected_range() {
            let x0 = text_origin_x + widths[start.min(widths.len() - 1)];
            let x1 = text_origin_x + widths[end.min(widths.len() - 1)];
            self.draw_list.rect(
                Rect::new(x0, band_y, x1 - x0, band_height),
                style.selection_color,
            );
        }

        let text_pos =
            self.text_pos_in_bounds(&edit.text, bounds, TextAlign::Left, style.font_size, style.padding);
        self.draw_text_at_baseline(&edit.text, text_pos, style.text_color, style.font_size);

        // Caret at the cursor position
        let caret_x = text_origin_x + widths[edit.cursor.min(widths.len() - 1)];
        self.draw_list.rect(
            Rect::new(caret_x, band_y, CARET_WIDTH, band_height),
            style.cursor_color,
        );

        self.pop_clip_rect();
    }

    /// Draw a float input text box (shared by unfocused and committed states).
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
}
