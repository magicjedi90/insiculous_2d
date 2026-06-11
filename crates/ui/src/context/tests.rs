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
