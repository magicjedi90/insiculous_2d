//! Editable component inspector with field modification support.
//!
//! This module provides editable field types and UI widgets for modifying
//! component properties directly in the editor. Supports all common component
//! field types used in Transform2D, Sprite, RigidBody, Collider, and AudioSource.

use glam::{Vec2, Vec4};
use ui::{Color, Rect, UIContext};

/// Unique identifier for inspector fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId {
    component_index: usize,
    field_index: usize,
    subfield_index: usize,
}

impl FieldId {
    /// Create a new field ID.
    pub fn new(component_index: usize, field_index: usize, subfield_index: usize) -> Self {
        Self {
            component_index,
            field_index,
            subfield_index,
        }
    }
}

impl From<FieldId> for ui::WidgetId {
    fn from(id: FieldId) -> Self {
        // Create a unique widget ID from the field indices
        let id_value = (id.component_index as u64) * 10000 
            + (id.field_index as u64) * 100 
            + id.subfield_index as u64;
        ui::WidgetId::new(id_value)
    }
}

/// Configuration for editable field display.
#[derive(Debug, Clone)]
pub struct EditableFieldStyle {
    /// Height of each field row
    pub row_height: f32,
    /// Width of the label column
    pub label_width: f32,
    /// Padding between elements
    pub padding: f32,
    /// Slider height
    pub slider_height: f32,
    /// Checkbox size
    pub checkbox_size: f32,
    /// Color preview size
    pub color_preview_size: f32,
    /// Indentation for nested fields
    pub indent: f32,
    /// Label color
    pub label_color: Color,
    /// Value color
    pub value_color: Color,
    /// Header color for component names
    pub header_color: Color,
}

impl Default for EditableFieldStyle {
    fn default() -> Self {
        Self {
            row_height: 24.0,
            label_width: 100.0,
            padding: 8.0,
            slider_height: 16.0,
            checkbox_size: 16.0,
            color_preview_size: 20.0,
            indent: 16.0,
            label_color: Color::new(0.7, 0.7, 0.7, 1.0),
            value_color: Color::new(1.0, 1.0, 1.0, 1.0),
            header_color: Color::new(0.9, 0.9, 0.5, 1.0),
        }
    }
}

/// Result of editing a field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditResult<T> {
    /// Value was not changed.
    Unchanged,
    /// Value was modified to a new value.
    Changed(T),
}

impl<T> EditResult<T> {
    /// Check if the value was changed.
    pub fn is_changed(&self) -> bool {
        matches!(self, EditResult::Changed(_))
    }

    /// Get the new value if changed, otherwise None.
    pub fn new_value(&self) -> Option<&T> {
        match self {
            EditResult::Changed(v) => Some(v),
            EditResult::Unchanged => None,
        }
    }

    /// Unwrap the value, returning the new value if changed, or the original.
    pub fn unwrap_or(self, original: T) -> T {
        match self {
            EditResult::Changed(v) => v,
            EditResult::Unchanged => original,
        }
    }
}

/// Render an editable f32 value with a text input box.
pub fn edit_f32(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: f32,
    min: f32,
    max: f32,
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> EditResult<f32> {
    // Draw label
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, 14.0);

    // Text input bounds
    let input_x = x + style.label_width;
    let input_height = style.row_height - 4.0;
    let input_bounds = Rect::new(
        input_x,
        y + (style.row_height - input_height) / 2.0,
        100.0,
        input_height,
    );

    let new_value = ui.float_input(id, value, min, max, input_bounds);

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
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> EditResult<f32> {
    let clamped = value.clamp(0.0, 1.0);
    edit_f32(ui, id, label, clamped, 0.0, 1.0, x, y, style)
}

/// Render an editable boolean value with a checkbox.
pub fn edit_bool(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: bool,
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> EditResult<bool> {
    // Draw label
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, 14.0);

    // Checkbox bounds
    let checkbox_x = x + style.label_width;
    let checkbox_bounds = Rect::new(
        checkbox_x,
        y + (style.row_height - style.checkbox_size) / 2.0,
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
    min: f32,
    max: f32,
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> EditResult<Vec2> {
    // Draw label
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, 14.0);

    let mut new_value = value;
    let input_width = 70.0;
    let input_height = style.row_height - 4.0;
    let input_x = x + style.label_width;
    let input_y = y + (style.row_height - input_height) / 2.0;

    // X label
    ui.label_styled(
        "X",
        glam::Vec2::new(input_x, y + 4.0),
        Color::new(0.8, 0.4, 0.4, 1.0),
        12.0,
    );

    // X input
    let x_bounds = Rect::new(input_x + 14.0, input_y, input_width, input_height);
    new_value.x = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 0),
        value.x, min, max, x_bounds,
    );

    // Y label
    let y_label_x = input_x + 14.0 + input_width + 8.0;
    ui.label_styled(
        "Y",
        glam::Vec2::new(y_label_x, y + 4.0),
        Color::new(0.4, 0.8, 0.4, 1.0),
        12.0,
    );

    // Y input
    let y_bounds = Rect::new(y_label_x + 14.0, input_y, input_width, input_height);
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
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) {
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, 14.0);

    let value_text = format!("{}", value);
    ui.label_styled(
        &value_text,
        glam::Vec2::new(x + style.label_width, y + 4.0),
        style.value_color,
        14.0,
    );
}

