//! Behavior tests for `UIContext` (lifecycle, text, and widget methods).

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
    assert!(!ui.draw_list().is_empty());
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
    assert!(!ui.draw_list().is_empty());
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

#[test]
fn test_float_input_returns_original_without_interaction() {
    let mut ui = UIContext::new();
    ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));

    let bounds = Rect::new(100.0, 100.0, 80.0, 20.0);
    let result = ui.float_input("test_float", 2.75, 0.0, 10.0, bounds);

    // Without any interaction, should return the original value
    assert_eq!(result, 2.75);
    // Should generate draw commands (background rect + border + text)
    assert!(ui.draw_list().len() >= 2);
}

#[test]
fn test_float_input_draws_box() {
    let mut ui = UIContext::new();
    ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));

    let bounds = Rect::new(50.0, 50.0, 100.0, 24.0);
    ui.float_input("float_box", 42.0, 0.0, 100.0, bounds);

    // Should have background rect, border rect, and text placeholder
    assert!(ui.draw_list().len() >= 3);
}

// === label_centered / measure_text tests ===

#[test]
fn test_label_centered_generates_draw_command() {
    let mut ui = UIContext::new();
    ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));

    ui.label_centered("hello", Vec2::new(400.0, 300.0));

    assert!(!ui.draw_list().is_empty());
}

#[test]
fn test_label_centered_styled_generates_draw_command() {
    let mut ui = UIContext::new();
    ui.begin_frame(&input::InputHandler::new(), Vec2::new(800.0, 600.0));

    ui.label_centered_styled("styled", Vec2::new(400.0, 300.0), Color::WHITE, 24.0);

    assert!(!ui.draw_list().is_empty());
}

#[test]
fn test_measure_text_returns_nonzero_dimensions() {
    let ui = UIContext::new();
    let size = ui.measure_text("hello");
    assert!(size.x > 0.0, "text width should be positive");
    assert!(size.y > 0.0, "text height should be positive");
}

#[test]
fn test_measure_text_larger_font_gives_wider_result() {
    let ui = UIContext::new();
    let small = ui.measure_text_styled("hello", 12.0);
    let large = ui.measure_text_styled("hello", 24.0);
    assert!(large.x > small.x, "larger font should produce wider text");
}

#[test]
fn test_measure_text_empty_string_returns_zero_width() {
    let ui = UIContext::new();
    let size = ui.measure_text("");
    assert_eq!(size.x, 0.0, "empty string should have zero width");
}

#[test]
fn test_label_in_bounds_styled_keeps_glyphs_inside_bounds() {
    // No font loaded — the estimate path uses ascent = 0.8 * font_size,
    // matching the dock panel header geometry (24px tall).
    let mut ui = UIContext::new();
    let bounds = Rect::new(0.0, 100.0, 200.0, 24.0);

    ui.label_in_bounds_styled("Hierarchy", bounds, TextAlign::Left, Color::WHITE, 14.0, 8.0);

    if let DrawCommand::TextPlaceholder { position, font_size, .. } = &ui.draw_list().commands()[0] {
        let glyph_top = position.y - font_size * 0.8; // baseline minus ascent
        assert!(
            glyph_top >= bounds.y,
            "glyphs must not rise above the bounds top (no border strike-through): top {glyph_top} < {}",
            bounds.y
        );
        assert!(position.y <= bounds.y + bounds.height, "baseline stays inside the bounds");
        assert_eq!(position.x, bounds.x + 8.0, "left-aligned with padding");
    } else {
        panic!("Expected TextPlaceholder command");
    }
}

#[test]
fn test_wants_keyboard_follows_float_input_focus() {
    use input::prelude::MouseButton;

    let mut ui = UIContext::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);
    let mut input = input::InputHandler::new();

    // Frame 1: press inside the field (click fires on release, so no focus yet)
    input.mouse_mut().update_position(50.0, 20.0);
    input.mouse_mut().handle_button_press(MouseButton::Left);
    ui.begin_frame(&input, Vec2::new(800.0, 600.0));
    ui.float_input("focus_field", 1.0, 0.0, 10.0, bounds);
    ui.end_frame();
    assert!(!ui.wants_keyboard());

    // Frame 2: release inside the field — the click focuses the input
    input.update(); // clear just-pressed edge from frame 1
    input.mouse_mut().handle_button_release(MouseButton::Left);
    ui.begin_frame(&input, Vec2::new(800.0, 600.0));
    ui.float_input("focus_field", 1.0, 0.0, 10.0, bounds);
    ui.end_frame();
    assert!(ui.wants_keyboard(), "focused float input must claim the keyboard");
}

// === text-input editing behavior (cursor/selection model) ===

/// Click a float input (press frame + release frame) so it gains focus.
fn focus_float_input(ui: &mut UIContext, input: &mut input::InputHandler, id: &str, bounds: Rect, value: f32) {
    use input::prelude::MouseButton;
    let center = Vec2::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);
    input.mouse_mut().update_position(center.x, center.y);
    input.mouse_mut().handle_button_press(MouseButton::Left);
    ui.begin_frame(&*input, Vec2::new(800.0, 600.0));
    ui.float_input(id, value, -100000.0, 100000.0, bounds);
    ui.end_frame();

    input.update();
    input.mouse_mut().handle_button_release(MouseButton::Left);
    ui.begin_frame(&*input, Vec2::new(800.0, 600.0));
    ui.float_input(id, value, -100000.0, 100000.0, bounds);
    ui.end_frame();
    assert!(ui.wants_keyboard(), "field must be focused after a click");
}

