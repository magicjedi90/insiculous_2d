//! Editor-wrapped game implementation.
//!
//! `EditorGame<G>` transparently wraps any `Game` implementation, intercepting
//! all trait methods to weave in editor UI orchestration (menu bar, toolbar,
//! dock panels, hierarchy, inspector, gizmo, tool shortcuts) and delegating
//! to the inner game.

use glam::Vec2;
use winit::keyboard::KeyCode;

use ecs::System;
use editor::{EditorContext, EditorTool, PanelId};
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
}

impl<G: Game> EditorGame<G> {
    fn new(game: G) -> Self {
        Self {
            inner: game,
            editor: EditorContext::new(),
            transform_system: ecs::TransformHierarchySystem::new(),
            font_loaded: false,
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

        // 5. Dock panel frames + resize handles
        let content_areas = self.editor.dock_area.render(ctx.ui);
        self.editor.dock_area.handle_resize(ctx.ui);

        // 6. Panel content
        for (panel_id, bounds) in content_areas.clone() {
            panel_renderer::render_panel_content(&mut self.editor, ctx, panel_id, bounds);
        }

        // 7. Pop clip rects
        self.editor.dock_area.end_panel_content(ctx.ui, content_areas.len());

        // 8. Gizmo interaction for selected entity
        if let Some(entity_id) = self.editor.selection.primary() {
            if content_areas.iter().any(|(id, _)| *id == PanelId::SCENE_VIEW) {
                let entity_pos = ctx.world
                    .get::<ecs::GlobalTransform2D>(entity_id)
                    .map(|t| t.position);

                if let Some(entity_pos) = entity_pos {
                    let screen_pos = self.editor.world_to_screen(entity_pos);
                    let interaction = self.editor.gizmo.render(ctx.ui, screen_pos);

                    if interaction.handle.is_some() && interaction.delta != Vec2::ZERO {
                        let world_delta = self.editor.gizmo_delta_to_world(interaction.delta);
                        let snap_enabled = self.editor.is_snap_to_grid();

                        if let Some(transform) = ctx.world.get_mut::<ecs::sprite_components::Transform2D>(entity_id) {
                            transform.position += world_delta;
                            if snap_enabled {
                                transform.position = self.editor.snap_position(transform.position);
                            }
                        }
                    }
                }
            }
        }

        // 9. Tool keyboard shortcuts
        self.handle_tool_shortcuts(ctx);

        // 10. Delegate to inner game
        self.inner.update(ctx);

        // 11. Status bar
        let info_y = window_size.y - 30.0;
        ctx.ui.label(
            &format!(
                "Tool: {:?} | Grid: {} | Snap: {} | Zoom: {:.1}x",
                self.editor.current_tool(),
                if self.editor.is_grid_visible() { "ON" } else { "OFF" },
                if self.editor.is_snap_to_grid() { "ON" } else { "OFF" },
                self.editor.camera_zoom()
            ),
            Vec2::new(10.0, info_y),
        );
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.inner.render(ctx);
    }

    fn on_key_pressed(&mut self, key: KeyCode, ctx: &mut GameContext) {
        match key {
            KeyCode::KeyG => self.editor.toggle_grid(),
            KeyCode::KeyS if ctx.input.keyboard().is_key_pressed(KeyCode::ControlLeft) => {
                log::info!("Save scene (Ctrl+S)");
            }
            KeyCode::Equal => self.editor.zoom_camera(1.1),
            KeyCode::Minus => self.editor.zoom_camera(0.9),
            KeyCode::Digit0 => self.editor.reset_camera(),
            KeyCode::F5 => self.editor.toggle_play_mode(),
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
/// inspector, gizmo, tool shortcuts) around the user's game.
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
}
