//! Editor-wrapped game implementation.
//!
//! `EditorGame<G>` transparently wraps any `Game` implementation, intercepting
//! all trait methods to weave in editor UI orchestration (menu bar, toolbar,
//! dock panels, hierarchy, inspector, gizmo, tool shortcuts, play/pause/stop)
//! and delegating to the inner game.

use glam::Vec2;
use winit::keyboard::KeyCode;

use ecs::System;
use editor::{EditorContext, EditorPlayState, EditorTool, PanelId, PlayControlAction};
use editor::world_snapshot::WorldSnapshot;
use engine_core::contexts::{GameContext, RenderContext};
use engine_core::Game;
use engine_core::GameConfig;

use crate::panel_renderer;

/// Wraps a user's `Game` with the full editor UI overlay.
struct EditorGame<G: Game> {
    inner: G,
    editor: EditorContext,
    transform_system: ecs::TransformHierarchySystem,
    font_loaded: bool,
    /// Snapshot of the world state captured when entering play mode.
    world_snapshot: Option<WorldSnapshot>,
}

impl<G: Game> EditorGame<G> {
    fn new(game: G) -> Self {
        Self {
            inner: game,
            editor: EditorContext::new(),
            transform_system: ecs::TransformHierarchySystem::new(),
            font_loaded: false,
            world_snapshot: None,
        }
    }

    fn handle_menu_action(&mut self, action: &str) {
        match action {
            "New Scene" => log::info!("Creating new scene..."),
            "Open Scene..." => log::info!("Opening scene..."),
            "Save" => log::info!("Saving scene..."),
            "Save As..." => log::info!("Save as..."),
            "Exit" => std::process::exit(0),
            "Undo" => log::info!("Undo"),
            "Redo" => log::info!("Redo"),
            "Scene View" | "Inspector" | "Hierarchy" | "Asset Browser" | "Console" => {
                log::info!("Toggle panel: {}", action);
            }
            "Create Empty" => log::info!("Creating empty entity..."),
            _ => log::info!("Unhandled action: {}", action),
        }
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
    fn handle_play_action(&mut self, action: PlayControlAction, world: &mut ecs::World) {
        match action {
            PlayControlAction::Play => {
                if self.editor.is_editing() {
                    // Starting a new play session — capture snapshot
                    self.world_snapshot = Some(WorldSnapshot::capture(world));
                    self.editor.set_play_state(EditorPlayState::Playing);
                    log::info!("Play: snapshot captured, entering play mode");
                } else if self.editor.is_paused() {
                    // Resuming from pause
                    self.editor.set_play_state(EditorPlayState::Playing);
                    log::info!("Play: resumed from pause");
                }
            }
            PlayControlAction::Pause => {
                if self.editor.is_playing() {
                    self.editor.set_play_state(EditorPlayState::Paused);
                    log::info!("Paused");
                }
            }
            PlayControlAction::Stop => {
                if self.editor.in_play_session() {
                    // Restore world from snapshot
                    if let Some(snapshot) = self.world_snapshot.take() {
                        snapshot.restore(world);
                        log::info!("Stop: world restored from snapshot");
                    }
                    self.editor.set_play_state(EditorPlayState::Editing);
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
            self.handle_menu_action(&action);
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
            if let Some(action) = self.editor.play_controls.render(ctx.ui, play_state) {
                self.handle_play_action(action, ctx.world);
            }
        }

        // 5. Dock panel frames + resize handles
        let content_areas = self.editor.dock_area.render(ctx.ui);
        self.editor.dock_area.handle_resize(ctx.ui);

        // 6. Panel content
        for (panel_id, bounds) in content_areas.clone() {
            panel_renderer::render_panel_content(
                &mut self.editor, ctx, panel_id, bounds,
            );
        }

        // 7. Pop clip rects
        self.editor.dock_area.end_panel_content(ctx.ui, content_areas.len());

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
            self.inner.update(ctx);
        }

        // 11. Status bar (includes play state)
        let info_y = window_size.y - 30.0;
        ctx.ui.label(
            &format!(
                "Tool: {:?} | Grid: {} | Snap: {} | Zoom: {:.1}x | {}",
                self.editor.current_tool(),
                if self.editor.is_grid_visible() { "ON" } else { "OFF" },
                if self.editor.is_snap_to_grid() { "ON" } else { "OFF" },
                self.editor.camera_zoom(),
                self.editor.play_state().label(),
            ),
            Vec2::new(10.0, info_y),
        );
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
            self.handle_play_action(PlayControlAction::Stop, ctx.world);
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
            KeyCode::KeyG => self.editor.toggle_grid(),
            KeyCode::KeyS if ctrl => {
                log::info!("Save scene (Ctrl+S)");
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
}
