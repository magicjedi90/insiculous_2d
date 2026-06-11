//! Viewport picking (click + rectangle selection) and gizmo dragging.

use glam::Vec2;

use ecs::{GlobalTransform2D, Pair, World};
use editor::{PanelId, PickableEntity};
use engine_core::contexts::GameContext;
use engine_core::Game;

use crate::constants::MIN_ENTITY_SCALE;

use super::EditorGame;

impl<G: Game> EditorGame<G> {
    /// Handle viewport input: pan/zoom plus click and rectangle selection.
    pub(super) fn handle_viewport_picking(&mut self, ctx: &mut GameContext) {
        if self.editor.is_playing() {
            return;
        }

        let input_result = self.editor.viewport_input.handle_input_simple(
            &mut self.editor.viewport,
            &self.editor.input_mapping,
            ctx.input,
        );

        if self.editor.gizmo_has_priority() {
            return;
        }

        if input_result.clicked {
            self.editor.close_add_component_popup();
            let pickables = build_pickable_entities(ctx.world);
            let pick_result = self.editor.picker.pick_at_screen_pos(
                &self.editor.viewport,
                input_result.click_position,
                &pickables,
            );

            if let Some(entity_id) = pick_result.topmost() {
                if input_result.shift_held {
                    self.editor.selection.add(entity_id);
                } else if input_result.ctrl_held {
                    self.editor.selection.toggle(entity_id);
                } else {
                    self.editor.selection.select(entity_id);
                }
            } else if !input_result.shift_held && !input_result.ctrl_held {
                self.editor.selection.clear();
            }
        }

        // Rectangle selection (drag just completed)
        if !input_result.selection_drag_active
            && input_result.selection_start != Vec2::ZERO
            && !input_result.clicked
        {
            let pickables = build_pickable_entities(ctx.world);
            let pick_result = self.editor.picker.pick_in_screen_rect(
                &self.editor.viewport,
                input_result.selection_start,
                input_result.selection_end,
                &pickables,
            );

            if !input_result.shift_held {
                self.editor.selection.clear();
            }
            for &entity_id in &pick_result.hits {
                self.editor.selection.add(entity_id);
            }
        }
    }

    /// Render the gizmo for the primary selection and apply drag deltas,
    /// recording a single undo entry per drag.
    pub(super) fn handle_gizmo(&mut self, ctx: &mut GameContext, content_areas: &[(PanelId, common::Rect)]) {
        if self.editor.is_playing() {
            return;
        }
        let Some(entity_id) = self.editor.selection.primary() else {
            return;
        };
        if !content_areas.iter().any(|(id, _)| *id == PanelId::SCENE_VIEW) {
            return;
        }

        let entity_pos = ctx.world
            .get::<ecs::GlobalTransform2D>(entity_id)
            .map(|t| t.position);
        let Some(entity_pos) = entity_pos else {
            return;
        };

        let screen_pos = self.editor.world_to_screen(entity_pos);
        let interaction = self.editor.gizmo.render(ctx.ui, screen_pos);

        // Capture initial transform when gizmo drag starts
        if interaction.handle.is_some() && self.gizmo_drag_start.is_none() {
            if let Some(t) = ctx.world.get::<ecs::sprite_components::Transform2D>(entity_id) {
                self.gizmo_drag_start = Some(*t);
            }
        }

        if interaction.handle.is_some() {
            // Translation
            if interaction.delta != Vec2::ZERO {
                let world_delta = self.editor.gizmo_delta_to_world(interaction.delta);
                let snap_enabled = self.editor.is_snap_to_grid();

                if let Some(transform) = ctx.world.get_mut::<ecs::sprite_components::Transform2D>(entity_id) {
                    transform.position += world_delta;
                    if snap_enabled {
                        transform.position = self.editor.snap_position(transform.position);
                    }
                }
            }

            // Rotation
            if interaction.rotation_delta != 0.0 {
                if let Some(transform) = ctx.world.get_mut::<ecs::sprite_components::Transform2D>(entity_id) {
                    transform.rotation += interaction.rotation_delta;
                }
            }

            // Scale
            if interaction.scale_delta != Vec2::ZERO {
                if let Some(transform) = ctx.world.get_mut::<ecs::sprite_components::Transform2D>(entity_id) {
                    transform.scale += interaction.scale_delta;
                    transform.scale = transform.scale.max(Vec2::splat(MIN_ENTITY_SCALE));
                }
            }
        }

        // Gizmo released — create undo command for the drag
        if interaction.handle.is_none() && self.gizmo_drag_start.is_some() {
            if let Some(initial) = self.gizmo_drag_start.take() {
                if let Some(final_val) = ctx.world.get::<ecs::sprite_components::Transform2D>(entity_id) {
                    let cmd = editor::commands::TransformGizmoCommand::new(entity_id, initial, *final_val);
                    self.command_history.push_already_executed(Box::new(cmd));
                    self.editor.mark_dirty();
                }
            }
        }
    }
}

/// Build the list of pickable entities from the world.
///
/// Queries for entities that have both `GlobalTransform2D` and `Sprite` components,
/// which are required for viewport picking (position + visual size).
pub(super) fn build_pickable_entities(world: &World) -> Vec<PickableEntity> {
    let entities = world.query_entities::<Pair<GlobalTransform2D, ecs::sprite_components::Sprite>>();
    entities
        .into_iter()
        .filter_map(|entity_id| {
            let global_t = world.get::<GlobalTransform2D>(entity_id)?;
            let sprite = world.get::<ecs::sprite_components::Sprite>(entity_id)?;
            // Visual size = sprite scale * global transform scale
            let size = sprite.scale * global_t.scale;
            Some(PickableEntity::new(
                entity_id,
                global_t.position,
                size,
                sprite.depth,
            ))
        })
        .collect()
}
