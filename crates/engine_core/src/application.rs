//! Engine application handler using winit.
//!
//! This module provides the `EngineApplication` struct which implements
//! the winit `ApplicationHandler` trait for running games.

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event_loop::ActiveEventLoop,
    window::Window,
};

use renderer::WindowConfig;

use crate::render_manager::RenderManager;
use crate::{GameLoop, Scene};
use ecs::SystemRegistry;

/// Core application handler for the engine.
///
/// This struct orchestrates the game loop, delegating specific responsibilities
/// to focused managers:
/// - `RenderManager`: Renderer lifecycle and sprite rendering
/// - Scene stack: Multiple game scenes with lifecycle management
/// - Input handling: Keyboard, mouse, and gamepad events
pub struct EngineApplication {
    /// Window reference for event handling
    window: Option<Arc<Window>>,
    /// Window configuration
    window_config: WindowConfig,
    /// Render management (renderer, sprite pipeline, camera)
    render_manager: RenderManager,
    /// Stack of scenes (0+ scenes)
    pub scenes: Vec<Scene>,
    /// Schedule for systems
    pub schedule: SystemRegistry,
    /// Game loop
    pub game_loop: GameLoop,
    /// Flag to track whether the renderer needs to be initialized
    needs_renderer_init: bool,
    /// Input handler for managing keyboard, mouse, and gamepad input
    pub input_handler: input::InputHandler,
}

impl EngineApplication {
    /// Create a new engine application with existing scene and game loop.
    pub fn new(scene: Scene, game_loop: GameLoop) -> Self {
        Self {
            window: None,
            window_config: WindowConfig::default(),
            render_manager: RenderManager::new(),
            scenes: vec![scene],
            schedule: SystemRegistry::new(),
            game_loop,
            needs_renderer_init: false,
            input_handler: input::InputHandler::new(),
        }
    }

    /// Create a new engine application with a scene.
    pub fn with_scene(scene: Scene) -> Self {
        Self {
            window: None,
            window_config: WindowConfig::default(),
            render_manager: RenderManager::new(),
            scenes: vec![scene],
            schedule: SystemRegistry::new(),
            game_loop: GameLoop::new(crate::GameLoopConfig::default()),
            needs_renderer_init: false,
            input_handler: input::InputHandler::new(),
        }
    }

