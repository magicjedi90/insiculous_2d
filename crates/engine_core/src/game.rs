//! Simple game trait for rapid game development.
//!
//! This module provides a `Game` trait that hides all the complexity of
//! window management, event loops, and rendering setup. Game developers
//! just implement a few simple methods and call `run()`.
//!
//! # Example
//! ```ignore
//! use engine_core::prelude::*;
//!
//! struct MyGame {
//!     player_x: f32,
//! }
//!
//! impl Game for MyGame {
//!     fn update(&mut self, ctx: &mut GameContext) {
//!         if ctx.input.is_key_pressed(KeyCode::KeyD) {
//!             self.player_x += 5.0;
//!         }
//!     }
//! }
//!
//! fn main() {
//!     run_game(MyGame { player_x: 0.0 }, GameConfig::default()).unwrap();
//! }
//! ```

use std::collections::HashMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

use audio::AudioManager;
use input::InputHandler;
use renderer::{
    sprite::{SpriteBatch, SpriteBatcher},
    texture::TextureHandle,
};
use ui::{UIContext, DrawCommand, Color as UIColor};

/// Key for caching glyph textures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    /// Character being rendered
    character: char,
    /// Width of the glyph bitmap
    width: u32,
    /// Height of the glyph bitmap
    height: u32,
    /// Color as RGB (u8 each)
    color_rgb: [u8; 3],
}

impl GlyphCacheKey {
    fn new(character: char, width: u32, height: u32, color: &UIColor) -> Self {
        Self {
            character,
            width,
            height,
            color_rgb: [
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ],
        }
    }
}

use crate::assets::AssetManager;
use crate::render_manager::RenderManager;
use crate::window_manager::{WindowConfig, WindowManager};
use crate::Scene;
use ecs::World;
use ecs::sprite_components::{Sprite as EcsSprite, Transform2D};

/// Configuration for the game window and engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Window title
    pub title: String,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Target frames per second (0 = unlimited)
    pub target_fps: u32,
    /// Background clear color (RGBA)
    pub clear_color: [f32; 4],
    /// Whether the window is resizable
    pub resizable: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            title: "Insiculous 2D Game".to_string(),
            width: 800,
            height: 600,
            target_fps: 60,
            clear_color: [0.1, 0.1, 0.15, 1.0],
            resizable: true,
        }
    }
}

impl GameConfig {
    /// Create a new game configuration with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the clear color
    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.clear_color = [r, g, b, a];
        self
    }

    /// Set target FPS
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

/// Context passed to game methods, providing access to engine systems.
pub struct GameContext<'a> {
    /// Input handler for keyboard, mouse, and gamepad
    pub input: &'a InputHandler,
    /// The ECS world for entity/component management
    pub world: &'a mut World,
    /// Asset manager for loading textures and other resources
    pub assets: &'a mut AssetManager,
    /// Audio manager for sound playback
    pub audio: &'a mut AudioManager,
    /// UI context for immediate-mode UI
    pub ui: &'a mut UIContext,
    /// Delta time since last frame in seconds
    pub delta_time: f32,
    /// Current window size
    pub window_size: Vec2,
}

/// Render context passed to the render method.
pub struct RenderContext<'a> {
    /// The ECS world (read-only during render)
    pub world: &'a World,
    /// Sprite batcher for adding sprites to render
    pub sprites: &'a mut SpriteBatcher,
    /// The 2D camera
    pub camera: &'a mut renderer::Camera2D,
    /// Current window size
    pub window_size: Vec2,
    /// UI draw commands to render
    pub ui_commands: &'a [DrawCommand],
    /// Cached glyph textures for text rendering
    pub glyph_textures: &'a HashMap<GlyphCacheKey, TextureHandle>,
}

