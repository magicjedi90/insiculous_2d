//! String field widgets for the editable inspector: an editable text input
//! plus the read-only string/u32 displays (moved from `editable_inspector.rs`
//! for file size).

use glam::Vec2;
use ui::{Rect, UIContext};

use crate::field_style::{EditResult, EditableFieldStyle, FieldId};

/// Render an editable string field (label + free-form text input).
///
/// Commits on Enter/Tab/click-away, cancels on Escape — the semantics of
/// `UIContext::text_input`. Returns `Changed` only when the committed text
/// differs from the current value.
pub fn edit_string(
    ui: &mut UIContext,
    id: FieldId,
    label: &str,
    value: &str,
    pos: Vec2,
    style: &EditableFieldStyle,
) -> EditResult<String> {
    // Draw label
    ui.label_styled(label, Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    // Text input bounds — wider than numeric inputs; strings are longer.
    let input_x = pos.x + style.label_width;
    let input_height = style.row_height - 4.0;
    let input_bounds = Rect::new(
        input_x,
        pos.y + (style.row_height - input_height) / 2.0,
        style.input_width * 1.6,
        input_height,
    );

    match ui.text_input(id, value, input_bounds) {
        Some(new_value) if new_value != value => EditResult::Changed(new_value),
        _ => EditResult::Unchanged,
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
    ui.label_styled(label, Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);

    let value_text = format!("{}", value);
    ui.label_styled(
        &value_text,
        Vec2::new(pos.x + style.label_width, pos.y + 4.0),
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
    ui.label_styled(label, Vec2::new(pos.x, pos.y + 4.0), style.label_color, style.label_font);
    ui.label_styled(
        value,
        Vec2::new(pos.x + style.label_width, pos.y + 4.0),
        style.value_color,
        style.label_font,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_string_without_interaction_is_unchanged() {
        let mut ui = UIContext::new();
        ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));
        let result = edit_string(
            &mut ui,
            FieldId::new(0, 0, 0),
            "Text",
            "hello",
            Vec2::new(10.0, 10.0),
            &EditableFieldStyle::default(),
        );
        ui.end_frame();
        assert!(!result.is_changed());
    }
}
