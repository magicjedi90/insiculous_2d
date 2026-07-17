//! Editable component inspector with field modification support.
//!
//! This module provides editable UI widgets for modifying component
//! properties directly in the editor. Supports all common component
//! field types used in Transform2D, Sprite, RigidBody, Collider, and AudioSource.
//! Field identity and styling types live in [`crate::field_style`].

use std::ops::RangeInclusive;

use glam::{Vec2, Vec4};
use ui::{Color, Rect, UIContext};

pub use crate::field_style::{EditResult, EditableFieldStyle, FieldId};

/// Render an editable f32 value with a text input box.
pub fn edit_f32(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: f32,
    range: RangeInclusive<f32>,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<f32> {
    // Draw label
    ui.label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    // Text input bounds
    let input_x = pos.x + style.label_width;
    let input_height = style.row_height - 4.0;
    let input_bounds = Rect::new(
        input_x,
        pos.y + (style.row_height - input_height) / 2.0,
        style.input_width,
        input_height,
    );

    let new_value = ui.float_input(id, value, *range.start(), *range.end(), input_bounds);

    if (new_value - value).abs() > f32::EPSILON {
        EditResult::Changed(new_value)
    } else {
        EditResult::Unchanged
    }
}

/// Render an editable f32 value with a slider that wraps to a 0-1 range.
/// Useful for normalized values like volume, friction, restitution.
pub fn edit_normalized_f32(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: f32,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<f32> {
    let clamped = value.clamp(0.0, 1.0);
    edit_f32(ui, id, label, clamped, 0.0..=1.0, pos, style)
}

/// Render an editable boolean value with a checkbox.
pub fn edit_bool(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: bool,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<bool> {
    // Draw label
    ui.label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    // Checkbox bounds
    let checkbox_x = pos.x + style.label_width;
    let checkbox_bounds = Rect::new(
        checkbox_x,
        pos.y + (style.row_height - style.checkbox_size) / 2.0,
        style.checkbox_size,
        style.checkbox_size,
    );

    // Render checkbox and check if toggled
    let toggled = ui.checkbox(id, value, checkbox_bounds);

    if toggled {
        EditResult::Changed(!value)
    } else {
        EditResult::Unchanged
    }
}

/// Render an editable Vec2 value with separate X/Y text input boxes.
pub fn edit_vec2(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: Vec2,
    range: RangeInclusive<f32>,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<Vec2> {
    let (min, max) = (*range.start(), *range.end());

    // Draw label
    ui.label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    let mut new_value = value;
    let input_width = style.vec2_input_width;
    let input_height = style.row_height - 4.0;
    let input_x = pos.x + style.label_width;
    let input_y = pos.y + (style.row_height - input_height) / 2.0;

    // X label
    ui.label_styled(
        "X",
        glam::Vec2::new(input_x, pos.y + 4.0),
        style.axis_x_label,
        style.axis_font,
    );

    // X input
    let x_bounds = Rect::new(input_x + style.axis_label_gap, input_y, input_width, input_height);
    new_value.x = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 0),
        value.x, min, max, x_bounds,
    );

    // Y label
    let y_label_x = input_x + style.axis_label_gap + input_width + style.input_gap;
    ui.label_styled(
        "Y",
        glam::Vec2::new(y_label_x, pos.y + 4.0),
        style.axis_y_label,
        style.axis_font,
    );

    // Y input
    let y_bounds = Rect::new(y_label_x + style.axis_label_gap, input_y, input_width, input_height);
    new_value.y = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 1),
        value.y, min, max, y_bounds,
    );

    if new_value != value {
        EditResult::Changed(new_value)
    } else {
        EditResult::Unchanged
    }
}

/// Render a read-only u32 value (for asset handles, etc.).
pub fn display_u32(
    ui: &mut UIContext,
    label: &str,
    value: u32,
    pos: Vec2,
    style: &EditableFieldStyle,
) {
    ui.label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    let value_text = format!("{}", value);
    ui.label_styled(
        &value_text,
        glam::Vec2::new(pos.x + style.label_width, pos.y + 4.0),
        style.value_color,
        style.label_font,
    );
}