/// Helper function to convert UI draw commands to sprites.
///
/// UI elements render in screen space (0,0 = top-left) at high depth values
/// to appear on top of game content. Rectangles and circles use the white
/// texture (handle 0) with color tinting. Text glyphs use cached glyph textures.
fn render_ui_commands(
    sprites: &mut SpriteBatcher,
    commands: &[DrawCommand],
    window_size: Vec2,
    glyph_textures: &HashMap<GlyphCacheKey, TextureHandle>,
) {
    let white_texture = TextureHandle { id: 0 };

    for cmd in commands {
        match cmd {
            DrawCommand::Rect { bounds, color, depth, .. } => {
                // Convert screen coordinates (0,0 = top-left) to world coordinates (0,0 = center)
                let center_x = bounds.x + bounds.width / 2.0 - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - (bounds.y + bounds.height / 2.0);

                let sprite = renderer::Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(bounds.width, bounds.height))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::RectBorder { bounds, color, width, depth, .. } => {
                // Render border as 4 thin rectangles
                let half_width = *width / 2.0;

                // Top edge
                let top = ui::Rect::new(bounds.x - half_width, bounds.y - half_width, bounds.width + *width, *width);
                render_ui_rect(sprites, &top, color, *depth, window_size);

                // Bottom edge
                let bottom = ui::Rect::new(bounds.x - half_width, bounds.y + bounds.height - half_width, bounds.width + *width, *width);
                render_ui_rect(sprites, &bottom, color, *depth, window_size);

                // Left edge
                let left = ui::Rect::new(bounds.x - half_width, bounds.y + half_width, *width, bounds.height - *width);
                render_ui_rect(sprites, &left, color, *depth, window_size);

                // Right edge
                let right = ui::Rect::new(bounds.x + bounds.width - half_width, bounds.y + half_width, *width, bounds.height - *width);
                render_ui_rect(sprites, &right, color, *depth, window_size);
            }
            DrawCommand::Text { data, depth } => {
                // Render text with rasterized glyph data
                if data.glyphs.is_empty() {
                    // No glyphs - render as placeholder rectangle
                    let center_x = data.position.x + data.width / 2.0 - window_size.x / 2.0;
                    let center_y = window_size.y / 2.0 - (data.position.y + data.height / 2.0);

                    let sprite = renderer::Sprite::new(white_texture)
                        .with_position(Vec2::new(center_x, center_y))
                        .with_scale(Vec2::new(data.width.max(data.font_size * 4.0), data.height.max(data.font_size)))
                        .with_color(glam::Vec4::new(data.color.r, data.color.g, data.color.b, data.color.a * 0.3))
                        .with_depth(*depth);

                    sprites.add_sprite(&sprite);
                } else {
                    // Render each glyph using cached glyph textures
                    for glyph in &data.glyphs {
                        // Skip glyphs with no bitmap (spaces, etc.)
                        if glyph.width == 0 || glyph.height == 0 {
                            continue;
                        }

                        // Calculate glyph position in world coordinates
                        // glyph.x and glyph.y are offsets from the text origin
                        let glyph_x = data.position.x + glyph.x + glyph.width as f32 / 2.0 - window_size.x / 2.0;
                        let glyph_y = window_size.y / 2.0 - (data.position.y + glyph.y + glyph.height as f32 / 2.0);

                        // Look up glyph texture in cache
                        let glyph_key = GlyphCacheKey::new(
                            glyph.character,
                            glyph.width,
                            glyph.height,
                            &data.color,
                        );

                        let texture = glyph_textures
                            .get(&glyph_key)
                            .copied()
                            .unwrap_or(white_texture);

                        // Render glyph with its texture (white color since texture has baked color)
                        // Use minimum size of 16x16 to ensure visibility (for testing)
                        let render_width = (glyph.width as f32).max(16.0);
                        let render_height = (glyph.height as f32).max(16.0);

                        let sprite = renderer::Sprite::new(texture)
                            .with_position(Vec2::new(glyph_x, glyph_y))
                            .with_scale(Vec2::new(render_width, render_height))
                            .with_color(glam::Vec4::new(1.0, 1.0, 1.0, data.color.a))
                            .with_depth(*depth);

                        sprites.add_sprite(&sprite);
                    }

                    // Debug output removed - font rendering is working
                }
            }
            DrawCommand::TextPlaceholder { text, position, color, font_size, depth } => {
                // Placeholder: render a small rectangle where text would be
                let estimated_width = text.len() as f32 * *font_size * 0.6;
                let center_x = position.x + estimated_width / 2.0 - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - (position.y + *font_size / 2.0);

                let sprite = renderer::Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(estimated_width, *font_size))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a * 0.3))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::Circle { center, radius, color, depth } => {
                // Render circle as a square (approximation until we have circle shader)
                let center_x = center.x - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - center.y;

                let sprite = renderer::Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(*radius * 2.0, *radius * 2.0))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::Line { start, end, color, width, depth } => {
                // Render line as a thin rotated rectangle
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let length = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                let mid_x = (start.x + end.x) / 2.0 - window_size.x / 2.0;
                let mid_y = window_size.y / 2.0 - (start.y + end.y) / 2.0;

                let sprite = renderer::Sprite::new(white_texture)
                    .with_position(Vec2::new(mid_x, mid_y))
                    .with_rotation(-angle) // Negate for coordinate system
                    .with_scale(Vec2::new(length, *width))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
        }
    }
}

