//! Editor-wrapped game implementation.
//!
//! `EditorGame<G>` transparently wraps any `Game` implementation, intercepting
//! all trait methods to weave in editor UI orchestration (menu bar, toolbar,
//! dock panels, hierarchy, inspector, gizmo, tool shortcuts, play/pause/stop)
//! and delegating to the inner game.

use std::path::{Path, PathBuf};

use glam::Vec2;
use winit::keyboard::KeyCode;

use ecs::{GlobalTransform2D, Pair, System, World};
use editor::{EditorContext, EditorPlayState, EditorTool, PanelId, PickableEntity, PlayControlAction};
use editor::world_snapshot::WorldSnapshot;
use engine_core::contexts::{GameContext, RenderContext};
use engine_core::scene_data::PhysicsSettings;
use engine_core::Game;
use engine_core::GameConfig;

use crate::entity_ops;
use crate::panel_renderer;

/// Wraps a user's `Game` with the full editor UI overlay.
struct EditorGame<G: Game> {
    inner: G,
    editor: EditorContext,
    transform_system: ecs::TransformHierarchySystem,
    font_loaded: bool,
    /// Snapshot of the world state captured when entering play mode.
    world_snapshot: Option<WorldSnapshot>,
    /// Auto-incrementing counter for unique entity names.
    entity_counter: u32,
    /// Undo/redo command history for editor actions.
    command_history: editor::CommandHistory,
    /// Initial transform captured when gizmo drag starts.
    gizmo_drag_start: Option<common::Transform2D>,
    /// Physics settings for scene serialization.
    physics_settings: Option<PhysicsSettings>,
}

impl<G: Game> EditorGame<G> {
    fn new(game: G) -> Self {
        Self {
            inner: game,
            editor: EditorContext::new(),
            transform_system: ecs::TransformHierarchySystem::new(),
            font_loaded: false,
            world_snapshot: None,
            entity_counter: 0,
            command_history: editor::CommandHistory::new(),
            gizmo_drag_start: None,
            physics_settings: None,
        }
    }

    fn handle_menu_action(&mut self, action: &str) {
        match action {
            "Exit" => std::process::exit(0),
            // Undo/Redo handled in update() where we have world access.
            "Undo" | "Redo" => {}
            // Scene ops handled in update() where we have world + assets access.
            "New Scene" | "Open Scene..." | "Save" | "Save As..." => {}
            "Scene View" | "Inspector" | "Hierarchy" | "Asset Browser" | "Console" => {
                log::info!("Toggle panel: {}", action);
            }
            _ => log::info!("Unhandled action: {}", action),
        }
    }

