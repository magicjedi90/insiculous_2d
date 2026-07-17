//! Draws data-driven UI components (`UiPanel`/`UiButton`/`UiLabel` from the
//! ecs crate) as immediate-mode UI each frame, and surfaces button presses.
//!
//! The engine calls [`draw_ui_elements`] after the game's `update()` (same
//! slot as achievement toasts). Button presses are buffered by the runner
//! and emitted onto the world event bus right after the NEXT frame's
//! `flush_events()` — the bus flushes before `update()` runs, so presses
//! reach games one frame late; read them with
//! `world.read_events::<UiButtonPressed>()`.

use ecs::{resolve_anchored_pos, EntityId, Single, UiButton, UiLabel, UiPanel, World};
use glam::{Vec2, Vec4};
use ui::{Color, Rect, TextAlign, UIContext, WidgetId};

use crate::localization::Strings;

/// World resource marker: while present, [`draw_ui_elements`] draws nothing.
/// The editor inserts it while Editing/Paused so scene-authored UI doesn't
/// cover the viewport; standalone games never insert it.
pub struct UiElementsHidden;

/// Event: a scene-defined [`UiButton`] was clicked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiButtonPressed {
    /// The button's `id` field (games match on this).
    pub id: String,
    /// The entity carrying the `UiButton` component.
    pub entity: EntityId,
}

fn to_color(v: Vec4) -> Color {
    Color::new(v.x, v.y, v.z, v.w)
}