/// Helper to render a single UI rect as a sprite.
fn render_ui_rect(sprites: &mut SpriteBatcher, bounds: &ui::Rect, color: &UIColor, depth: f32, window_size: Vec2) {
    let white_texture = TextureHandle { id: 0 };
    let center_x = bounds.x + bounds.width / 2.0 - window_size.x / 2.0;
    let center_y = window_size.y / 2.0 - (bounds.y + bounds.height / 2.0);

    let sprite = renderer::Sprite::new(white_texture)
        .with_position(Vec2::new(center_x, center_y))
        .with_scale(Vec2::new(bounds.width, bounds.height))
        .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
        .with_depth(depth);

    sprites.add_sprite(&sprite);
}

/// The main game trait. Implement this to create your game.
///
/// Only `update` is required - all other methods have default implementations.
pub trait Game: Sized + 'static {
    /// Called once when the game starts, after the window and renderer are ready.
    /// Use this to set up your initial game state, create entities, load assets, etc.
    fn init(&mut self, _ctx: &mut GameContext) {}

    /// Called every frame. Update your game logic here.
    /// This is the only required method.
    fn update(&mut self, ctx: &mut GameContext);

    /// Called every frame to render sprites. Add sprites to `ctx.sprites`.
    /// Default implementation extracts sprites from ECS entities with Transform2D and Sprite components,
    /// then renders UI draw commands on top.
    fn render(&mut self, ctx: &mut RenderContext) {
        // Default: extract sprites from ECS
        for entity_id in ctx.world.entities() {
            let sprite = ctx.world.get::<EcsSprite>(entity_id);

            if let Some(ecs_sprite) = sprite {
                // Use GlobalTransform2D if available (for hierarchical entities),
                // otherwise fall back to local Transform2D
                let (position, rotation, scale) = if let Some(global) = ctx.world.get::<ecs::hierarchy::GlobalTransform2D>(entity_id) {
                    (global.position, global.rotation, global.scale)
                } else if let Some(transform) = ctx.world.get::<Transform2D>(entity_id) {
                    (transform.position, transform.rotation, transform.scale)
                } else {
                    continue; // No transform, skip this entity
                };

                // Use the texture handle from the ECS sprite component
                let texture = TextureHandle { id: ecs_sprite.texture_handle };
                let renderer_sprite = renderer::Sprite::new(texture)
                    .with_position(position)
                    .with_rotation(rotation)
                    .with_scale(scale * ecs_sprite.scale * 80.0)
                    .with_color(ecs_sprite.color)
                    .with_depth(ecs_sprite.depth);

                ctx.sprites.add_sprite(&renderer_sprite);
            }
        }

        // Render UI draw commands on top
        render_ui_commands(ctx.sprites, ctx.ui_commands, ctx.window_size, ctx.glyph_textures);
    }

    /// Called when a key is pressed. Override for custom key handling.
    fn on_key_pressed(&mut self, _key: KeyCode, _ctx: &mut GameContext) {}

    /// Called when a key is released. Override for custom key handling.
    fn on_key_released(&mut self, _key: KeyCode, _ctx: &mut GameContext) {}

    /// Called when the window is resized.
    fn on_resize(&mut self, _width: u32, _height: u32) {}

    /// Called when the game is about to exit. Clean up resources here.
    fn on_exit(&mut self) {}
}

