use winit::{
    event_loop::ActiveEventLoop,
    application::ApplicationHandler,
    window::Window,
};
use std::sync::Arc;
use renderer::{Renderer, WindowConfig, create_window_with_active_loop, init};
use crate::{Scene, GameLoop};
use ecs::SystemRegistry;


/// Core application handler for the engine
pub struct EngineApplication {
    /// Renderer created by the application
    pub renderer: Option<Renderer>,
    /// Window created by the application
    pub window: Option<Arc<Window>>,
    /// Window configuration
    pub window_config: WindowConfig,
    /// Stack of scenes (0+ scenes)
    pub scenes: Vec<Scene>,
    /// Schedule for systems
    pub schedule: SystemRegistry,
    /// Game loop
    pub game_loop: GameLoop,
    /// Flag to track whether the renderer needs to be initialized
    needs_renderer_init: bool,
    /// Sprite pipeline for 2D rendering
    pub sprite_pipeline: Option<renderer::sprite::SpritePipeline>,
    /// Default camera for 2D rendering
    pub camera_2d: renderer::Camera2D,
    /// Input handler for managing keyboard, mouse, and gamepad input
    pub input_handler: input::InputHandler,
}

impl EngineApplication {
    /// Create a new engine application with existing scene and game loop
    pub fn new(scene: Scene, game_loop: GameLoop) -> Self {
        Self {
            renderer: None,
            window: None,
            window_config: WindowConfig::default(),
            scenes: vec![scene],
            schedule: SystemRegistry::new(),
            game_loop,
            needs_renderer_init: false,
            sprite_pipeline: None,
            camera_2d: renderer::Camera2D::default(),
            input_handler: input::InputHandler::new(),
        }
    }

    /// Create a new engine application with a scene
    pub fn with_scene(scene: Scene) -> Self {
        Self {
            renderer: None,
            window: None,
            window_config: WindowConfig::default(),
            scenes: vec![scene],
            schedule: SystemRegistry::new(),
            game_loop: GameLoop::new(crate::GameLoopConfig::default()),
            needs_renderer_init: false,
            sprite_pipeline: None,
            camera_2d: renderer::Camera2D::default(),
            input_handler: input::InputHandler::new(),
        }
    }

