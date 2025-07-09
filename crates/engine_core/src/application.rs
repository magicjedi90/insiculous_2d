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
    pub renderer: Option<Renderer<'static>>,
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
    /// 
    /// This function requires tokio runtime to be available as it uses async/await.
    /// It should be called after the window has been created.
    pub async fn init_renderer(&mut self) -> Result<(), renderer::RendererError> {
        if let Some(window) = &self.window {
            // Initialize the renderer with the window
            let renderer = init(window.clone()).await?;
            self.renderer = Some(renderer);
            log::info!("Renderer initialized");
            Ok(())
        } else {
            Err(renderer::RendererError::WindowCreationError("Window not created yet".to_string()))
        }
    }

    /// Process a single frame
    pub fn frame(&mut self, dt: f32) {
        if let Some(active) = self.scenes.last_mut() {
            active.update_with_schedule(&mut self.schedule, dt);
            if let Some(renderer) = &mut self.renderer {

                // Render the scene
                // Note: This is a placeholder as the actual rendering implementation
                // might differ based on the renderer's API
                renderer.render().unwrap();
                log::trace!("Rendering scene");
            }
        }
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

    /// Create a window using the configured window settings
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), renderer::RendererError> {
        match create_window_with_active_loop(&self.window_config, event_loop) {
            Ok(window) => {
                self.window = Some(Arc::new(window));
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

impl ApplicationHandler<()> for EngineApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application is resumed if it doesn't exist
        if self.window.is_none() {
            if let Ok(_) = self.create_window(event_loop) {
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
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Handle window events
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.game_loop.stop();
                log::info!("Game loop stopped");
                // Request exit from the event loop
                event_loop.exit();
                log::info!("Requested exit from event loop");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Initialize the renderer if needed
        if self.needs_renderer_init && self.renderer.is_none() && self.window.is_some() {
            // Use the current tokio runtime handle to run the async init_renderer call
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    // Use block_in_place to allow blocking within an async context
                    tokio::task::block_in_place(|| {
                        if let Err(e) = handle.block_on(self.init_renderer()) {
                            log::error!("Failed to initialize renderer: {}", e);
                        } else {
                            log::info!("Renderer initialized successfully");
                            self.needs_renderer_init = false;
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to get current tokio runtime handle: {}", e);
                }
            }
        }

        // Update the active scene
        let delta_time = self.game_loop.timer().delta_time().as_secs_f32();
        self.frame(delta_time);

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
