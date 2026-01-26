//! Editor demo - demonstrates the editor UI framework.
//!
//! Run with: cargo run --example editor_demo

use engine_core::prelude::*;
use editor::prelude::*;
use glam::Vec2;

struct EditorDemo {
    editor: EditorContext,
    font_loaded: bool,
}

impl Default for EditorDemo {
    fn default() -> Self {
        Self {
            editor: EditorContext::new(),
            font_loaded: false,
        }
    }
}

impl Game for EditorDemo {
    fn init(&mut self, ctx: &mut GameContext) {
        // Try to load a font for text rendering
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
                log::info!("Font loaded from: {}", path);
                break;
            }
        }

        if !self.font_loaded {
            log::warn!("No font loaded. Text will render as placeholders.");
            log::warn!("To enable font rendering, add a .ttf file to examples/assets/fonts/font.ttf");
        }

        // Create some test entities to edit
        let entity1 = ctx.world.create_entity();
        ctx.world.add_component(&entity1, ecs::sprite_components::Transform2D::new(Vec2::new(-100.0, 0.0))).ok();

        let entity2 = ctx.world.create_entity();
        ctx.world.add_component(&entity2, ecs::sprite_components::Transform2D::new(Vec2::new(100.0, 50.0))).ok();

        let entity3 = ctx.world.create_entity();
        ctx.world.add_component(&entity3, ecs::sprite_components::Transform2D::new(Vec2::new(0.0, -100.0))).ok();

        log::info!("Editor demo initialized with 3 test entities");
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let window_size = ctx.window_size;

        // Update editor layout
        self.editor.update_layout(window_size);

        // Render menu bar
        if let Some(action) = self.editor.menu_bar.render(ctx.ui, window_size.x) {
            log::info!("Menu action: {}", action);
            self.handle_menu_action(&action);
        }

        // Render toolbar
        if let Some(tool) = self.editor.toolbar.render(ctx.ui) {
            log::info!("Tool changed: {:?}", tool);
        }

        // Render dock panels
        let content_areas = self.editor.dock_area.render(ctx.ui);

        // Handle panel resize
        self.editor.dock_area.handle_resize(ctx.ui);

        // Render content in each panel
        for (panel_id, bounds) in content_areas.clone() {
            self.render_panel_content(ctx, panel_id, bounds);
        }

        // Handle gizmo interaction for selected entity
        if let Some(entity_id) = self.editor.selection.primary() {
            if let Some(scene_bounds) = content_areas.iter()
                .find(|(id, _)| *id == PanelId::SCENE_VIEW)
                .map(|(_, b)| *b)
            {
                // Get entity position (clone to release borrow)
                let entity_pos = ctx.world
                    .get::<ecs::sprite_components::Transform2D>(entity_id)
                    .map(|t| t.position);

                if let Some(entity_pos) = entity_pos {
                    // Convert world position to screen position (simple viewport mapping)
                    let viewport_center = scene_bounds.center();
                    let zoom = self.editor.camera_zoom();
                    let camera_offset = self.editor.camera_offset();

                    // Screen position = viewport_center + (world_pos - camera_offset) * zoom
                    let screen_pos = viewport_center + (entity_pos - camera_offset) * zoom;

                    // Render gizmo and get interaction
                    let interaction = self.editor.gizmo.render(ctx.ui, screen_pos);

                    // Apply gizmo delta to entity transform
                    if interaction.handle.is_some() && interaction.delta != Vec2::ZERO {
                        // Convert screen delta to world delta
                        let world_delta = self.editor.gizmo_delta_to_world(interaction.delta);
                        let snap_enabled = self.editor.is_snap_to_grid();

                        // Apply to entity
                        if let Some(transform) = ctx.world.get_mut::<ecs::sprite_components::Transform2D>(entity_id) {
                            transform.position += world_delta;

                            // Apply grid snapping if enabled
                            if snap_enabled {
                                transform.position = self.editor.snap_position(transform.position);
                            }
                        }
                    }
                }
            }
        }

        // Handle keyboard shortcuts for tools
        self.handle_tool_shortcuts(ctx);

        // Show editor info
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

    fn on_key_pressed(&mut self, key: winit::keyboard::KeyCode, _ctx: &mut GameContext) {
        use winit::keyboard::KeyCode;

        match key {
            KeyCode::KeyG => self.editor.toggle_grid(),
            KeyCode::KeyS if _ctx.input.keyboard().is_key_pressed(KeyCode::ControlLeft) => {
                log::info!("Save scene (Ctrl+S)");
            }
            KeyCode::Equal => self.editor.zoom_camera(1.1),
            KeyCode::Minus => self.editor.zoom_camera(0.9),
            KeyCode::Digit0 => self.editor.reset_camera(),
            KeyCode::Space => self.editor.toggle_play_mode(),
            _ => {}
        }
    }
}

