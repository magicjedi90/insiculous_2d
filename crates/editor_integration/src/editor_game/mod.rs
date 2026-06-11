//! Editor-wrapped game implementation.
//!
//! `EditorGame<G>` transparently wraps any `Game` implementation, intercepting
//! all trait methods to weave in editor UI orchestration (menu bar, toolbar,
//! dock panels, hierarchy, inspector, gizmo, tool shortcuts, play/pause/stop)
//! and delegating to the inner game.
//!
//! The wrapper is split by feature:
//! - [`menu_actions`] — menu bar rendering and action dispatch
//! - [`scene_io`] — scene save/load/new
//! - [`shortcuts`] — keyboard shortcuts and play state transitions
//! - [`viewport_interaction`] — viewport picking and gizmo dragging

use glam::Vec2;
use winit::keyboard::KeyCode;

use ecs::System;
use editor::EditorContext;
use editor::world_snapshot::WorldSnapshot;
use engine_core::contexts::{GameContext, RenderContext};
use engine_core::scene_data::PhysicsSettings;
use engine_core::Game;
use engine_core::GameConfig;

use crate::constants::{MIN_EDITOR_WINDOW_HEIGHT, MIN_EDITOR_WINDOW_WIDTH};
use crate::panel_renderer;

mod menu_actions;
mod scene_io;
mod shortcuts;
mod viewport_interaction;

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

    /// Render the toolbar and the play controls next to it.
    fn render_toolbar_and_play_controls(&mut self, ctx: &mut GameContext) {
        if let Some(tool) = self.editor.toolbar.render(ctx.ui, &self.editor.theme) {
            log::info!("Tool changed: {:?}", tool);
        }

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

    /// Render the dock panel frames and their content. Returns the panel
    /// content areas for later viewport/gizmo hit testing.
    fn render_panels(&mut self, ctx: &mut GameContext) -> Vec<(editor::PanelId, common::Rect)> {
        let theme = &self.editor.theme;
        let content_areas = self.editor.dock_area.render(ctx.ui, theme);
        self.editor.dock_area.handle_resize(ctx.ui);

        for (panel_id, bounds) in content_areas.clone() {
            ctx.ui.push_clip_rect(ui::Rect::new(bounds.x, bounds.y, bounds.width, bounds.height));
            panel_renderer::render_panel_content(
                &mut self.editor, ctx, panel_id, bounds, &mut self.command_history,
            );
            ctx.ui.pop_clip_rect();
        }

        content_areas
    }

    /// Delegate the frame to the inner game — only while Playing, clipped to
    /// the scene view.
    fn update_inner_game(&mut self, ctx: &mut GameContext) {
        if !self.editor.is_playing() {
            return;
        }
        if let Some(scene_bounds) = self.editor.scene_view_bounds() {
            ctx.ui.push_clip_rect(ui::Rect::new(
                scene_bounds.x, scene_bounds.y, scene_bounds.width, scene_bounds.height,
            ));
        }
        self.inner.update(ctx);
        if self.editor.scene_view_bounds().is_some() {
            ctx.ui.pop_clip_rect();
        }
    }

    /// Update status bar stats and render it.
    fn render_status_bar(&mut self, ctx: &mut GameContext, window_size: Vec2) {
        let fps = if ctx.delta_time > 0.0 { 1.0 / ctx.delta_time } else { 0.0 };
        let smoothed_fps = fps.min(999.0); // Cap for display
        self.editor.status_bar.update_stats(ctx.world.entity_count(), smoothed_fps);
        self.editor.status_bar.update(ctx.delta_time);

        let theme = &self.editor.theme;
        self.editor.status_bar.render(ctx.ui, window_size, theme);
    }
}

impl<G: Game> Game for EditorGame<G> {
    fn init(&mut self, ctx: &mut GameContext) {
        // Load font from common search paths
        let font_paths = [
            "assets/fonts/font.ttf",
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

        // 3. Menu bar + action dispatch
        self.handle_menu_bar(ctx, window_size);

        // 4. Toolbar + play controls
        self.render_toolbar_and_play_controls(ctx);

        // 5. Dock panels + content
        let content_areas = self.render_panels(ctx);

        // 6. Viewport input (pan, zoom, click, rectangle selection)
        self.handle_viewport_picking(ctx);

        // 7. Gizmo interaction for the selected entity
        self.handle_gizmo(ctx, &content_areas);

        // 8. Tool keyboard shortcuts (skip during play)
        if !self.editor.is_playing() {
            self.handle_tool_shortcuts(ctx);
        }

        // 9. Delegate to inner game (only when Playing)
        self.update_inner_game(ctx);

        // 10. Status bar
        self.render_status_bar(ctx, window_size);
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.inner.render(ctx);
    }

    fn on_key_pressed(&mut self, key: KeyCode, ctx: &mut GameContext) {
        self.handle_editor_key(key, ctx);
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
    if config.width < MIN_EDITOR_WINDOW_WIDTH {
        config.width = MIN_EDITOR_WINDOW_WIDTH;
    }
    if config.height < MIN_EDITOR_WINDOW_HEIGHT {
        config.height = MIN_EDITOR_WINDOW_HEIGHT;
    }

    let editor_game = EditorGame::new(game);
    engine_core::run_game(editor_game, config)
}

#[cfg(test)]
mod tests;