/// Draw every visible UI element in `world` and return this frame's button
/// presses. Draw order: panels, then buttons, then labels — so labels can
/// sit on panels without depth fiddling.
pub fn draw_ui_elements(
    world: &World,
    ui: &mut UIContext,
    window_size: Vec2,
    strings: &Strings,
) -> Vec<UiButtonPressed> {
    if world.has_resource::<UiElementsHidden>() {
        return Vec::new();
    }

    for entity in world.query_entities::<Single<UiPanel>>() {
        let Some(panel) = world.get::<UiPanel>(entity) else { continue };
        if !panel.visible {
            continue;
        }
        let pos = resolve_anchored_pos(panel.anchor, panel.offset, panel.size, window_size);
        let bounds = Rect::new(pos.x, pos.y, panel.size.x, panel.size.y);
        ui.rect(bounds, to_color(panel.background));
        if panel.border_width > 0.0 {
            ui.rect_border(bounds, to_color(panel.border), panel.border_width, 0.0);
        }
    }

    let mut pressed = Vec::new();
    for entity in world.query_entities::<Single<UiButton>>() {
        let Some(button) = world.get::<UiButton>(entity) else { continue };
        if !button.visible {
            continue;
        }
        let pos = resolve_anchored_pos(button.anchor, button.offset, button.size, window_size);
        let bounds = Rect::new(pos.x, pos.y, button.size.x, button.size.y);
        let label = strings.resolve(&button.text);
        let widget_id = WidgetId::from_str_index(&button.id, entity.value() as usize);
        if ui.button(widget_id, label, bounds) {
            pressed.push(UiButtonPressed { id: button.id.clone(), entity });
        }
    }

    for entity in world.query_entities::<Single<UiLabel>>() {
        let Some(label) = world.get::<UiLabel>(entity) else { continue };
        if !label.visible {
            continue;
        }
        let text = strings.resolve(&label.text);
        let size = ui.measure_text_styled(text, label.font_size);
        let pos = resolve_anchored_pos(label.anchor, label.offset, size, window_size);
        // label_in_bounds_styled centers via font metrics — text never
        // straddles the computed box (label_styled's y would be a baseline).
        ui.label_in_bounds_styled(
            text,
            Rect::new(pos.x, pos.y, size.x, size.y),
            TextAlign::Left,
            to_color(label.color),
            label.font_size,
            0.0,
        );
    }

    pressed
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::UiAnchor;
    use input::InputHandler;

    fn ui_frame() -> UIContext {
        let mut ui = UIContext::new();
        ui.begin_frame(&InputHandler::new(), Vec2::new(800.0, 600.0));
        ui
    }

    fn world_with_elements() -> World {
        let mut world = World::new();
        let panel_entity = world.create_entity();
        world
            .add_component(&panel_entity, UiPanel { visible: true, ..Default::default() })
            .ok();
        let button_entity = world.create_entity();
        world
            .add_component(
                &button_entity,
                UiButton { id: "play".into(), text: "Play".into(), ..Default::default() },
            )
            .ok();
        let label_entity = world.create_entity();
        world
            .add_component(
                &label_entity,
                UiLabel { text: "Score".into(), ..Default::default() },
            )
            .ok();
        world
    }

    #[test]
    fn draws_panels_buttons_and_labels() {
        let world = world_with_elements();
        let mut ui = ui_frame();
        let before = ui.draw_list().len();
        let pressed = draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        assert!(pressed.is_empty());
        assert!(ui.draw_list().len() > before, "elements must emit draw commands");
        ui.end_frame();
    }

    #[test]
    fn hidden_resource_suppresses_everything() {
        let mut world = world_with_elements();
        world.insert_resource(UiElementsHidden);
        let mut ui = ui_frame();
        let before = ui.draw_list().len();
        let pressed = draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        assert!(pressed.is_empty());
        assert_eq!(ui.draw_list().len(), before, "hidden marker must suppress all drawing");
        ui.end_frame();
    }

    #[test]
    fn invisible_elements_are_skipped() {
        let mut world = World::new();
        let e = world.create_entity();
        world
            .add_component(&e, UiLabel { text: "hidden".into(), visible: false, ..Default::default() })
            .ok();
        let mut ui = ui_frame();
        let before = ui.draw_list().len();
        draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        assert_eq!(ui.draw_list().len(), before);
        ui.end_frame();
    }

    #[test]
    fn panels_draw_before_buttons_and_labels() {
        // A panel at Center and a label at Center: the panel's rect command
        // must appear before the label's text command in the draw list.
        let mut world = World::new();
        let e = world.create_entity();
        world
            .add_component(
                &e,
                UiLabel { text: "On top".into(), anchor: UiAnchor::Center, ..Default::default() },
            )
            .ok();
        let p = world.create_entity();
        world
            .add_component(&p, UiPanel { anchor: UiAnchor::Center, ..Default::default() })
            .ok();

        let mut ui = ui_frame();
        draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        let commands = ui.draw_list().commands();
        let rect_idx = commands
            .iter()
            .position(|c| matches!(c, ui::DrawCommand::Rect { .. }))
            .expect("panel rect drawn");
        let text_idx = commands
            .iter()
            .position(|c| {
                matches!(c, ui::DrawCommand::Text { .. } | ui::DrawCommand::TextPlaceholder { .. })
            })
            .expect("label text drawn");
        assert!(rect_idx < text_idx, "panel must draw before label");
        ui.end_frame();
    }

    #[test]
    fn button_click_returns_press_event() {
        use input::prelude::MouseButton;

        let mut world = World::new();
        let e = world.create_entity();
        // Anchored top-left, 120x32 at origin — click its center.
        world
            .add_component(
                &e,
                UiButton { id: "fire".into(), text: "Fire".into(), ..Default::default() },
            )
            .ok();

        let mut input = InputHandler::new();
        let mut ui = UIContext::new();

        // Press frame
        input.mouse_mut().update_position(60.0, 16.0);
        input.mouse_mut().handle_button_press(MouseButton::Left);
        ui.begin_frame(&input, Vec2::new(800.0, 600.0));
        let pressed =
            draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        ui.end_frame();
        assert!(pressed.is_empty(), "no click until release");

        // Release frame → click fires
        input.update();
        input.mouse_mut().handle_button_release(MouseButton::Left);
        ui.begin_frame(&input, Vec2::new(800.0, 600.0));
        let pressed =
            draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &Strings::empty());
        ui.end_frame();
        assert_eq!(pressed.len(), 1);
        assert_eq!(pressed[0].id, "fire");
        assert_eq!(pressed[0].entity, e);
    }

    #[test]
    fn button_and_label_text_resolve_localization_keys() {
        let mut strings = Strings::empty();
        strings.insert_locale_source(
            "en",
            r#"LocaleFile(version: 1, display_name: "English", strings: {"menu.play": "Play"})"#,
        );

        let mut world = World::new();
        let e = world.create_entity();
        world
            .add_component(&e, UiLabel { text: "@menu.play".into(), ..Default::default() })
            .ok();

        let mut ui = ui_frame();
        draw_ui_elements(&world, &mut ui, Vec2::new(800.0, 600.0), &strings);
        let has_resolved = ui.draw_list().commands().iter().any(|c| match c {
            ui::DrawCommand::TextPlaceholder { text, .. } => text == "Play",
            ui::DrawCommand::Text { data, .. } => data.text == "Play",
            _ => false,
        });
        assert!(has_resolved, "@menu.play must render as its translation");
        ui.end_frame();
    }
}