/// Run a game with the given configuration.
///
/// This function handles all the window creation, event loop, and rendering
/// boilerplate. Just implement the `Game` trait and call this function.
///
/// # Example
/// ```ignore
/// struct MyGame;
///
/// impl Game for MyGame {
///     fn update(&mut self, ctx: &mut GameContext) {
///         // Game logic here
///     }
/// }
///
/// fn main() {
///     run_game(MyGame, GameConfig::default()).unwrap();
/// }
/// ```
pub fn run_game<G: Game>(game: G, config: GameConfig) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut runner = GameRunner::new(game, config);
    event_loop.run_app(&mut runner)?;
    Ok(())
}

/// Internal game runner that implements ApplicationHandler.
///
/// This struct orchestrates the game loop, delegating specific responsibilities
/// to focused managers:
/// - `WindowManager`: Window creation and size tracking
/// - `RenderManager`: Renderer lifecycle and sprite rendering
/// - `AssetManager`: Texture and asset loading
/// - `AudioManager`: Sound playback
struct GameRunner<G: Game> {
    /// The user's game implementation
    game: G,
    /// Game configuration (title, size, etc.)
    config: GameConfig,
    /// Window management
    window_manager: WindowManager,
    /// Rendering management
    render_manager: RenderManager,
    /// Asset loading and management
    asset_manager: Option<AssetManager>,
    /// Audio playback management
    audio_manager: Option<AudioManager>,
    /// Input handling
    input: InputHandler,
    /// UI context for immediate-mode UI
    ui_context: UIContext,
    /// Cached glyph textures for text rendering
    glyph_textures: HashMap<GlyphCacheKey, TextureHandle>,
    /// Main game scene containing ECS world
    scene: Scene,
    /// Whether the game's init() has been called
    initialized: bool,
    /// Time of last frame for delta calculation
    last_frame_time: std::time::Instant,
}

impl<G: Game> GameRunner<G> {
    fn new(game: G, config: GameConfig) -> Self {
        // Create window manager from game config
        let window_config = WindowConfig::new(&config.title)
            .with_size(config.width, config.height)
            .with_resizable(config.resizable);

        // Try to initialize audio (non-fatal if it fails)
        let audio_manager = match AudioManager::new() {
            Ok(audio) => Some(audio),
            Err(e) => {
                log::warn!("Failed to initialize audio: {}. Audio will be disabled.", e);
                None
            }
        };

        Self {
            game,
            config,
            window_manager: WindowManager::new(window_config),
            render_manager: RenderManager::new(),
            asset_manager: None,
            audio_manager,
            input: InputHandler::new(),
            ui_context: UIContext::new(),
            glyph_textures: HashMap::new(),
            scene: Scene::new("main"),
            initialized: false,
            last_frame_time: std::time::Instant::now(),
        }
    }

