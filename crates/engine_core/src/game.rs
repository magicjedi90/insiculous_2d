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
use std::sync::Arc;

use glam::Vec2;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

use input::InputHandler;
use renderer::{
    sprite::{SpriteBatch, SpriteBatcher, SpritePipeline, TextureHandle},
    Camera2D, Renderer, TextureResource,
};

use crate::Scene;
use ecs::World;
use ecs::sprite_components::{Sprite as EcsSprite, Transform2D};

/// Configuration for the game window and engine.
#[derive(Debug, Clone)]
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
    pub camera: &'a mut Camera2D,
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
        let white_texture = TextureHandle { id: 0 };

        for entity_id in ctx.world.entities() {
            let transform = ctx.world.get::<Transform2D>(entity_id);
            let sprite = ctx.world.get::<EcsSprite>(entity_id);

            if let (Some(transform), Some(ecs_sprite)) = (transform, sprite) {
                let renderer_sprite = renderer::Sprite::new(white_texture)
                    .with_position(transform.position)
                    .with_rotation(transform.rotation)
                    .with_scale(transform.scale * ecs_sprite.scale * 80.0)
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
struct GameRunner<G: Game> {
    game: G,
    config: GameConfig,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    input: InputHandler,
    scene: Scene,
    initialized: bool,
    last_frame_time: std::time::Instant,
}

impl<G: Game> GameRunner<G> {
    fn new(game: G, config: GameConfig) -> Self {
        Self {
            game,
            config,
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            input: InputHandler::new(),
            scene: Scene::new("main"),
            initialized: false,
            last_frame_time: std::time::Instant::now(),
        }
    }

    fn init_renderer(&mut self) -> Result<(), renderer::RendererError> {
        let window = self.window.as_ref().ok_or_else(|| {
            renderer::RendererError::WindowCreationError("No window".to_string())
        })?;

        let mut renderer = pollster::block_on(renderer::init(window.clone()))?;
        renderer.set_clear_color(
            self.config.clear_color[0] as f64,
            self.config.clear_color[1] as f64,
            self.config.clear_color[2] as f64,
            self.config.clear_color[3] as f64,
        );

        let sprite_pipeline = SpritePipeline::new(renderer.device(), 1000);

        self.camera.viewport_size = Vec2::new(self.config.width as f32, self.config.height as f32);
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);

        Ok(())
    }

    fn update_and_render(&mut self) {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        let window_size = Vec2::new(self.config.width as f32, self.config.height as f32);

        // Initialize game if not yet done
        if !self.initialized {
            let mut ctx = GameContext {
                input: &self.input,
                world: &mut self.scene.world,
                delta_time,
                window_size,
            };
            self.game.init(&mut ctx);
            self.initialized = true;
        }

        // Update
        {
            let mut ctx = GameContext {
                input: &self.input,
                world: &mut self.scene.world,
                delta_time,
                window_size,
            };
            self.game.update(&mut ctx);
        }

        // Update input state (clear "just pressed" flags)
        self.input.update();

        // Render
        let (Some(renderer), Some(pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) else {
            return;
        };

        let mut batcher = SpriteBatcher::new(1000);

        {
            let mut ctx = RenderContext {
                world: &self.scene.world,
                sprites: &mut batcher,
                camera: &mut self.camera,
                window_size,
            };
            self.game.render(&mut ctx);
        }

        // Collect batches and render
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let textures: HashMap<TextureHandle, TextureResource> = HashMap::new();

        if let Err(e) = renderer.render_with_sprites(pipeline, &self.camera, &textures, &batch_refs) {
            log::error!("Render error: {}", e);
        }
    }
}

impl<G: Game> ApplicationHandler<()> for GameRunner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_attributes = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ))
            .with_resizable(self.config.resizable);

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);

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
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                self.config.width = size.width;
                self.config.height = size.height;
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
                self.game.on_resize(size.width, size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    let window_size = Vec2::new(self.config.width as f32, self.config.height as f32);
                    let mut ctx = GameContext {
                        input: &self.input,
                        world: &mut self.scene.world,
                        delta_time: 0.0,
                        window_size,
                    };

                    match event.state {
                        ElementState::Pressed => {
                            // Handle escape to exit
                            if key == KeyCode::Escape {
                                self.game.on_exit();
                                let _ = self.scene.stop();
                                let _ = self.scene.shutdown();
                                event_loop.exit();
                                return;
                            }
                            self.game.on_key_pressed(key, &mut ctx);
                        }
                        ElementState::Released => {
                            self.game.on_key_released(key, &mut ctx);
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

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