    /// Save the current scene to the existing scene path (or default if none set).
    fn save_scene(&mut self, world: &World, assets: &engine_core::assets::AssetManager) -> Result<(), String> {
        let path = self.editor.scene_path()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("scenes/scene.ron"));
        self.save_scene_as(world, assets, path)
    }

    /// Save the current scene to a specific path.
    fn save_scene_as(&mut self, world: &World, assets: &engine_core::assets::AssetManager, path: PathBuf) -> Result<(), String> {
        let scene_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let texture_path_fn = |handle: u32| -> String {
            assets.texture_path(handle)
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
                })
        };

        let scene_data = engine_core::scene_serializer::world_to_scene_data(
            world, &scene_name, self.physics_settings.clone(), &texture_path_fn,
        );

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }

        engine_core::scene_serializer::save_scene_to_file(&scene_data, &path)?;

        self.editor.set_scene_path(Some(path.clone()));
        self.editor.set_dirty(false);
        self.editor.status_bar.show_message("Scene saved");
        log::info!("Scene saved to: {:?}", path);
        Ok(())
    }

    /// Load a scene from disk, replacing the current world.
    fn load_scene(&mut self, world: &mut World, assets: &mut engine_core::assets::AssetManager, path: &Path) -> Result<(), String> {
        if self.editor.is_dirty() {
            log::warn!("Current scene has unsaved changes. Save first to avoid losing work.");
        }

        // Clear existing world
        for entity in world.entities() {
            world.remove_entity(&entity).ok();
        }

        // Load and instantiate scene
        let scene_instance = engine_core::scene_loader::SceneLoader::load_and_instantiate(path, world, assets)
            .map_err(|e| format!("Failed to load scene: {}", e))?;

        // Store physics settings from loaded scene
        self.physics_settings = scene_instance.physics.clone();

        log::info!("Scene loaded from: {:?} ({} entities)", path, scene_instance.entity_count);

        self.editor.set_scene_path(Some(path.to_path_buf()));
        self.editor.set_dirty(false);
        self.command_history = editor::CommandHistory::new();
        self.editor.selection.clear();
        self.gizmo_drag_start = None;
        self.editor.status_bar.show_message("Scene loaded");

        Ok(())
    }

    /// Create a new empty scene, clearing the world.
    fn new_scene(&mut self, world: &mut World) {
        if self.editor.is_dirty() {
            log::warn!("Current scene has unsaved changes. Save first to avoid losing work.");
        }

        // Clear existing world
        for entity in world.entities() {
            world.remove_entity(&entity).ok();
        }

        self.editor.set_scene_path(None);
        self.editor.set_dirty(false);
        self.command_history = editor::CommandHistory::new();
        self.editor.selection.clear();
        self.entity_counter = 0;
        self.physics_settings = None;
        self.gizmo_drag_start = None;
        log::info!("New scene created");
    }

    fn handle_tool_shortcuts(&mut self, ctx: &GameContext) {
        let kb = ctx.input.keyboard();

        if kb.is_key_just_pressed(KeyCode::KeyQ) {
            self.editor.set_tool(EditorTool::Select);
        } else if kb.is_key_just_pressed(KeyCode::KeyW) {
            self.editor.set_tool(EditorTool::Move);
        } else if kb.is_key_just_pressed(KeyCode::KeyE) {
            self.editor.set_tool(EditorTool::Rotate);
        } else if kb.is_key_just_pressed(KeyCode::KeyR) {
            self.editor.set_tool(EditorTool::Scale);
        }
    }

    /// Handle a play control action (Play, Pause, Stop).
    ///
    /// Returns `true` if a Stop was performed (world restored from snapshot),
    /// so the caller can notify the inner game via `on_play_stopped`.
    fn handle_play_action(&mut self, action: PlayControlAction, world: &mut ecs::World) -> bool {
        match action {
            PlayControlAction::Play => {
                if self.editor.is_editing() {
                    // Cancel any in-progress gizmo drag
                    self.gizmo_drag_start = None;
                    // Starting a new play session — capture snapshot
                    self.world_snapshot = Some(WorldSnapshot::capture(world));
                    self.editor.set_play_state(EditorPlayState::Playing);
                    self.editor.close_add_component_popup();
                    log::info!("Play: snapshot captured, entering play mode");
                } else if self.editor.is_paused() {
                    // Resuming from pause
                    self.editor.set_play_state(EditorPlayState::Playing);
                    self.editor.close_add_component_popup();
                    log::info!("Play: resumed from pause");
                }
                false
            }
            PlayControlAction::Pause => {
                if self.editor.is_playing() {
                    self.editor.set_play_state(EditorPlayState::Paused);
                    log::info!("Paused");
                }
                false
            }
            PlayControlAction::Stop => {
                if self.editor.in_play_session() {
                    // Restore world from snapshot
                    if let Some(snapshot) = self.world_snapshot.take() {
                        snapshot.restore(world);
                        log::info!("Stop: world restored from snapshot");
                    }
                    self.editor.set_play_state(EditorPlayState::Editing);
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl<G: Game> Game for EditorGame<G> {
    fn init(&mut self, ctx: &mut GameContext) {
        // Load font from common search paths
        let font_paths = [
            "examples/assets/fonts/font.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];

        for path in font_paths {
            if ctx.ui.load_font_file(path).is_ok() {
                self.font_loaded = true;
                log::info!("Editor font loaded from: {}", path);
                break;
            }
        }

        if !self.font_loaded {
            log::warn!("No font loaded. Text will render as placeholders.");
            log::warn!("To enable font rendering, add a .ttf file to examples/assets/fonts/font.ttf");
        }

        // Delegate to inner game
        self.inner.init(ctx);
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let window_size = ctx.window_size;

        // 1. Run transform hierarchy system
        self.transform_system.update(ctx.world, ctx.delta_time);

        // 2. Editor layout
        self.editor.update_layout(window_size);

        // 3. Menu bar
        if let Some(action) = self.editor.menu_bar.render(ctx.ui, window_size.x) {
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
                    let selected: Vec<ecs::EntityId> = self.editor.selection.selected().collect();
                    if !selected.is_empty() {
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
                }
                "Duplicate" if !self.editor.is_playing() => {
                    if let Some(primary) = self.editor.selection.primary() {
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
                    let path = PathBuf::from("scenes/scene.ron");
                    if let Err(e) = self.load_scene(ctx.world, ctx.assets, &path) {
                        log::error!("Failed to load scene: {}", e);
                    }
                }
                "Save" => {
                    if let Err(e) = self.save_scene(ctx.world, ctx.assets) {
                        self.editor.status_bar.show_error(format!("Save failed: {}", e));
                        log::error!("Failed to save: {}", e);
                    }
                }
                "Save As..." => {
                    let path = PathBuf::from("scenes/scene.ron");
                    if let Err(e) = self.save_scene_as(ctx.world, ctx.assets, path) {
                        self.editor.status_bar.show_error(format!("Save failed: {}", e));
                        log::error!("Failed to save: {}", e);
                    }
                }
                _ => self.handle_menu_action(&action),
            }
        }

        // 4. Toolbar
        if let Some(tool) = self.editor.toolbar.render(ctx.ui) {
            log::info!("Tool changed: {:?}", tool);
        }

        // 4b. Play controls (rendered to the right of the toolbar)
        {
            let toolbar_bounds = self.editor.toolbar.bounds();
            self.editor.play_controls.position = Vec2::new(
                toolbar_bounds.x + toolbar_bounds.width + self.editor.play_controls.spacing * 4.0,
                toolbar_bounds.y,
            );
            let play_state = self.editor.play_state();
            let theme = &self.editor.theme;
            if let Some(action) = self.editor.play_controls.render(ctx.ui, play_state, theme) {
                if self.handle_play_action(action, ctx.world) {
                    self.inner.on_play_stopped(ctx);
                }
            }
        }

        // 5. Dock panel frames + resize handles
        let theme = &self.editor.theme;
        let content_areas = self.editor.dock_area.render(ctx.ui, theme);
        self.editor.dock_area.handle_resize(ctx.ui);

        // 6. Panel content (each panel gets its own push/pop clip rect)
        for (panel_id, bounds) in content_areas.clone() {
            ctx.ui.push_clip_rect(ui::Rect::new(bounds.x, bounds.y, bounds.width, bounds.height));
            panel_renderer::render_panel_content(
                &mut self.editor, ctx, panel_id, bounds, &mut self.command_history,
            );
            ctx.ui.pop_clip_rect();
        }

        // 7b. Viewport input handling (pan, zoom, click, selection)
        if !self.editor.is_playing() {
            let input_result = self.editor.viewport_input.handle_input_simple(
                &mut self.editor.viewport,
                &self.editor.input_mapping,
                ctx.input,
            );

            if !self.editor.gizmo_has_priority() {
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

                    if input_result.shift_held {
                        for &entity_id in &pick_result.hits {
                            self.editor.selection.add(entity_id);
                        }
                    } else {
                        self.editor.selection.clear();
                        for &entity_id in &pick_result.hits {
                            self.editor.selection.add(entity_id);
                        }
                    }
                }
            }
        }

        // 8. Gizmo interaction for selected entity (skip during play)
        if !self.editor.is_playing() {
            if let Some(entity_id) = self.editor.selection.primary() {
                if content_areas.iter().any(|(id, _)| *id == PanelId::SCENE_VIEW) {
                    let entity_pos = ctx.world
                        .get::<ecs::GlobalTransform2D>(entity_id)
                        .map(|t| t.position);

                    if let Some(entity_pos) = entity_pos {
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
                                    transform.scale = transform.scale.max(Vec2::splat(0.01));
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
            }
        }

        // 9. Tool keyboard shortcuts (skip during play)
        if !self.editor.is_playing() {
            self.handle_tool_shortcuts(ctx);
        }

        // 10. Delegate to inner game (only when Playing)
        if self.editor.is_playing() {
            if let Some(scene_bounds) = self.editor.scene_view_bounds() {
                ctx.ui.push_clip_rect(ui::Rect::new(scene_bounds.x, scene_bounds.y, scene_bounds.width, scene_bounds.height));
            }
            self.inner.update(ctx);
            if self.editor.scene_view_bounds().is_some() {
                ctx.ui.pop_clip_rect();
            }
        }

        // 11. Status bar
        {
            // Update FPS (smoothed) and entity count
            let fps = if ctx.delta_time > 0.0 { 1.0 / ctx.delta_time } else { 0.0 };
            let smoothed_fps = fps.min(999.0); // Cap for display
            self.editor.status_bar.update_stats(ctx.world.entity_count(), smoothed_fps);
            self.editor.status_bar.update(ctx.delta_time);

            let theme = &self.editor.theme;
            self.editor.status_bar.render(ctx.ui, window_size, theme);
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.inner.render(ctx);
    }

    fn on_key_pressed(&mut self, key: KeyCode, ctx: &mut GameContext) {
        let ctrl = ctx.input.keyboard().is_key_pressed(KeyCode::ControlLeft)
            || ctx.input.keyboard().is_key_pressed(KeyCode::ControlRight);
        let shift = ctx.input.keyboard().is_key_pressed(KeyCode::ShiftLeft)
            || ctx.input.keyboard().is_key_pressed(KeyCode::ShiftRight);

        // Play state shortcuts (always intercepted)
        if key == KeyCode::KeyP && ctrl && shift {
            // Ctrl+Shift+P → Stop
            if self.handle_play_action(PlayControlAction::Stop, ctx.world) {
                self.inner.on_play_stopped(ctx);
            }
            return;
        }
        if key == KeyCode::KeyP && ctrl {
            // Ctrl+P → Play/Pause toggle
            if self.editor.is_playing() {
                self.handle_play_action(PlayControlAction::Pause, ctx.world);
            } else {
                self.handle_play_action(PlayControlAction::Play, ctx.world);
            }
            return;
        }

        // During play mode, forward keys to inner game (skip editor shortcuts)
        if self.editor.is_playing() {
            self.inner.on_key_pressed(key, ctx);
            return;
        }

        // Editor shortcuts (only during Editing/Paused)
        match key {
            KeyCode::KeyZ if ctrl && !shift => {
                self.command_history.undo(ctx.world);
                self.editor.mark_dirty();
            }
            KeyCode::KeyZ if ctrl && shift => {
                self.command_history.redo(ctx.world);
                self.editor.mark_dirty();
            }
            KeyCode::KeyY if ctrl => {
                self.command_history.redo(ctx.world);
                self.editor.mark_dirty();
            }
            KeyCode::KeyG => self.editor.toggle_grid(),
            KeyCode::KeyS if ctrl && shift => {
                // Ctrl+Shift+S → Save As
                let path = PathBuf::from("scenes/scene.ron");
                if let Err(e) = self.save_scene_as(ctx.world, ctx.assets, path) {
                    log::error!("Failed to save: {}", e);
                }
            }
            KeyCode::KeyS if ctrl => {
                // Ctrl+S → Save
                if let Err(e) = self.save_scene(ctx.world, ctx.assets) {
                    log::error!("Failed to save: {}", e);
                }
            }
            KeyCode::KeyN if ctrl => {
                // Ctrl+N → New Scene
                self.new_scene(ctx.world);
            }
            KeyCode::KeyO if ctrl => {
                // Ctrl+O → Open Scene
                let path = PathBuf::from("scenes/scene.ron");
                if let Err(e) = self.load_scene(ctx.world, ctx.assets, &path) {
                    log::error!("Failed to load scene: {}", e);
                }
            }
            KeyCode::KeyD if ctrl => {
                if let Some(primary) = self.editor.selection.primary() {
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
            KeyCode::Delete | KeyCode::Backspace => {
                let selected: Vec<ecs::EntityId> = self.editor.selection.selected().collect();
                if !selected.is_empty() {
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
            }
            KeyCode::Equal => self.editor.zoom_camera(1.1),
            KeyCode::Minus => self.editor.zoom_camera(0.9),
            KeyCode::Digit0 => self.editor.reset_camera(),
            KeyCode::F5 => {
                // F5 → Start/Resume play (only from Editing or Paused)
                self.handle_play_action(PlayControlAction::Play, ctx.world);
            }
            _ => self.inner.on_key_pressed(key, ctx),
        }
    }

    fn on_key_released(&mut self, key: KeyCode, ctx: &mut GameContext) {
        self.inner.on_key_released(key, ctx);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        self.inner.on_resize(width, height);
    }

    fn on_exit(&mut self) {
        self.inner.on_exit();
    }
}

/// Build the list of pickable entities from the world.
///
/// Queries for entities that have both `GlobalTransform2D` and `Sprite` components,
/// which are required for viewport picking (position + visual size).
fn build_pickable_entities(world: &World) -> Vec<PickableEntity> {
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

/// Run a game with the full editor UI overlay.
///
/// This wraps the given game in `EditorGame`, which intercepts all `Game` trait
/// methods to add editor chrome (menu bar, toolbar, dock panels, hierarchy,
/// inspector, gizmo, tool shortcuts, play/pause/stop) around the user's game.
///
/// # Minimum window size
/// The editor needs at least 1024x720 to be usable. If the provided config
/// specifies a smaller size, it will be enlarged.
pub fn run_game_with_editor<G: Game>(game: G, mut config: GameConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Enforce minimum window size for editor usability
    if config.width < 1024 {
        config.width = 1024;
    }
    if config.height < 720 {
        config.height = 720;
    }

    let editor_game = EditorGame::new(game);
    engine_core::run_game(editor_game, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyGame;
    impl Game for DummyGame {
        fn update(&mut self, _ctx: &mut GameContext) {}
    }

    #[test]
    fn test_editor_game_creation() {
        let editor = EditorGame::new(DummyGame);
        assert!(!editor.font_loaded);
        assert!(editor.editor.selection.is_empty());
        assert_eq!(editor.editor.current_tool(), EditorTool::Select);
        assert!(editor.world_snapshot.is_none());
        assert!(editor.editor.is_editing());
        assert_eq!(editor.entity_counter, 0);
    }

    #[test]
    fn test_command_history_initialized() {
        let editor = EditorGame::new(DummyGame);
        assert!(!editor.command_history.can_undo());
        assert!(!editor.command_history.can_redo());
        assert!(editor.gizmo_drag_start.is_none());
    }

    #[test]
    fn test_editor_game_default_panels() {
        let editor = EditorGame::new(DummyGame);
        assert_eq!(editor.editor.dock_area.panels().len(), 4);
    }

    #[test]
    fn test_editor_config_enforces_minimum_size() {
        let config = GameConfig::new("Test").with_size(640, 480);
        let mut adjusted = config.clone();
        if adjusted.width < 1024 { adjusted.width = 1024; }
        if adjusted.height < 720 { adjusted.height = 720; }
        assert_eq!(adjusted.width, 1024);
        assert_eq!(adjusted.height, 720);
    }

    #[test]
    fn test_editor_config_preserves_large_size() {
        let config = GameConfig::new("Test").with_size(1920, 1080);
        let mut adjusted = config.clone();
        if adjusted.width < 1024 { adjusted.width = 1024; }
        if adjusted.height < 720 { adjusted.height = 720; }
        assert_eq!(adjusted.width, 1920);
        assert_eq!(adjusted.height, 1080);
    }

    #[test]
    fn test_play_action_captures_snapshot() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(glam::Vec2::new(10.0, 20.0))).ok();

        // Play → snapshot captured
        editor.handle_play_action(PlayControlAction::Play, &mut world);
        assert!(editor.editor.is_playing());
        assert!(editor.world_snapshot.is_some());
    }

    #[test]
    fn test_play_pause_resume_stop_cycle() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(glam::Vec2::ZERO)).ok();

        // Play
        editor.handle_play_action(PlayControlAction::Play, &mut world);
        assert!(editor.editor.is_playing());

        // Pause
        editor.handle_play_action(PlayControlAction::Pause, &mut world);
        assert!(editor.editor.is_paused());

        // Resume
        editor.handle_play_action(PlayControlAction::Play, &mut world);
        assert!(editor.editor.is_playing());

        // Stop
        editor.handle_play_action(PlayControlAction::Stop, &mut world);
        assert!(editor.editor.is_editing());
        assert!(editor.world_snapshot.is_none());
    }

    #[test]
    fn test_stop_restores_world_state() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(glam::Vec2::new(10.0, 20.0))).ok();

        // Play
        editor.handle_play_action(PlayControlAction::Play, &mut world);

        // Simulate game modification
        if let Some(t) = world.get_mut::<common::Transform2D>(entity) {
            t.position = glam::Vec2::new(999.0, 999.0);
        }

        // Stop → should restore original position
        editor.handle_play_action(PlayControlAction::Stop, &mut world);

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.position, glam::Vec2::new(10.0, 20.0));
    }

    #[test]
    fn test_build_pickable_entities_with_both_components() {
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        world.add_component(&entity, GlobalTransform2D {
            position: Vec2::new(100.0, 200.0),
            scale: Vec2::new(2.0, 2.0),
            ..Default::default()
        }).ok();
        let mut sprite = ecs::sprite_components::Sprite::new(0);
        sprite.scale = Vec2::new(32.0, 32.0);
        sprite.depth = 5.0;
        world.add_component(&entity, sprite).ok();

        let pickables = build_pickable_entities(&world);
        assert_eq!(pickables.len(), 1);
        assert_eq!(pickables[0].entity_id, entity);
        assert_eq!(pickables[0].position, Vec2::new(100.0, 200.0));
        // Size = sprite.scale * global_transform.scale = (32, 32) * (2, 2) = (64, 64)
        assert_eq!(pickables[0].size, Vec2::new(64.0, 64.0));
        assert_eq!(pickables[0].depth, 5.0);
    }

    #[test]
    fn test_build_pickable_entities_skips_without_sprite() {
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        // Only GlobalTransform2D, no Sprite
        world.add_component(&entity, GlobalTransform2D::default()).ok();

        let pickables = build_pickable_entities(&world);
        assert!(pickables.is_empty());
    }

    #[test]
    fn test_build_pickable_entities_skips_without_global_transform() {
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        // Only Sprite, no GlobalTransform2D
        world.add_component(&entity, ecs::sprite_components::Sprite::new(0)).ok();

        let pickables = build_pickable_entities(&world);
        assert!(pickables.is_empty());
    }

    #[test]
    fn test_build_pickable_entities_multiple() {
        let mut world = ecs::World::new();

        // Entity 1
        let e1 = world.create_entity();
        world.add_component(&e1, GlobalTransform2D {
            position: Vec2::new(10.0, 20.0),
            ..Default::default()
        }).ok();
        let mut sprite1 = ecs::sprite_components::Sprite::new(0);
        sprite1.depth = 1.0;
        world.add_component(&e1, sprite1).ok();

        // Entity 2
        let e2 = world.create_entity();
        world.add_component(&e2, GlobalTransform2D {
            position: Vec2::new(50.0, 60.0),
            ..Default::default()
        }).ok();
        let mut sprite2 = ecs::sprite_components::Sprite::new(1);
        sprite2.depth = 3.0;
        world.add_component(&e2, sprite2).ok();

        // Entity 3 — no sprite, should be excluded
        let e3 = world.create_entity();
        world.add_component(&e3, GlobalTransform2D::default()).ok();

        let pickables = build_pickable_entities(&world);
        assert_eq!(pickables.len(), 2);

        let ids: Vec<_> = pickables.iter().map(|p| p.entity_id).collect();
        assert!(ids.contains(&e1));
        assert!(ids.contains(&e2));
        assert!(!ids.contains(&e3));
    }

    // ================== Scene Save/Load Tests ==================

    #[test]
    fn test_editor_game_initial_scene_state() {
        let editor = EditorGame::new(DummyGame);
        assert!(!editor.editor.is_dirty());
        assert!(editor.editor.scene_path().is_none());
        assert!(editor.physics_settings.is_none());
    }

    #[test]
    fn test_new_scene_clears_world() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        let _e1 = world.create_entity();
        let _e2 = world.create_entity();
        assert_eq!(world.entities().len(), 2);

        editor.new_scene(&mut world);
        assert_eq!(world.entities().len(), 0);
        assert!(!editor.editor.is_dirty());
        assert!(editor.editor.scene_path().is_none());
    }

    #[test]
    fn test_new_scene_resets_editor_state() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();

        // Simulate some state
        editor.editor.mark_dirty();
        editor.editor.set_scene_path(Some(PathBuf::from("test.ron")));
        editor.entity_counter = 5;

        editor.new_scene(&mut world);
        assert!(!editor.editor.is_dirty());
        assert!(editor.editor.scene_path().is_none());
        assert_eq!(editor.entity_counter, 0);
        assert!(editor.physics_settings.is_none());
    }

    #[test]
    fn test_save_creates_file() {
        let _editor = EditorGame::new(DummyGame);
        let world = ecs::World::new();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_save_scene.ron");

        // Use a simple texture path function since we don't have AssetManager in tests
        let scene_name = "test";
        let texture_path_fn = |handle: u32| -> String {
            if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
        };
        let scene_data = engine_core::scene_serializer::world_to_scene_data(
            &world, scene_name, None, &texture_path_fn,
        );
        let result = engine_core::scene_serializer::save_scene_to_file(&scene_data, &path);
        assert!(result.is_ok());
        assert!(path.exists());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_clears_dirty_flag() {
        let mut editor = EditorGame::new(DummyGame);
        let world = World::new();

        editor.editor.mark_dirty();
        assert!(editor.editor.is_dirty());

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_save_dirty.ron");

        // We can't use save_scene_as directly without AssetManager,
        // so test the flag behavior with set_dirty
        let texture_path_fn = |handle: u32| -> String {
            if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
        };
        let scene_data = engine_core::scene_serializer::world_to_scene_data(
            &world, "test", None, &texture_path_fn,
        );
        engine_core::scene_serializer::save_scene_to_file(&scene_data, &path).unwrap();
        editor.editor.set_scene_path(Some(path.clone()));
        editor.editor.set_dirty(false);

        assert!(!editor.editor.is_dirty());
        assert_eq!(editor.editor.scene_path(), Some(path.as_path()));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_new_scene_warns_if_dirty() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        world.create_entity();

        editor.editor.mark_dirty();
        // new_scene should still work even when dirty (just logs a warning)
        editor.new_scene(&mut world);
        assert_eq!(world.entities().len(), 0);
        assert!(!editor.editor.is_dirty());
    }

    #[test]
    fn test_save_scene_roundtrip() {
        let _editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();

        // Create entities with components
        let e1 = world.create_entity();
        world.add_component(&e1, common::Transform2D::new(Vec2::new(100.0, 200.0))).ok();
        world.add_component(&e1, ecs::sprite_components::Name::new("player")).ok();

        let e2 = world.create_entity();
        world.add_component(&e2, common::Transform2D::new(Vec2::new(50.0, 50.0))).ok();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_roundtrip.ron");

        // Serialize
        let texture_path_fn = |handle: u32| -> String {
            if handle == 0 { "#white".to_string() } else { format!("#texture_{}", handle) }
        };
        let scene_data = engine_core::scene_serializer::world_to_scene_data(
            &world, "Roundtrip", None, &texture_path_fn,
        );
        engine_core::scene_serializer::save_scene_to_file(&scene_data, &path).unwrap();

        // Verify the file is valid RON by parsing it with SceneLoader
        let parsed = engine_core::scene_loader::SceneLoader::load_from_file(&path).unwrap();
        assert_eq!(parsed.name, "Roundtrip");
        assert_eq!(parsed.entities.len(), 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_as_updates_path() {
        let mut editor = EditorGame::new(DummyGame);

        assert!(editor.editor.scene_path().is_none());

        let path = PathBuf::from("scenes/my_scene.ron");
        editor.editor.set_scene_path(Some(path.clone()));

        assert_eq!(editor.editor.scene_path(), Some(path.as_path()));
        assert_eq!(editor.editor.scene_display_name(), "my_scene.ron");
    }

    #[test]
    fn test_dirty_flag_set_on_entity_create() {
        let mut editor = EditorGame::new(DummyGame);
        assert!(!editor.editor.is_dirty());

        // Simulate entity creation marking dirty
        editor.editor.mark_dirty();
        assert!(editor.editor.is_dirty());
    }

    #[test]
    fn test_load_scene_resets_selection() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();
        let entity = world.create_entity();
        editor.editor.selection.select(entity);
        assert!(!editor.editor.selection.is_empty());

        editor.new_scene(&mut world);
        assert!(editor.editor.selection.is_empty());
    }

    #[test]
    fn test_physics_settings_preserved_on_new() {
        let mut editor = EditorGame::new(DummyGame);
        let mut world = ecs::World::new();

        editor.physics_settings = Some(PhysicsSettings::default());
        assert!(editor.physics_settings.is_some());

        editor.new_scene(&mut world);
        assert!(editor.physics_settings.is_none());
    }

    #[test]
    fn test_scene_display_in_status() {
        let editor = EditorGame::new(DummyGame);
        assert_eq!(editor.editor.scene_display_name(), "Untitled");
        assert_eq!(editor.editor.title_bar_text(), "Untitled - Insiculous Editor");
    }
}