    /// Push a scene onto the stack
    pub fn push_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }

    /// Pop a scene from the stack
    pub fn pop_scene(&mut self) -> Option<Scene> {
        self.scenes.pop()
    }

    /// Create a new engine application with a custom window configuration
    pub fn with_window_config(mut self, window_config: WindowConfig) -> Self {
        self.window_config = window_config;
        self
    }

    /// Initialize the renderer with the current window
    pub fn init_renderer(&mut self) -> Result<(), renderer::RendererError> {
        if let Some(window) = &self.window {
            // Create a simple async runtime for renderer initialization
            let window_clone = window.clone();
            
            // Use pollster for async execution
            let renderer = pollster::block_on(async {
                init(window_clone).await
            })?;

            // Create sprite pipeline
            let sprite_pipeline = Some(renderer::sprite::SpritePipeline::new(renderer.device(), 1000)); // Max 1000 sprites per batch

            // Store renderer and sprite pipeline
            self.renderer = Some(renderer);
            self.sprite_pipeline = sprite_pipeline;

            // Log initialization
            if let Some(renderer) = &self.renderer {
                log::debug!("Renderer initialized with adapter: {}", renderer.adapter_info());
                log::debug!("Surface format: {:?}", renderer.surface_format());
                log::debug!("Surface size: {}x{}", renderer.surface_width(), renderer.surface_height());
            }

            log::info!("Renderer initialized");
            Ok(())
        } else {
            Err(renderer::RendererError::WindowCreationError("Window not created yet".to_string()))
        }
    }

    /// Process a single frame with proper error handling
    pub fn frame(&mut self, dt: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Update input state for this frame
        self.input_handler.update();

        if let Some(active) = self.scenes.last_mut() {
            // Only update if the scene is operational
            if active.is_operational() {
                active.update_with_schedule(&mut self.schedule, dt)?;
            } else {
                log::warn!("Scene '{}' is not operational (state: {:?}), skipping update", 
                          active.name(), active.lifecycle_state());
            }

            // Update camera viewport size based on window size
            if let Some(renderer) = &self.renderer {
                let width = renderer.surface_width() as f32;
                let height = renderer.surface_height() as f32;
                if height > 0.0 {
                    self.camera_2d.viewport_size = glam::Vec2::new(width, height);
                }
            }

            if let Some(renderer) = &mut self.renderer {
                // Render the scene with sprites if possible
                if let Some(sprite_pipeline) = &mut self.sprite_pipeline {
                    // For now, use empty sprite data for testing
                    let sprite_batches: Vec<&renderer::sprite::SpriteBatch> = vec![];
                    let texture_resources = std::collections::HashMap::new();

                    match renderer.render_with_sprites(sprite_pipeline, &self.camera_2d, &texture_resources, &sprite_batches) {
                        Ok(_) => {
                            log::trace!("Rendering scene with sprites");
                        }
                        Err(renderer::RendererError::SurfaceError(_)) => {
                            // Try to recreate the surface
                            if let Err(e) = renderer.recreate_surface() {
                                log::error!("Failed to recreate surface: {}", e);
                            } else {
                                log::debug!("Surface recreated after loss");
                            }
                        }
                        Err(e) => {
                            log::error!("Render error: {}", e);
                        }
                    }
                } else {
                    // Fallback to basic rendering
                    match renderer.render() {
                        Ok(_) => {
                            log::trace!("Rendering scene");
                        }
                        Err(renderer::RendererError::SurfaceError(_)) => {
                            // Try to recreate the surface
                            if let Err(e) = renderer.recreate_surface() {
                                log::error!("Failed to recreate surface: {}", e);
                            } else {
                                log::debug!("Surface recreated after loss");
                            }
                        }
                        Err(e) => {
                            log::error!("Render error: {}", e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Get a reference to the game loop
    pub fn game_loop(&self) -> &GameLoop {
        &self.game_loop
    }

    /// Get a mutable reference to the game loop
    pub fn game_loop_mut(&mut self) -> &mut GameLoop {
        &mut self.game_loop
    }

    /// Get the window if it exists
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// Get a reference to the active scene (last scene in the stack)
    pub fn active_scene(&self) -> Option<&Scene> {
        self.scenes.last()
    }

    /// Get a mutable reference to the active scene (last scene in the stack)
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.scenes.last_mut()
    }

    /// Get a reference to the input handler
    pub fn input(&self) -> &input::InputHandler {
        &self.input_handler
    }

    /// Get a mutable reference to the input handler
    pub fn input_mut(&mut self) -> &mut input::InputHandler {
        &mut self.input_handler
    }

    /// Create a window using the configured window settings
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), renderer::RendererError> {
        match create_window_with_active_loop(&self.window_config, event_loop) {
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

    /// Start the game loop
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

impl EngineApplication {
    /// Extract sprite data from the current scene for rendering
    /// For now, return empty data since we're testing basic rendering functionality
    #[allow(dead_code)]
    fn extract_sprite_data(&self) -> (Vec<renderer::sprite::SpriteBatch>, std::collections::HashMap<renderer::sprite::TextureHandle, renderer::TextureResource>) {
        // Return empty data for now - the test example will create its own sprites
        (Vec::new(), std::collections::HashMap::new())
    }
}

impl ApplicationHandler<()> for EngineApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application is resumed if it doesn't exist
        if self.window.is_none() {
            if self.create_window(event_loop).is_ok() {
                // Set the flag to indicate that the renderer needs to be initialized
                self.needs_renderer_init = true;
                log::info!("Window created, renderer initialization needed");
            }
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
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            _ => {
                // Forward all other events to input handler
                self.input_handler.handle_window_event(&event);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Initialize the renderer if needed - now synchronous
        if self.needs_renderer_init && self.renderer.is_none() && self.window.is_some() {
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
            // In a production engine, you might want to handle this more gracefully
            // For now, we'll log the error and continue
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