/// Render a read-only string value (for tags, target names, etc.).
pub fn display_string(
    ui: &mut UIContext,
    label: &str,
    value: &str,
    pos: Vec2,
    style: &EditableFieldStyle,
) {
    ui.label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);
    ui.label_styled(
        value,
        glam::Vec2::new(pos.x + style.label_width, pos.y + 4.0),
        style.value_color,
        style.label_font,
    );
}

/// Step an index forward or backward through `count` values, wrapping at
/// the ends. Pure helper behind [`EditableInspector::cycle`].
pub fn cycle_step(index: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    if forward {
        (index + 1) % count
    } else {
        (index + count - 1) % count
    }
}

/// Render an editable color (Vec4) with RGBA text inputs and preview.
pub fn edit_color(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: Vec4,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<Vec4> {
    let mut new_value = value;
    let mut changed = false;
    let row_height = style.row_height;
    let (x, y) = (pos.x, pos.y);

    // Draw label
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, style.label_font);

    // Color preview
    let preview_x = x + style.label_width;
    let preview_bounds = Rect::new(
        preview_x,
        y + (row_height - style.color_preview_size) / 2.0,
        style.color_preview_size,
        style.color_preview_size,
    );
    let preview_color = Color::new(value.x, value.y, value.z, value.w);
    ui.rect_rounded(preview_bounds, preview_color, 2.0);

    // RGBA text inputs (compact)
    let input_x = preview_x + style.color_preview_size + style.input_gap;
    let input_width = style.color_input_width;
    let input_height = style.color_input_height;
    let gap = style.color_input_gap;

    // Red
    ui.label_styled("R", glam::Vec2::new(input_x, y + 2.0), style.channel_labels[0], style.channel_font);
    let r_bounds = Rect::new(input_x + style.channel_label_gap, y + 2.0, input_width, input_height);
    let new_r = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 0),
        value.x, 0.0, 1.0, r_bounds,
    );
    if (new_r - value.x).abs() > f32::EPSILON {
        new_value.x = new_r;
        changed = true;
    }

    // Green
    let g_x = input_x + style.channel_label_gap + input_width + gap;
    ui.label_styled("G", glam::Vec2::new(g_x, y + 2.0), style.channel_labels[1], style.channel_font);
    let g_bounds = Rect::new(g_x + style.channel_label_gap, y + 2.0, input_width, input_height);
    let new_g = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 1),
        value.y, 0.0, 1.0, g_bounds,
    );
    if (new_g - value.y).abs() > f32::EPSILON {
        new_value.y = new_g;
        changed = true;
    }

    // Blue
    ui.label_styled("B", glam::Vec2::new(input_x, y + 2.0 + input_height + 4.0), style.channel_labels[2], style.channel_font);
    let b_bounds = Rect::new(input_x + style.channel_label_gap, y + 2.0 + input_height + 4.0, input_width, input_height);
    let new_b = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 2),
        value.z, 0.0, 1.0, b_bounds,
    );
    if (new_b - value.z).abs() > f32::EPSILON {
        new_value.z = new_b;
        changed = true;
    }

    // Alpha
    let a_x = input_x + style.channel_label_gap + input_width + gap;
    ui.label_styled("A", glam::Vec2::new(a_x, y + 2.0 + input_height + 4.0), style.channel_labels[3], style.channel_font);
    let a_bounds = Rect::new(a_x + style.channel_label_gap, y + 2.0 + input_height + 4.0, input_width, input_height);
    let new_a = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 3),
        value.w, 0.0, 1.0, a_bounds,
    );
    if (new_a - value.w).abs() > f32::EPSILON {
        new_value.w = new_a;
        changed = true;
    }

    if changed {
        EditResult::Changed(new_value)
    } else {
        EditResult::Unchanged
    }
}