    /// Prepare glyph textures from UI draw commands.
    ///
    /// Scans Text commands for glyphs that need textures and creates them
    /// on-demand, caching them for reuse.
    fn prepare_glyph_textures(&mut self, commands: &[DrawCommand]) {
        let asset_manager = match &mut self.asset_manager {
            Some(am) => am,
            None => return,
        };

        for cmd in commands {
            if let DrawCommand::Text { data, .. } = cmd {
                for glyph in &data.glyphs {
                    // Skip empty glyphs (spaces, etc.)
                    if glyph.width == 0 || glyph.height == 0 || glyph.bitmap.is_empty() {
                        continue;
                    }

                    let key = GlyphCacheKey::new(
                        glyph.character,
                        glyph.width,
                        glyph.height,
                        &data.color,
                    );

                    // Skip if already cached
                    if self.glyph_textures.contains_key(&key) {
                        continue;
                    }

                    // Create glyph texture
                    let color_rgb = [
                        (data.color.r * 255.0) as u8,
                        (data.color.g * 255.0) as u8,
                        (data.color.b * 255.0) as u8,
                    ];

                    match asset_manager.create_glyph_texture(
                        glyph.width,
                        glyph.height,
                        &glyph.bitmap,
                        color_rgb,
                    ) {
                        Ok(handle) => {
                            self.glyph_textures.insert(key, handle);
                        }
                        Err(e) => {
                            log::warn!("Failed to create glyph texture for '{}': {}", glyph.character, e);
                        }
                    }
                }
            }
        }
    }

    /// Initialize the render manager with the current window.
    fn init_renderer(&mut self) -> Result<(), renderer::RendererError> {
        let window = self.window_manager.window_clone().ok_or_else(|| {
            renderer::RendererError::WindowCreationError("No window".to_string())
        })?;

        // Initialize render manager
        self.render_manager.init(window, self.config.clear_color)?;
        self.render_manager.set_viewport_size(
            self.config.width as f32,
            self.config.height as f32,
        );

        // Create asset manager with renderer's device and queue
        if let (Some(device), Some(queue)) = (self.render_manager.device(), self.render_manager.queue()) {
            self.asset_manager = Some(AssetManager::new(device, queue));
            log::info!("Asset manager initialized");
        }

        Ok(())
    }

    /// Helper to get window size from window manager.
    fn window_size(&self) -> Vec2 {
        let (w, h) = self.window_manager.size();
        Vec2::new(w as f32, h as f32)
    }

    fn update_and_render(&mut self) {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Get window size
        let window_size = self.window_size();

        // Check if managers are available (audio is optional)
        let has_managers = self.asset_manager.is_some();
        if !has_managers {
            return;
        }

        // Update audio manager (cleans up finished sounds)
        if let Some(audio_manager) = &mut self.audio_manager {
            audio_manager.update();
        }

        // Begin UI frame
        self.ui_context.begin_frame(&self.input, window_size);

        // Initialize game if not yet done
        if !self.initialized {
            // Get mutable references to managers
            let asset_manager = self.asset_manager.as_mut().unwrap();
            let audio_manager = self.audio_manager.as_mut();

            // Create a placeholder audio manager if none exists
            let mut placeholder_audio = AudioManager::new().ok();
            let audio = audio_manager.or(placeholder_audio.as_mut());

            if let Some(audio) = audio {
                let mut ctx = GameContext {
                    input: &self.input,
                    world: &mut self.scene.world,
                    assets: asset_manager,
                    audio,
                    ui: &mut self.ui_context,
                    delta_time,
                    window_size,
                };
                self.game.init(&mut ctx);
            }
            self.initialized = true;
        }

        // Process queued input events before game logic runs
        // This ensures keyboard/mouse state reflects this frame's events
        self.input.process_queued_events();

        // Update game logic
        {
            let asset_manager = self.asset_manager.as_mut().unwrap();
            let audio_manager = self.audio_manager.as_mut();

            let mut placeholder_audio = AudioManager::new().ok();
            let audio = audio_manager.or(placeholder_audio.as_mut());

            if let Some(audio) = audio {
                let mut ctx = GameContext {
                    input: &self.input,
                    world: &mut self.scene.world,
                    assets: asset_manager,
                    audio,
                    ui: &mut self.ui_context,
                    delta_time,
                    window_size,
                };
                self.game.update(&mut ctx);
            }
        }

        // End UI frame
        self.ui_context.end_frame();

        // Clear "just pressed/released" flags for next frame
        self.input.end_frame();

        // Skip rendering if render manager isn't ready
        if !self.render_manager.is_initialized() {
            return;
        }

        // Get UI draw commands for rendering
        let ui_commands: Vec<DrawCommand> = self.ui_context.draw_list().commands().to_vec();

        // Prepare glyph textures for text rendering
        self.prepare_glyph_textures(&ui_commands);

        // Build sprite batches
        let mut batcher = SpriteBatcher::new(1000);

        {
            let mut ctx = RenderContext {
                world: &self.scene.world,
                sprites: &mut batcher,
                camera: self.render_manager.camera_mut(),
                window_size,
                ui_commands: &ui_commands,
                glyph_textures: &self.glyph_textures,
            };
            self.game.render(&mut ctx);
        }

        // Collect batches and render with asset manager's textures
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();

        // Get textures from asset manager (need to reborrow after RenderContext)
        if let Some(asset_manager) = &self.asset_manager {
            let textures = asset_manager.textures_cloned();
            if let Err(e) = self.render_manager.render(&batch_refs, &textures) {
                log::error!("Render error: {}", e);
            }
        }
    }
}

