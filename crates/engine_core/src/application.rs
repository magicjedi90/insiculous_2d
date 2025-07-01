use winit::{
    event_loop::ActiveEventLoop,
    application::ApplicationHandler,
    window::Window,
};
use std::sync::Arc;
use crate::{World, GameLoop};

/// Core application handler for the engine
pub struct EngineApplication {
    /// Window created by the application
    pub window: Option<Arc<Window>>,
    /// Game world
    pub world: World,
    /// Game loop
    pub game_loop: GameLoop,
}

impl EngineApplication {
    /// Create a new engine application with existing world and game loop
    pub fn new(world: World, game_loop: GameLoop) -> Self {
        Self {
            window: None,
            world,
            game_loop,
        }
    }

    /// Get a reference to the world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a mutable reference to the world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Get a reference to the game loop
    pub fn game_loop(&self) -> &GameLoop {
        &self.game_loop
    }

    /// Get a mutable reference to the game loop
    pub fn game_loop_mut(&mut self) -> &mut GameLoop {
        &mut self.game_loop
    }
}

impl ApplicationHandler<()> for EngineApplication {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Start the game loop
        if let Err(e) = self.game_loop.start() {
            log::error!("Failed to start game loop: {}", e);
        } else {
            log::info!("Game loop started");
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Handle window events
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.game_loop.stop();
                log::info!("Game loop stopped");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Update the world
        let delta_time = self.game_loop.timer().delta_time().as_secs_f32();
        self.world.update(delta_time);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.game_loop.stop();
        log::info!("Game loop stopped");
        log::info!("Application exiting");
    }
}