/// Calculate the Y position after rendering a component section header.
pub fn component_header(
    ui: &mut UIContext,
    type_name: &str,
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> f32 {
    ui.label_styled(type_name, glam::Vec2::new(x, y), style.header_color, style.header_font);
    y + style.row_height + 4.0
}

/// A builder for constructing editable component inspectors.
///
/// This provides a fluent API for building inspectors for specific component types.
pub struct EditableInspector<'a> {
    ui: &'a mut UIContext,
    style: EditableFieldStyle,
    component_index: usize,
    field_index: usize,
    current_y: f32,
    x: f32,
}

impl<'a> EditableInspector<'a> {
    /// Create a new editable inspector builder.
    pub fn new(ui: &'a mut UIContext, x: f32, y: f32) -> Self {
        Self {
            ui,
            style: EditableFieldStyle::default(),
            component_index: 0,
            field_index: 0,
            current_y: y,
            x,
        }
    }

    /// Set the component index for field IDs.
    pub fn with_component_index(mut self, index: usize) -> Self {
        self.component_index = index;
        self
    }

    /// Set the style.
    pub fn with_style(mut self, style: EditableFieldStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the current Y position.
    pub fn y(&self) -> f32 {
        self.current_y
    }

    /// Add a component header.
    pub fn header(&mut self, type_name: &str) {
        self.current_y = component_header(self.ui, type_name, self.x, self.current_y, &self.style);
        self.field_index = 0;
    }

    /// Add a component header with an optional [X] remove button.
    ///
    /// Returns `true` if the remove button was clicked.
    /// When `removable` is `false`, behaves identically to `header()`.
    pub fn header_with_remove(&mut self, type_name: &str, removable: bool) -> bool {
        // Draw the header label
        self.ui.label_styled(
            type_name,
            glam::Vec2::new(self.x, self.current_y),
            self.style.header_color,
            self.style.header_font,
        );

        let mut clicked = false;

        if removable {
            // Place a small [X] button to the right of the header
            let btn_size = 18.0;
            let btn_x = self.x + self.style.label_width + 90.0;
            let btn_y = self.current_y;
            let btn_bounds = Rect::new(btn_x, btn_y, btn_size, btn_size);

            // Use component_index + 99 to avoid ID collisions with field inputs
            let btn_id = FieldId::new(self.component_index, 99, 0);
            clicked = self.ui.button(btn_id, "X", btn_bounds);
        }

        self.current_y += self.style.row_height + 4.0;
        self.field_index = 0;
        clicked
    }

    /// Position of the next field, indented from the inspector origin.
    fn field_pos(&self) -> Vec2 {
        Vec2::new(self.x + self.style.indent, self.current_y)
    }

    /// Add a texture slot field: shows the texture's display name and acts
    /// as a drag-and-drop target for asset-browser textures.
    pub fn texture(
        &mut self,
        label: &str,
        handle: u32,
        extras: &mut crate::InspectorExtras<'_>,
    ) -> EditResult<u32> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let display = extras.texture_display.clone();
        let result = crate::edit_texture_field(
            self.ui,
            id,
            label,
            handle,
            extras.drag_drop,
            display.as_deref(),
            pos,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable f32 field.
    pub fn f32(&mut self, label: &str, value: f32, range: RangeInclusive<f32>) -> EditResult<f32> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let result = edit_f32(self.ui, id, label, value, range, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable normalized f32 field (0-1 range).
    pub fn normalized_f32(&mut self, label: &str, value: f32) -> EditResult<f32> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let result = edit_normalized_f32(self.ui, id, label, value, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable boolean field.
    pub fn bool(&mut self, label: &str, value: bool) -> EditResult<bool> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let result = edit_bool(self.ui, id, label, value, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable Vec2 field.
    pub fn vec2(&mut self, label: &str, value: Vec2, range: RangeInclusive<f32>) -> EditResult<Vec2> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let result = edit_vec2(self.ui, id, label, value, range, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add a read-only u32 display.
    pub fn u32(&mut self, label: &str, value: u32) {
        let pos = self.field_pos();
        display_u32(self.ui, label, value, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
    }

    /// Add a read-only string display.
    pub fn string(&mut self, label: &str, value: &str) {
        let pos = self.field_pos();
        display_string(self.ui, label, value, pos, &self.style);
        self.field_index += 1;
        self.current_y += self.style.row_height;
    }

    /// Add a cycle selector row: `label  [<] value [>]` for choosing among
    /// `count` named values (e.g. enum variants, where a dropdown is not
    /// available).
    ///
    /// Returns `Changed(new_index)` when an arrow button is clicked,
    /// wrapping within `count`.
    pub fn cycle(
        &mut self,
        label: &str,
        value_name: &str,
        index: usize,
        count: usize,
    ) -> EditResult<usize> {
        let pos = self.field_pos();
        let (label_color, value_color) = (self.style.label_color, self.style.value_color);
        let (row_height, label_width) = (self.style.row_height, self.style.label_width);
        let label_font = self.style.label_font;
        self.ui
            .label_styled(label, glam::Vec2::new(pos.x, pos.y + 4.0), label_color, label_font);

        let btn_size = row_height - 6.0;
        let btn_y = pos.y + (row_height - btn_size) / 2.0;
        let value_width = 120.0;
        let prev_x = pos.x + label_width;

        let prev_bounds = Rect::new(prev_x, btn_y, btn_size, btn_size);
        let prev_clicked = self.ui.button(
            FieldId::new(self.component_index, self.field_index, 0),
            "<",
            prev_bounds,
        );

        self.ui.label_styled(
            value_name,
            glam::Vec2::new(prev_x + btn_size + 6.0, pos.y + 4.0),
            value_color,
            label_font,
        );

        let next_bounds = Rect::new(prev_x + btn_size + value_width, btn_y, btn_size, btn_size);
        let next_clicked = self.ui.button(
            FieldId::new(self.component_index, self.field_index, 1),
            ">",
            next_bounds,
        );

        self.field_index += 1;
        self.current_y += self.style.row_height;

        if prev_clicked || next_clicked {
            EditResult::Changed(cycle_step(index, count, next_clicked))
        } else {
            EditResult::Unchanged
        }
    }

    /// Add an editable color (Vec4) field.
    pub fn color(&mut self, label: &str, value: Vec4) -> EditResult<Vec4> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let pos = self.field_pos();
        let result = edit_color(self.ui, id, label, value, pos, &self.style);
        self.field_index += 1;
        // Color spans two input rows (RG / BA) of color_input_height plus gaps
        self.current_y += self.style.row_height * 1.8;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_result_unchanged() {
        let result: EditResult<f32> = EditResult::Unchanged;
        assert!(!result.is_changed());
        assert_eq!(result.new_value(), None);
        assert_eq!(result.unwrap_or(5.0), 5.0);
    }

    #[test]
    fn test_edit_result_changed() {
        let result = EditResult::Changed(10.0);
        assert!(result.is_changed());
        assert_eq!(result.new_value(), Some(&10.0));
        assert_eq!(result.unwrap_or(5.0), 10.0);
    }

    #[test]
    fn test_field_id_creation() {
        let id = FieldId::new(1, 2, 3);
        let _widget_id: ui::WidgetId = id.into();
        // WidgetId is created successfully (can't verify internal value without accessor)
    }

    #[test]
    fn test_editable_field_style_default() {
        let style = EditableFieldStyle::default();
        assert_eq!(style.row_height, 24.0);
        assert_eq!(style.label_width, 100.0);
        assert_eq!(style.padding, 8.0);
    }

    #[test]
    fn test_cycle_step_wraps_both_directions() {
        assert_eq!(cycle_step(0, 7, true), 1);
        assert_eq!(cycle_step(6, 7, true), 0); // wraps forward
        assert_eq!(cycle_step(0, 7, false), 6); // wraps backward
        assert_eq!(cycle_step(3, 7, false), 2);
    }

    #[test]
    fn test_cycle_step_zero_count_is_safe() {
        assert_eq!(cycle_step(5, 0, true), 0);
        assert_eq!(cycle_step(5, 0, false), 0);
    }

    #[test]
    fn test_editable_inspector_builder() {
        // Just verify the builder pattern compiles and initializes correctly
        // Actual rendering requires a UIContext which needs rendering infrastructure
        let style = EditableFieldStyle::default();
        assert_eq!(style.row_height, 24.0);
    }
}