impl EditorDemo {
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
        use winit::keyboard::KeyCode;
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

    fn render_panel_content(&mut self, ctx: &mut GameContext, panel_id: PanelId, bounds: common::Rect) {
        let padding = 8.0;
        let content_x = bounds.x + padding;
        let mut y = bounds.y + padding;
        let line_height = 20.0;

        match panel_id {
            PanelId::SCENE_VIEW => {
                // Scene view - show grid info
                ctx.ui.label("Scene View", Vec2::new(content_x, y));
                y += line_height;

                if self.editor.is_grid_visible() {
                    ctx.ui.label(
                        &format!("Grid: {}px", self.editor.grid_size()),
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

                // Gizmo is rendered in main update() method for proper transform handling
            }
            PanelId::HIERARCHY => {
                ctx.ui.label("Hierarchy", Vec2::new(content_x, y));
                y += line_height * 1.5;

                // List entities
                for (i, entity_id) in ctx.world.entities().into_iter().enumerate() {
                    let is_selected = self.editor.selection.contains(entity_id);
                    let prefix = if is_selected { "> " } else { "  " };
                    let label = format!("{}Entity {}", prefix, entity_id.value());

                    let item_bounds = common::Rect::new(
                        content_x,
                        y,
                        bounds.width - padding * 2.0,
                        line_height,
                    );

                    // Highlight selected
                    if is_selected {
                        ctx.ui.rect(item_bounds, ui::Color::new(0.2, 0.3, 0.5, 0.5));
                    }

                    let id = format!("hierarchy_entity_{}", i);
                    if ctx.ui.button(id.as_str(), &label, item_bounds) {
                        if ctx.input.keyboard().is_key_pressed(winit::keyboard::KeyCode::ControlLeft) {
                            self.editor.selection.toggle(entity_id);
                        } else {
                            self.editor.selection.select(entity_id);
                        }
                        log::info!("Selected entity: {}", entity_id.value());
                    }

                    y += line_height;
                    if y > bounds.y + bounds.height - padding {
                        break;
                    }
                }
            }
            PanelId::INSPECTOR => {
                ctx.ui.label("Inspector", Vec2::new(content_x, y));
                y += line_height * 1.5;

                if let Some(entity_id) = self.editor.selection.primary() {
                    ctx.ui.label(
                        &format!("Entity: {}", entity_id.value()),
                        Vec2::new(content_x, y),
                    );
                    y += line_height;

                    // Use generic inspector for all components
                    let style = InspectorStyle::default();

                    // Inspect Transform2D if present
                    if let Some(transform) = ctx.world.get::<ecs::sprite_components::Transform2D>(entity_id) {
                        y += line_height * 0.5;
                        y = inspect_component(ctx.ui, "Transform2D", &*transform, content_x, y, &style);
                    }

                    // Inspect Sprite if present
                    if let Some(sprite) = ctx.world.get::<ecs::sprite_components::Sprite>(entity_id) {
                        y += line_height * 0.5;
                        y = inspect_component(ctx.ui, "Sprite", &*sprite, content_x, y, &style);
                    }

                    // Inspect Camera if present
                    if let Some(camera) = ctx.world.get::<ecs::sprite_components::Camera>(entity_id) {
                        y += line_height * 0.5;
                        y = inspect_component(ctx.ui, "Camera", &*camera, content_x, y, &style);
                    }

                    // Inspect SpriteAnimation if present
                    if let Some(animation) = ctx.world.get::<ecs::sprite_components::SpriteAnimation>(entity_id) {
                        y += line_height * 0.5;
                        let _ = inspect_component(ctx.ui, "SpriteAnimation", &*animation, content_x, y, &style);
                    }
                } else {
                    ctx.ui.label("No selection", Vec2::new(content_x, y));
                }
            }
            PanelId::ASSET_BROWSER => {
                ctx.ui.label("Assets", Vec2::new(content_x, y));
                y += line_height * 1.5;
                ctx.ui.label("(Asset browser not yet implemented)", Vec2::new(content_x, y));
            }
            _ => {
                ctx.ui.label("Panel", Vec2::new(content_x, y));
            }
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = GameConfig::new("Insiculous 2D - Editor Demo")
        .with_size(1280, 720);

    if let Err(e) = run_game(EditorDemo::default(), config) {
        log::error!("Editor error: {}", e);
    }
}
