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

use crate::{GameLoopManager, UIManager};
use crate::game_config::GameConfig;
use crate::ui_integration::render_ui_commands;
use ui::DrawCommand;
use crate::contexts::{GameContext, RenderContext, GlyphCacheKey};
use crate::assets::AssetManager;
use crate::render_manager::RenderManager;
use crate::window_manager::{WindowConfig, WindowManager};
use crate::Scene;

use ecs::sprite_components::{Sprite as EcsSprite, Transform2D};

/// Helper function to convert UI draw commands to sprites.
///
/// UI elements render in screen space (0,0 = top-left) at high depth values
/// to appear on top of game content. Rectangles and circles use the white
/// texture (handle 0) with color tinting. Text glyphs use cached glyph textures.


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
/// - `GameLoopManager`: Frame timing and delta calculation
/// - `UIManager`: UI lifecycle and draw command collection
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
    /// UI management
    ui_manager: UIManager,
    /// Game loop timing and frame management
    game_loop_manager: GameLoopManager,
    /// Cached glyph textures for text rendering
    glyph_textures: HashMap<GlyphCacheKey, TextureHandle>,
    /// Main game scene containing ECS world
    scene: Scene,
    /// Whether the game's init() has been called
    initialized: bool,
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
            ui_manager: UIManager::new(),
            game_loop_manager: GameLoopManager::new(),
            glyph_textures: HashMap::new(),
            scene: Scene::new("main"),
            initialized: false,
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

                    // Cache key is color-agnostic - same texture can be reused for any color
                    let key = GlyphCacheKey::new(
                        glyph.character,
                        glyph.width,
                        glyph.height,
                    );

                    // Skip if already cached
                    if self.glyph_textures.contains_key(&key) {
                        continue;
                    }

                    // Create glyph texture (grayscale alpha mask)
                    match asset_manager.create_glyph_texture(
                        glyph.width,
                        glyph.height,
                        &glyph.bitmap,
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
        // Update game loop timing
        let delta_time = self.game_loop_manager.update();
        let window_size = self.window_size();

        // Check if managers are available (audio is optional)
        let has_managers = self.asset_manager.is_some();
        if !has_managers {
            return;
        }

        // Process queued input events FIRST so UI sees fresh mouse/keyboard state
        self.input.process_queued_events();

        // Update all subsystems
        self.update_audio();
        self.update_ui_begin(window_size);
        self.initialize_if_needed(delta_time, window_size);
        self.update_game_logic(delta_time, window_size);
        let ui_commands = self.update_ui_end();
        self.update_input_end();

        // Render frame if ready
        if self.render_manager.is_initialized() {
            self.render_frame(window_size, &ui_commands);
        }
    }

    /// Update audio manager to clean up finished sounds
    fn update_audio(&mut self) {
        if let Some(audio_manager) = &mut self.audio_manager {
            audio_manager.update();
        }
    }

    /// Begin UI frame and process input
    fn update_ui_begin(&mut self, window_size: Vec2) {
        self.ui_manager.begin_frame(&self.input, window_size);
    }

    /// Initialize game on first frame
    fn initialize_if_needed(&mut self, delta_time: f32, window_size: Vec2) {
        if self.initialized {
            return;
        }

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
                ui: &mut self.ui_manager.ui_context(),
                delta_time,
                window_size,
            };
            self.game.init(&mut ctx);
        }
        self.initialized = true;
    }

    /// Update game logic
    fn update_game_logic(&mut self, delta_time: f32, window_size: Vec2) {
        // Update game logic
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
                ui: &mut self.ui_manager.ui_context(),
                delta_time,
                window_size,
            };
            self.game.update(&mut ctx);
        }
    }

    /// End UI frame and return draw commands
    fn update_ui_end(&mut self) -> Vec<DrawCommand> {
        self.ui_manager.end_frame()
    }

    /// Clear input state for next frame
    fn update_input_end(&mut self) {
        self.input.end_frame();
    }

    /// Render complete frame with sprites and UI
    fn render_frame(&mut self, window_size: Vec2, ui_commands: &[DrawCommand]) {
        // Prepare glyph textures for text rendering
        self.prepare_glyph_textures(ui_commands);

        // Build sprite batches
        let mut batcher = SpriteBatcher::new(1000);

        {
            let mut ctx = RenderContext {
                world: &self.scene.world,
                sprites: &mut batcher,
                camera: self.render_manager.camera_mut(),
                window_size,
                ui_commands,
                glyph_textures: &self.glyph_textures,
            };
            self.game.render(&mut ctx);
        }

        // Collect batches, sort by depth, and render with asset manager's textures
        let mut batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        // Sort batches by their minimum depth (ascending for back-to-front rendering)
        batches.sort_by(|a, b| {
            let a_min = a.instances.iter().map(|i| i.depth).min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap_or(0.0);
            let b_min = b.instances.iter().map(|i| i.depth).min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap_or(0.0);
            a_min.partial_cmp(&b_min).unwrap()
        });
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
                            ui: &mut self.ui_manager.ui_context(),
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


