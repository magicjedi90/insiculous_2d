//! Panel content rendering for editor dock panels.
//!
//! Extracted from editor_demo.rs — renders the content inside each dock panel
//! (scene view, hierarchy tree, inspector, asset browser).

use glam::Vec2;

use editor::{
    EditorContext, HierarchyPanel, InspectorStyle, PanelId,
    inspect_component,
};
use engine_core::contexts::GameContext;

/// Render the content of a specific dock panel.
pub fn render_panel_content(
    editor: &mut EditorContext,
    ctx: &mut GameContext,
    panel_id: PanelId,
    bounds: common::Rect,
) {
    let padding = 8.0;
    let content_x = bounds.x + padding;
    let y = bounds.y + padding;

    match panel_id {
        PanelId::SCENE_VIEW => render_scene_view(editor, ctx, bounds),
        PanelId::HIERARCHY => render_hierarchy(editor, ctx, bounds),
        PanelId::INSPECTOR => render_inspector(editor, ctx, content_x, y),
        PanelId::ASSET_BROWSER => render_asset_browser(ctx, content_x, y),
        _ => render_default(ctx, content_x, y),
    }
}

/// Scene view — grid info and viewport origin crosshair.
fn render_scene_view(editor: &EditorContext, ctx: &mut GameContext, bounds: common::Rect) {
    let padding = 8.0;
    let content_x = bounds.x + padding;
    let y = bounds.y + padding;

    if editor.is_grid_visible() {
        ctx.ui.label(
            &format!("Grid: {}px", editor.grid_size()),
            Vec2::new(content_x, y),
        );
    }

    // Draw viewport origin crosshair
    let center = bounds.center();
    ctx.ui.circle(center, 5.0, ui::Color::new(0.3, 0.3, 0.3, 1.0));
    ctx.ui.line(
        Vec2::new(center.x - 20.0, center.y),
        Vec2::new(center.x + 20.0, center.y),
        ui::Color::new(0.4, 0.4, 0.4, 1.0),
        1.0,
    );
    ctx.ui.line(
        Vec2::new(center.x, center.y - 20.0),
        Vec2::new(center.x, center.y + 20.0),
        ui::Color::new(0.4, 0.4, 0.4, 1.0),
        1.0,
    );
}

/// Hierarchy — tree view with click-to-select and Ctrl toggle.
fn render_hierarchy(editor: &mut EditorContext, ctx: &mut GameContext, bounds: common::Rect) {
    let clicked = editor.hierarchy.render(
        ctx.ui,
        ctx.world,
        &mut editor.selection,
        bounds,
    );

    for entity_id in clicked {
        if ctx.input.keyboard().is_key_pressed(winit::keyboard::KeyCode::ControlLeft) {
            editor.selection.toggle(entity_id);
        } else {
            editor.selection.select(entity_id);
        }
        log::info!(
            "Selected entity: {} ({})",
            HierarchyPanel::entity_display_name(ctx.world, entity_id),
            entity_id.value()
        );
    }
}

/// Inspector — component inspection for the selected entity.
fn render_inspector(editor: &EditorContext, ctx: &mut GameContext, content_x: f32, mut y: f32) {
    let line_height = 20.0;

    if let Some(entity_id) = editor.selection.primary() {
        ctx.ui.label(
            &format!("Entity: {}", entity_id.value()),
            Vec2::new(content_x, y),
        );
        y += line_height;

        let style = InspectorStyle::default();

        if let Some(transform) = ctx.world.get::<ecs::sprite_components::Transform2D>(entity_id) {
            y += line_height * 0.5;
            y = inspect_component(ctx.ui, "Transform2D", transform, content_x, y, &style);
        }

        if let Some(sprite) = ctx.world.get::<ecs::sprite_components::Sprite>(entity_id) {
            y += line_height * 0.5;
            y = inspect_component(ctx.ui, "Sprite", sprite, content_x, y, &style);
        }

        if let Some(camera) = ctx.world.get::<ecs::sprite_components::Camera>(entity_id) {
            y += line_height * 0.5;
            y = inspect_component(ctx.ui, "Camera", camera, content_x, y, &style);
        }

        if let Some(animation) = ctx.world.get::<ecs::sprite_components::SpriteAnimation>(entity_id) {
            y += line_height * 0.5;
            let _ = inspect_component(ctx.ui, "SpriteAnimation", animation, content_x, y, &style);
        }
    } else {
        ctx.ui.label("No selection", Vec2::new(content_x, y));
    }
}

/// Asset browser — placeholder.
fn render_asset_browser(ctx: &mut GameContext, content_x: f32, y: f32) {
    ctx.ui.label("(Asset browser not yet implemented)", Vec2::new(content_x, y));
}

/// Fallback for unknown panels.
fn render_default(ctx: &mut GameContext, content_x: f32, y: f32) {
    ctx.ui.label("Panel", Vec2::new(content_x, y));
}