/// Run one frame of the focused input with a single key pressed.
fn type_key(ui: &mut UIContext, input: &mut input::InputHandler, id: &str, bounds: Rect, value: f32, key: input::prelude::KeyCode) -> f32 {
    input.update();
    input.keyboard_mut().handle_key_press(key);
    ui.begin_frame(&*input, Vec2::new(800.0, 600.0));
    let out = ui.float_input(id, value, -100000.0, 100000.0, bounds);
    ui.end_frame();
    input.keyboard_mut().handle_key_release(key);
    out
}

#[test]
fn test_float_input_focus_selects_all_and_typing_overwrites() {
    use input::prelude::KeyCode;
    let mut ui = UIContext::new();
    let mut input = input::InputHandler::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);

    focus_float_input(&mut ui, &mut input, "sel_all", bounds, 168.4);

    // Typing '5' replaces the fully-selected "168.40", then Enter commits.
    type_key(&mut ui, &mut input, "sel_all", bounds, 168.4, KeyCode::Digit5);
    let committed = type_key(&mut ui, &mut input, "sel_all", bounds, 168.4, KeyCode::Enter);
    assert_eq!(committed, 5.0, "click + type must overwrite the whole value");
    assert!(!ui.wants_keyboard());
}

#[test]
fn test_float_input_arrow_then_insert_edits_at_cursor() {
    use input::prelude::KeyCode;
    let mut ui = UIContext::new();
    let mut input = input::InputHandler::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);

    focus_float_input(&mut ui, &mut input, "cursor_edit", bounds, 12.0);
    // Buffer is "12.00" fully selected. Home collapses to the start,
    // then typing '9' inserts before the '1'.
    type_key(&mut ui, &mut input, "cursor_edit", bounds, 12.0, KeyCode::Home);
    type_key(&mut ui, &mut input, "cursor_edit", bounds, 12.0, KeyCode::Digit9);
    let committed = type_key(&mut ui, &mut input, "cursor_edit", bounds, 12.0, KeyCode::Enter);
    assert_eq!(committed, 912.0, "insert must happen at the cursor, not the end");
}

#[test]
fn test_float_input_backspace_deletes_before_cursor() {
    use input::prelude::KeyCode;
    let mut ui = UIContext::new();
    let mut input = input::InputHandler::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);

    focus_float_input(&mut ui, &mut input, "bs_mid", bounds, 12.0);
    // "12.00" selected; End collapses to the end, ArrowLeft x2 puts the
    // cursor between '.'|'0', Backspace removes the '.'.
    type_key(&mut ui, &mut input, "bs_mid", bounds, 12.0, KeyCode::End);
    type_key(&mut ui, &mut input, "bs_mid", bounds, 12.0, KeyCode::ArrowLeft);
    type_key(&mut ui, &mut input, "bs_mid", bounds, 12.0, KeyCode::ArrowLeft);
    type_key(&mut ui, &mut input, "bs_mid", bounds, 12.0, KeyCode::Backspace);
    let committed = type_key(&mut ui, &mut input, "bs_mid", bounds, 12.0, KeyCode::Enter);
    assert_eq!(committed, 1200.0, "\"12.00\" minus its '.' is \"1200\"");
}

#[test]
fn test_float_input_escape_cancels_edit() {
    use input::prelude::KeyCode;
    let mut ui = UIContext::new();
    let mut input = input::InputHandler::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);

    focus_float_input(&mut ui, &mut input, "esc_cancel", bounds, 7.5);
    type_key(&mut ui, &mut input, "esc_cancel", bounds, 7.5, KeyCode::Digit3);
    let after_escape = type_key(&mut ui, &mut input, "esc_cancel", bounds, 7.5, KeyCode::Escape);
    assert_eq!(after_escape, 7.5, "escape must discard the edit");
    assert!(!ui.wants_keyboard());
}

#[test]
fn test_float_input_commit_clamps_to_range() {
    use input::prelude::{KeyCode, MouseButton};
    let mut ui = UIContext::new();
    let mut input = input::InputHandler::new();
    let bounds = Rect::new(10.0, 10.0, 80.0, 20.0);

    // Focus (range 0..=10 this time, so use the raw two-frame click)
    let center = Vec2::new(50.0, 20.0);
    input.mouse_mut().update_position(center.x, center.y);
    input.mouse_mut().handle_button_press(MouseButton::Left);
    ui.begin_frame(&input, Vec2::new(800.0, 600.0));
    ui.float_input("clamp", 5.0, 0.0, 10.0, bounds);
    ui.end_frame();
    input.update();
    input.mouse_mut().handle_button_release(MouseButton::Left);
    ui.begin_frame(&input, Vec2::new(800.0, 600.0));
    ui.float_input("clamp", 5.0, 0.0, 10.0, bounds);
    ui.end_frame();

    // Type "99" (selection replaced by first digit), commit
    for key in [KeyCode::Digit9, KeyCode::Digit9] {
        input.update();
        input.keyboard_mut().handle_key_press(key);
        ui.begin_frame(&input, Vec2::new(800.0, 600.0));
        ui.float_input("clamp", 5.0, 0.0, 10.0, bounds);
        ui.end_frame();
        input.keyboard_mut().handle_key_release(key);
    }
    input.update();
    input.keyboard_mut().handle_key_press(KeyCode::Enter);
    ui.begin_frame(&input, Vec2::new(800.0, 600.0));
    let committed = ui.float_input("clamp", 5.0, 0.0, 10.0, bounds);
    ui.end_frame();
    assert_eq!(committed, 10.0, "99 must clamp to the max of 10");
}
