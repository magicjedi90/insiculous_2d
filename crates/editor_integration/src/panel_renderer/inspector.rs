//! Inspector panel: editable component fields with undo-recorded writeback,
//! read-only view during play, remove buttons, and the add-component popup.

use glam::Vec2;

use editor::{
    available_components, categorized_components, edit_all_components,
    inspect_all_components, CommandHistory, ComponentKind, EditorContext,
    FieldId, InspectorStyle,
};
use engine_core::contexts::GameContext;

/// Inspector — component inspection for the selected entity.
///
/// During Editing/Paused: renders editable fields with live writeback.
/// During Playing: renders read-only view via `inspect_component()`.
pub(super) fn render_inspector(
    editor: &mut EditorContext,
    ctx: &mut GameContext,
    content_x: f32,
    mut y: f32,
    command_history: &mut CommandHistory,
) {
    let line_height = 20.0;

    let entity_id = match editor.selection.primary() {
        Some(id) => id,
        None => {
            ctx.ui.label("No selection", Vec2::new(content_x, y));
            return;
        }
    };

    ctx.ui.label(
        &format!("Entity: {}", entity_id.value()),
        Vec2::new(content_x, y),
    );
    y += line_height;

    if editor.is_playing() {
        render_inspector_readonly(ctx, entity_id, content_x, y, &editor.theme.inspector_style());
    } else {
        render_inspector_editable(editor, ctx, entity_id, content_x, y, command_history);
    }
}

/// Read-only inspector using the editor's component registry (used during Playing).
fn render_inspector_readonly(
    ctx: &mut GameContext,
    entity_id: ecs::EntityId,
    content_x: f32,
    y: f32,
    style: &InspectorStyle,
) {
    let line_height = 20.0;
    inspect_all_components(
        ctx.ui, ctx.world, entity_id, content_x, y, style, line_height * 0.5,
    );
}

/// Editable inspector with live writeback (used during Editing/Paused).
fn render_inspector_editable(
    editor: &mut EditorContext,
    ctx: &mut GameContext,
    entity_id: ecs::EntityId,
    content_x: f32,
    mut y: f32,
    command_history: &mut CommandHistory,
) {
    let line_height = 20.0;
    let inspect_style = editor.theme.inspector_style();
    let field_style = editor.theme.editable_field_style();

    // Every per-component block (field editors, undo-recorded writeback,
    // remove buttons, read-only fallbacks) is generated from the editor's
    // component registry — adding a component to the registry is all it
    // takes to appear here.
    let (next_y, component_index) = edit_all_components(
        ctx.ui,
        ctx.world,
        entity_id,
        command_history,
        content_x,
        y,
        &inspect_style,
        &field_style,
        line_height * 0.5,
    );
    y = next_y;

    // --- [+ Add Component] button ---
    y += line_height;
    let btn_bounds = ui::Rect::new(content_x, y, 160.0, 24.0);
    let add_btn_id = FieldId::new(component_index + 50, 0, 0);
    if ctx.ui.button(add_btn_id, "+ Add Component", btn_bounds) {
        editor.toggle_add_component_popup();
    }
    y += 28.0;

    // --- Add Component Popup ---
    if editor.is_add_component_popup_open() {
        let available = available_components(ctx.world, entity_id);
        if available.is_empty() {
            ctx.ui.label("(all components added)", Vec2::new(content_x + 8.0, y));
        } else {
            let popup_height = categorized_popup_height(&available);
            let popup_bounds = ui::Rect::new(content_x, y, 180.0, popup_height);
            ctx.ui.panel(popup_bounds);

            let mut popup_y = y + 4.0;
            let mut popup_btn_idx: usize = 0;

            for (category, kinds) in categorized_components() {
                let visible: Vec<ComponentKind> = kinds.iter()
                    .copied()
                    .filter(|k| available.contains(k))
                    .collect();
                if visible.is_empty() {
                    continue;
                }

                ctx.ui.label_styled(
                    category.label(),
                    Vec2::new(content_x + 8.0, popup_y),
                    editor.theme.text_muted,
                    12.0,
                );
                popup_y += 18.0;

                for kind in visible {
                    let btn_bounds = ui::Rect::new(content_x + 16.0, popup_y, 148.0, 22.0);
                    let btn_id = FieldId::new(component_index + 60 + popup_btn_idx, 0, 0);
                    if ctx.ui.button(btn_id, kind.display_name(), btn_bounds) {
                        let cmd = editor::commands::AddComponentCommand::new(entity_id, kind);
                        command_history.execute(Box::new(cmd), ctx.world);
                        editor.close_add_component_popup();
                        log::info!("Added component: {}", kind.display_name());
                    }
                    popup_y += 24.0;
                    popup_btn_idx += 1;
                }
            }
        }
    }

    let _ = component_index;
}

/// Calculate the height needed for the categorized popup.
fn categorized_popup_height(available: &[ComponentKind]) -> f32 {
    let mut height = 8.0; // padding
    for (_, kinds) in categorized_components() {
        let visible_count = kinds.iter().filter(|k| available.contains(k)).count();
        if visible_count > 0 {
            height += 18.0; // category label
            height += visible_count as f32 * 24.0; // buttons
        }
    }
    height
}
