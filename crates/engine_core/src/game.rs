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

use glam::Vec2;
use serde::{Deserialize, Serialize};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

use input::InputHandler;
use renderer::{
    sprite::{SpriteBatch, SpriteBatcher},
    texture::TextureHandle,
};

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
    /// Default implementation extracts sprites from ECS entities with Transform2D and Sprite components.
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
    /// Input handling
    input: InputHandler,
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

        Self {
            game,
            config,
            window_manager: WindowManager::new(window_config),
            render_manager: RenderManager::new(),
            asset_manager: None,
            input: InputHandler::new(),
            scene: Scene::new("main"),
            initialized: false,
            last_frame_time: std::time::Instant::now(),
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

        // Get asset manager or return early (single check for entire frame)
        let Some(asset_manager) = &mut self.asset_manager else {
            return;
        };

        // Initialize game if not yet done
        if !self.initialized {
            let mut ctx = GameContext {
                input: &self.input,
                world: &mut self.scene.world,
                assets: asset_manager,
                delta_time,
                window_size,
            };
            self.game.init(&mut ctx);
            self.initialized = true;
        }

        // Process queued input events before game logic runs
        // This ensures keyboard/mouse state reflects this frame's events
        self.input.process_queued_events();

        // Update game logic
        let mut ctx = GameContext {
            input: &self.input,
            world: &mut self.scene.world,
            assets: asset_manager,
            delta_time,
            window_size,
        };
        self.game.update(&mut ctx);

        // Clear "just pressed/released" flags for next frame
        self.input.end_frame();

        // Skip rendering if render manager isn't ready
        if !self.render_manager.is_initialized() {
            return;
        }

        // Build sprite batches
        let mut batcher = SpriteBatcher::new(1000);

        {
            let mut ctx = RenderContext {
                world: &self.scene.world,
                sprites: &mut batcher,
                camera: self.render_manager.camera_mut(),
                window_size,
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
                    // Handle escape to exit early (before we need asset_manager)
                    if key == KeyCode::Escape && event.state == ElementState::Pressed {
                        self.game.on_exit();
                        let _ = self.scene.stop();
                        let _ = self.scene.shutdown();
                        event_loop.exit();
                        return;
                    }

                    // For other keys, create context and call handlers
                    let window_size = self.window_size();
                    if let Some(asset_manager) = &mut self.asset_manager {
                        let mut ctx = GameContext {
                            input: &self.input,
                            world: &mut self.scene.world,
                            assets: asset_manager,
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
