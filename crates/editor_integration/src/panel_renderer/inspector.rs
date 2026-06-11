//! Inspector panel: editable component fields with undo-recorded writeback,
//! read-only view during play, remove buttons, and the add-component popup.

use glam::Vec2;

use editor::{
    available_components, categorized_components, inspect_all_components,
    CommandHistory, ComponentKind, EditorContext, InspectorStyle,
    EditableInspector, FieldId,
    edit_transform2d, edit_sprite, edit_rigid_body, edit_collider, edit_audio_source,
    inspect_component,
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
    let mut component_index: usize = 0;
    let mut removals: Vec<ComponentKind> = Vec::new();
    let inspect_style = editor.theme.inspector_style();
    let field_style = editor.theme.editable_field_style();

    // --- Transform2D (not removable) ---
    if let Some(transform) = ctx.world.get::<common::Transform2D>(entity_id).copied() {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        let edit = edit_transform2d(&mut inspector, &transform);
        y = inspector.y();

        apply_component_edit(ctx.world, entity_id, &transform, edit, command_history,
            |e, old, new, hint| Box::new(editor::commands::SetTransformCommand::new(e, old, new, hint)));
        component_index += 1;
    }

    // --- Camera (removable, read-only display for now) ---
    if let Some(camera) = ctx.world.get::<common::Camera>(entity_id) {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        if inspector.header_with_remove("Camera", true) {
            removals.push(ComponentKind::Camera);
        }
        y = inspector.y();
        y = inspect_component(ctx.ui, "", camera, content_x + 16.0, y, &inspect_style);
        component_index += 1;
    }

    // --- Sprite (removable, uses edit_sprite which renders its own header) ---
    if let Some(sprite) = ctx.world.get::<ecs::sprite_components::Sprite>(entity_id).cloned() {
        y += line_height * 0.5;
        let header_y = y;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        let edit = edit_sprite(&mut inspector, &sprite);
        y = inspector.y();

        if render_remove_button(ctx.ui, component_index, content_x, header_y, &field_style) {
            removals.push(ComponentKind::Sprite);
        }

        apply_component_edit(ctx.world, entity_id, &sprite, edit, command_history,
            |e, old, new, hint| Box::new(editor::commands::SetSpriteCommand::new(e, old, new, hint)));
        component_index += 1;
    }

    // --- SpriteAnimation (removable, read-only display for now) ---
    if let Some(anim) = ctx.world.get::<ecs::sprite_components::SpriteAnimation>(entity_id) {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        if inspector.header_with_remove("SpriteAnimation", true) {
            removals.push(ComponentKind::SpriteAnimation);
        }
        y = inspector.y();
        y = inspect_component(ctx.ui, "", anim, content_x + 16.0, y, &inspect_style);
        component_index += 1;
    }

    // --- RigidBody (removable, uses edit_rigid_body which renders its own header) ---
    if let Some(body) = ctx.world.get::<physics::components::RigidBody>(entity_id).cloned() {
        y += line_height * 0.5;
        let header_y = y;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        let edit = edit_rigid_body(&mut inspector, &body);
        y = inspector.y();

        if render_remove_button(ctx.ui, component_index, content_x, header_y, &field_style) {
            removals.push(ComponentKind::RigidBody);
        }

        apply_component_edit(ctx.world, entity_id, &body, edit, command_history,
            |e, old, new, hint| Box::new(editor::commands::SetRigidBodyCommand::new(e, old, new, hint)));
        component_index += 1;
    }

    // --- Collider (removable, uses edit_collider which renders its own header) ---
    if let Some(collider) = ctx.world.get::<physics::components::Collider>(entity_id).cloned() {
        y += line_height * 0.5;
        let header_y = y;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        let edit = edit_collider(&mut inspector, &collider);
        y = inspector.y();

        if render_remove_button(ctx.ui, component_index, content_x, header_y, &field_style) {
            removals.push(ComponentKind::Collider);
        }

        apply_component_edit(ctx.world, entity_id, &collider, edit, command_history,
            |e, old, new, hint| Box::new(editor::commands::SetColliderCommand::new(e, old, new, hint)));
        component_index += 1;
    }

    // --- AudioSource (removable, uses edit_audio_source which renders its own header) ---
    if let Some(source) = ctx.world.get::<ecs::audio_components::AudioSource>(entity_id).cloned() {
        y += line_height * 0.5;
        let header_y = y;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        let edit = edit_audio_source(&mut inspector, &source);
        y = inspector.y();

        if render_remove_button(ctx.ui, component_index, content_x, header_y, &field_style) {
            removals.push(ComponentKind::AudioSource);
        }

        apply_component_edit(ctx.world, entity_id, &source, edit, command_history,
            |e, old, new, hint| Box::new(editor::commands::SetAudioSourceCommand::new(e, old, new, hint)));
        component_index += 1;
    }

    // --- AudioListener (removable, read-only display for now) ---
    if let Some(listener) = ctx.world.get::<ecs::audio_components::AudioListener>(entity_id) {
        y += line_height * 0.5;
        let mut inspector = EditableInspector::new(ctx.ui, content_x, y)
            .with_component_index(component_index)
            .with_style(field_style.clone());
        if inspector.header_with_remove("AudioListener", true) {
            removals.push(ComponentKind::AudioListener);
        }
        y = inspector.y();
        y = inspect_component(ctx.ui, "", listener, content_x + 16.0, y, &inspect_style);
        component_index += 1;
    }

    // Apply removals via commands
    for kind in &removals {
        let cmd = editor::commands::RemoveComponentCommand::new(entity_id, *kind);
        command_history.execute(Box::new(cmd), ctx.world);
        log::info!("Removed component: {}", kind.display_name());
    }

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

/// Apply an inspector edit: write the new value to the world (for immediate
/// visual feedback) and record it on the undo stack with merge support, so
/// continuous slider drags collapse into a single undo entry.
pub(super) fn apply_component_edit<T: ecs::Component + Clone>(
    world: &mut ecs::World,
    entity: ecs::EntityId,
    old: &T,
    edit: Option<editor::ComponentEdit<T>>,
    history: &mut CommandHistory,
    make_cmd: impl FnOnce(ecs::EntityId, T, T, &'static str) -> Box<dyn editor::EditorCommand>,
) {
    if let Some(editor::ComponentEdit { new_value, field_hint }) = edit {
        if let Some(c) = world.get_mut::<T>(entity) {
            *c = new_value.clone();
        }
        history.try_merge_or_push(make_cmd(entity, old.clone(), new_value, field_hint));
    }
}

/// Render a small [X] remove button at the header position of a component.
///
/// Used for components that have their own `edit_*()` function which renders
/// the header internally. The button is overlaid at the same Y position.
fn render_remove_button(
    ui: &mut ui::UIContext,
    component_index: usize,
    content_x: f32,
    header_y: f32,
    style: &editor::EditableFieldStyle,
) -> bool {
    let btn_size = 18.0;
    let btn_x = content_x + style.label_width + 90.0;
    let btn_bounds = ui::Rect::new(btn_x, header_y, btn_size, btn_size);
    let btn_id = FieldId::new(component_index, 99, 0);
    ui.button(btn_id, "X", btn_bounds)
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