    /// Push a scene onto the stack.
    pub fn push_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }

    /// Pop a scene from the stack.
    pub fn pop_scene(&mut self) -> Option<Scene> {
        self.scenes.pop()
    }

    /// Create a new engine application with a custom window configuration.
    pub fn with_window_config(mut self, window_config: WindowConfig) -> Self {
        self.window_config = window_config;
        self
    }

    /// Initialize the renderer with the current window.
    pub fn init_renderer(&mut self) -> Result<(), renderer::RendererError> {
        let window = self.window.clone().ok_or_else(|| {
            renderer::RendererError::WindowCreationError("Window not created yet".to_string())
        })?;

        // Default clear color
        let clear_color = [0.1, 0.1, 0.15, 1.0];
        self.render_manager.init(window, clear_color)?;

        // Log initialization
        if let Some(info) = self.render_manager.adapter_info() {
            log::debug!("Renderer initialized with adapter: {}", info);
        }
        if let (Some(w), Some(h)) = (self.render_manager.surface_width(), self.render_manager.surface_height()) {
            log::debug!("Surface size: {}x{}", w, h);
        }

        log::info!("Renderer initialized");
        Ok(())
    }

    /// Process a single frame with proper error handling.
    pub fn frame(&mut self, dt: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Update input state for this frame
        self.input_handler.update();

        if let Some(active) = self.scenes.last_mut() {
            // Only update if the scene is operational
            if active.is_operational() {
                active.update_with_schedule(&mut self.schedule, dt)?;
            } else {
                log::warn!(
                    "Scene '{}' is not operational (state: {:?}), skipping update",
                    active.name(),
                    active.lifecycle_state()
                );
            }

            // Update camera viewport size based on renderer surface
            self.render_manager.update_viewport_from_renderer();

            // Render the scene
            if self.render_manager.is_initialized() {
                // For now, use empty sprite data (scene-based rendering doesn't extract from ECS)
                let empty_textures = std::collections::HashMap::new();
                if let Err(e) = self.render_manager.render(&[], &empty_textures) {
                    log::error!("Render error: {}", e);
                } else {
                    log::trace!("Rendering scene with sprites");
                }
            }
        }

        Ok(())
    }

    /// Get a reference to the game loop.
    pub fn game_loop(&self) -> &GameLoop {
        &self.game_loop
    }

    /// Get a mutable reference to the game loop.
    pub fn game_loop_mut(&mut self) -> &mut GameLoop {
        &mut self.game_loop
    }

    /// Get the window if it exists.
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// Get a reference to the active scene (last scene in the stack).
    pub fn active_scene(&self) -> Option<&Scene> {
        self.scenes.last()
    }

    /// Get a mutable reference to the active scene (last scene in the stack).
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.scenes.last_mut()
    }

    /// Get a reference to the input handler.
    pub fn input(&self) -> &input::InputHandler {
        &self.input_handler
    }

    /// Get a mutable reference to the input handler.
    pub fn input_mut(&mut self) -> &mut input::InputHandler {
        &mut self.input_handler
    }

    /// Get a reference to the render manager.
    pub fn render_manager(&self) -> &RenderManager {
        &self.render_manager
    }

    /// Get a mutable reference to the render manager.
    pub fn render_manager_mut(&mut self) -> &mut RenderManager {
        &mut self.render_manager
    }

    /// Get a reference to the 2D camera.
    pub fn camera_2d(&self) -> &renderer::Camera2D {
        self.render_manager.camera()
    }

    /// Get a mutable reference to the 2D camera.
    pub fn camera_2d_mut(&mut self) -> &mut renderer::Camera2D {
        self.render_manager.camera_mut()
    }

    /// Create a window using the configured window settings.
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), renderer::RendererError> {
        match renderer::create_window_with_active_loop(&self.window_config, event_loop) {
            Ok(window) => {
                self.window = Some(window);
                log::info!("Window created");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to create window: {}", e);
                Err(e)
            }
        }
    }

    /// Start the game loop.
    pub fn start_game_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = self.game_loop.start() {
            log::error!("Failed to start game loop: {}", e);
            Err(Box::new(e))
        } else {
            log::info!("Game loop started");
            Ok(())
        }
    }
}

impl ApplicationHandler<()> for EngineApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application is resumed if it doesn't exist
        if self.window.is_none() && self.create_window(event_loop).is_ok() {
            // Set the flag to indicate that the renderer needs to be initialized
            self.needs_renderer_init = true;
            log::info!("Window created, renderer initialization needed");
        }

        // Start the game loop if it's not already running
        if !self.game_loop.is_running() {
            let _ = self.start_game_loop();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Only handle events for our window
        if let Some(window) = &self.window {
            if window_id != window.id() {
                return;
            }
        }

        // Handle window events and forward input events to input handler
        match &event {
            winit::event::WindowEvent::CloseRequested => {
                self.game_loop.stop();
                log::info!("Game loop stopped");
                // Request exit from the event loop
                event_loop.exit();
                log::info!("Requested exit from event loop");
            }
            winit::event::WindowEvent::Resized(size) => {
                log::debug!("Window resized to {}x{}", size.width, size.height);
                self.render_manager.resize(size.width, size.height);
            }
            _ => {
                // Forward all other events to input handler
                self.input_handler.handle_window_event(&event);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Initialize the renderer if needed
        if self.needs_renderer_init && !self.render_manager.is_initialized() && self.window.is_some() {
            match self.init_renderer() {
                Err(e) => {
                    log::error!("Failed to initialize renderer: {}", e);
                }
                Ok(_) => {
                    log::info!("Renderer initialized successfully");
                    self.needs_renderer_init = false;
                }
            }
        }

        // Update the active scene
        let delta_time = self.game_loop.timer().delta_time().as_secs_f32();
        if let Err(e) = self.frame(delta_time) {
            log::error!("Frame processing error: {}", e);
        }

        // Request a redraw after each frame update
        if let Some(window) = &self.window {
            window.request_redraw();
            log::trace!("Requested redraw");
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.game_loop.stop();
        log::info!("Game loop stopped");
        log::info!("Application exiting");
    }
}