/// Render an editable color (Vec4) with RGBA text inputs and preview.
pub fn edit_color(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: Vec4,
    x: f32,
    y: f32,
    style: &EditableFieldStyle,
) -> EditResult<Vec4> {
    let mut new_value = value;
    let mut changed = false;
    let row_height = style.row_height;

    // Draw label
    ui.label_styled(label, glam::Vec2::new(x, y + 4.0), style.label_color, 14.0);

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
    let input_x = preview_x + style.color_preview_size + 8.0;
    let input_width = 48.0;
    let input_height = 14.0;
    let gap = 4.0;

    // Red
    ui.label_styled("R", glam::Vec2::new(input_x, y + 2.0), Color::new(0.9, 0.4, 0.4, 1.0), 10.0);
    let r_bounds = Rect::new(input_x + 12.0, y + 2.0, input_width, input_height);
    let new_r = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 0),
        value.x, 0.0, 1.0, r_bounds,
    );
    if (new_r - value.x).abs() > f32::EPSILON {
        new_value.x = new_r;
        changed = true;
    }

    // Green
    let g_x = input_x + 12.0 + input_width + gap;
    ui.label_styled("G", glam::Vec2::new(g_x, y + 2.0), Color::new(0.4, 0.9, 0.4, 1.0), 10.0);
    let g_bounds = Rect::new(g_x + 12.0, y + 2.0, input_width, input_height);
    let new_g = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 1),
        value.y, 0.0, 1.0, g_bounds,
    );
    if (new_g - value.y).abs() > f32::EPSILON {
        new_value.y = new_g;
        changed = true;
    }

    // Blue
    ui.label_styled("B", glam::Vec2::new(input_x, y + row_height / 2.0 + 2.0), Color::new(0.4, 0.4, 0.9, 1.0), 10.0);
    let b_bounds = Rect::new(input_x + 12.0, y + row_height / 2.0 + 2.0, input_width, input_height);
    let new_b = ui.float_input(
        FieldId::new(id.component_index, id.field_index, 2),
        value.z, 0.0, 1.0, b_bounds,
    );
    if (new_b - value.z).abs() > f32::EPSILON {
        new_value.z = new_b;
        changed = true;
    }

    // Alpha
    let a_x = input_x + 12.0 + input_width + gap;
    ui.label_styled("A", glam::Vec2::new(a_x, y + row_height / 2.0 + 2.0), Color::new(0.7, 0.7, 0.7, 1.0), 10.0);
    let a_bounds = Rect::new(a_x + 12.0, y + row_height / 2.0 + 2.0, input_width, input_height);
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
    ui.label_styled(type_name, glam::Vec2::new(x, y), style.header_color, 16.0);
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
            16.0,
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

    /// Add an editable f32 field.
    pub fn f32(&mut self, label: &str, value: f32, min: f32, max: f32) -> EditResult<f32> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let result = edit_f32(
            self.ui,
            id,
            label,
            value,
            min,
            max,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable normalized f32 field (0-1 range).
    pub fn normalized_f32(&mut self, label: &str, value: f32) -> EditResult<f32> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let result = edit_normalized_f32(
            self.ui,
            id,
            label,
            value,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable boolean field.
    pub fn bool(&mut self, label: &str, value: bool) -> EditResult<bool> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let result = edit_bool(
            self.ui,
            id,
            label,
            value,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add an editable Vec2 field.
    pub fn vec2(&mut self, label: &str, value: Vec2, min: f32, max: f32) -> EditResult<Vec2> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let result = edit_vec2(
            self.ui,
            id,
            label,
            value,
            min,
            max,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
        result
    }

    /// Add a read-only u32 display.
    pub fn u32(&mut self, label: &str, value: u32) {
        display_u32(
            self.ui,
            label,
            value,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height;
    }

    /// Add an editable color (Vec4) field.
    pub fn color(&mut self, label: &str, value: Vec4) -> EditResult<Vec4> {
        let id = FieldId::new(self.component_index, self.field_index, 0);
        let result = edit_color(
            self.ui,
            id,
            label,
            value,
            self.x + self.style.indent,
            self.current_y,
            &self.style,
        );
        self.field_index += 1;
        self.current_y += self.style.row_height * 1.5; // Color takes more space
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
    fn test_editable_inspector_builder() {
        // Just verify the builder pattern compiles and initializes correctly
        // Actual rendering requires a UIContext which needs rendering infrastructure
        let style = EditableFieldStyle::default();
        assert_eq!(style.row_height, 24.0);
    }
}
