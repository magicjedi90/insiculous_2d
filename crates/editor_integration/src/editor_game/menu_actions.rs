//! Menu bar rendering and action dispatch.

use std::path::PathBuf;

use glam::Vec2;

use engine_core::contexts::GameContext;
use engine_core::Game;

use crate::constants::DEFAULT_SCENE_PATH;
use crate::entity_ops;

use super::EditorGame;

impl<G: Game> EditorGame<G> {
    /// Render the menu bar and dispatch any selected action.
    pub(super) fn handle_menu_bar(&mut self, ctx: &mut GameContext, window_size: Vec2) {
        let Some(action) = self.editor.menu_bar.render(ctx.ui, window_size.x, &self.editor.theme) else {
            return;
        };
        log::info!("Menu action: {}", action);

        match action.as_str() {
            "Create Empty" | "Create Sprite" | "Create Camera"
            | "Create Static Body" | "Create Dynamic Body" | "Create Kinematic Body"
                if !self.editor.is_playing() =>
            {
                if let Some(entity) = entity_ops::handle_create_action(
                    &action,
                    ctx.world,
                    &mut self.editor.selection,
                    Vec2::ZERO,
                    &mut self.entity_counter,
                ) {
                    let cmd = editor::commands::CreateEntityCommand::already_created(ctx.world, entity);
                    self.command_history.push_already_executed(Box::new(cmd));
                    self.editor.mark_dirty();
                }
            }
            "Delete" if !self.editor.is_playing() => {
                self.delete_selected_entities(ctx);
            }
            "Duplicate" if !self.editor.is_playing() => {
                self.duplicate_selected_entities(ctx);
            }
            "Undo" if !self.editor.is_playing() => {
                if let Some(name) = self.command_history.undo_name() {
                    self.editor.status_bar.show_message(format!("Undo: {}", name));
                }
                self.command_history.undo(ctx.world);
                self.editor.mark_dirty();
            }
            "Redo" if !self.editor.is_playing() => {
                if let Some(name) = self.command_history.redo_name() {
                    self.editor.status_bar.show_message(format!("Redo: {}", name));
                }
                self.command_history.redo(ctx.world);
                self.editor.mark_dirty();
            }
            "New Scene" if !self.editor.is_playing() => {
                self.new_scene(ctx.world);
            }
            "Open Scene..." if !self.editor.is_playing() => {
                let path = PathBuf::from(DEFAULT_SCENE_PATH);
                self.load_scene_with_feedback(ctx.world, ctx.assets, &path);
            }
            "Save" => {
                if let Err(e) = self.save_scene(ctx.world, ctx.assets) {
                    self.editor.status_bar.show_error(format!("Save failed: {}", e));
                    log::error!("Failed to save: {}", e);
                }
            }
            "Save As..." => {
                let path = PathBuf::from(DEFAULT_SCENE_PATH);
                if let Err(e) = self.save_scene_as(ctx.world, ctx.assets, path) {
                    self.editor.status_bar.show_error(format!("Save failed: {}", e));
                    log::error!("Failed to save: {}", e);
                }
            }
            "Exit" => std::process::exit(0),
            "Scene View" | "Inspector" | "Hierarchy" | "Asset Browser" | "Console" => {
                log::info!("Toggle panel: {}", action);
            }
            _ => log::info!("Unhandled action: {}", action),
        }
    }

    /// Delete all selected entities as a single undoable action.
    pub(super) fn delete_selected_entities(&mut self, ctx: &mut GameContext) {
        let selected: Vec<ecs::EntityId> = self.editor.selection.selected().collect();
        if selected.is_empty() {
            return;
        }
        if selected.len() == 1 {
            let cmd = editor::commands::DeleteEntityCommand::new(selected[0]);
            self.command_history.execute(Box::new(cmd), ctx.world);
        } else {
            let cmds: Vec<Box<dyn editor::EditorCommand>> = selected.iter()
                .map(|&e| Box::new(editor::commands::DeleteEntityCommand::new(e)) as Box<dyn editor::EditorCommand>)
                .collect();
            let cmd = editor::commands::MacroCommand::new("Delete Entities", cmds);
            self.command_history.execute(Box::new(cmd), ctx.world);
        }
        self.editor.selection.clear();
        self.editor.mark_dirty();
    }

    /// Duplicate the primary selected entity (and its subtree), recording undo.
    pub(super) fn duplicate_selected_entities(&mut self, ctx: &mut GameContext) {
        let Some(primary) = self.editor.selection.primary() else {
            return;
        };
        entity_ops::duplicate_selected_entities(
            ctx.world,
            &mut self.editor.selection,
            &mut self.entity_counter,
        );
        if let Some(new_entity) = self.editor.selection.primary() {
            if new_entity != primary {
                let cmd = editor::commands::CreateEntityCommand::already_created(ctx.world, new_entity);
                self.command_history.push_already_executed(Box::new(cmd));
                self.editor.mark_dirty();
            }
        }
    }
}