impl<G: Game> ApplicationHandler<()> for GameRunner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Only create window once
        if self.window_manager.is_created() {
            return;
        }

        // Create window using window manager
        if let Err(e) = self.window_manager.create(event_loop) {
            log::error!("Failed to create window: {}", e);
            event_loop.exit();
            return;
        }

        // Initialize renderer
        if let Err(e) = self.init_renderer() {
            log::error!("Failed to initialize renderer: {}", e);
            event_loop.exit();
            return;
        }

        // Initialize scene lifecycle
        if let Err(e) = self.scene.initialize() {
            log::error!("Scene init error: {}", e);
        }
        if let Err(e) = self.scene.start() {
            log::error!("Scene start error: {}", e);
        }

        log::info!("Game started: {}", self.config.title);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Only handle events for our window
        if !self.window_manager.is_our_window(window_id) {
            return;
        }

        // Forward to input handler
        self.input.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                self.game.on_exit();
                let _ = self.scene.stop();
                let _ = self.scene.shutdown();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                // Update window manager's tracked size
                self.window_manager.resize(size.width, size.height);
                // Update render manager
                self.render_manager.resize(size.width, size.height);
                // Notify game
                self.game.on_resize(size.width, size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    // Handle escape to exit early (before we need managers)
                    if key == KeyCode::Escape && event.state == ElementState::Pressed {
                        self.game.on_exit();
                        let _ = self.scene.stop();
                        let _ = self.scene.shutdown();
                        event_loop.exit();
                        return;
                    }

                    // For other keys, create context and call handlers
                    let window_size = self.window_size();
                    if let (Some(asset_manager), Some(audio_manager)) =
                        (&mut self.asset_manager, &mut self.audio_manager)
                    {
                        let mut ctx = GameContext {
                            input: &self.input,
                            world: &mut self.scene.world,
                            assets: asset_manager,
                            audio: audio_manager,
                            ui: &mut self.ui_context,
                            delta_time: 0.0,
                            window_size,
                        };

                        match event.state {
                            ElementState::Pressed => {
                                self.game.on_key_pressed(key, &mut ctx);
                            }
                            ElementState::Released => {
                                self.game.on_key_released(key, &mut ctx);
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Rendering is done in about_to_wait
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.update_and_render();
        self.window_manager.request_redraw();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_config_default() {
        let config = GameConfig::default();
        assert_eq!(config.title, "Insiculous 2D Game");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.target_fps, 60);
        assert!(config.resizable);
    }

    #[test]
    fn test_game_config_builder() {
        let config = GameConfig::new("Test Game")
            .with_size(1024, 768)
            .with_fps(120)
            .with_clear_color(0.5, 0.5, 0.5, 1.0);

        assert_eq!(config.title, "Test Game");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.target_fps, 120);
        assert_eq!(config.clear_color, [0.5, 0.5, 0.5, 1.0]);
    }
}
