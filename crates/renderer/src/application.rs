//! Application handler for the renderer.
//!
//! This module provides an application handler that integrates with the game engine.

use crate::window::{create_window_with_active_loop, WindowConfig};
use engine_core::EngineApplication;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::ActiveEventLoop,
    window::Window,
};

/// Renderer-specific application handler
pub struct RendererApplication {
    /// Window configuration
    pub window_config: WindowConfig,
    /// The core engine application
    pub engine_app: EngineApplication,
}

impl RendererApplication {
    /// Create a new renderer application
    pub fn new(window_config: WindowConfig, engine_app: EngineApplication) -> Self {
        Self {
            window_config,
            engine_app,
        }
    }
    
    /// Get the window if it exists
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.engine_app.window.as_ref()
    }
}

impl ApplicationHandler<()> for RendererApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application is resumed
        match create_window_with_active_loop(&self.window_config, event_loop) {
            Ok(window) => {
                self.engine_app.window = Some(Arc::new(window));
                // Delegate to engine application
                self.engine_app.resumed(event_loop);
            }
            Err(e) => {
                log::error!("Failed to create window: {}", e);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Delegate to engine application
        self.engine_app.window_event(event_loop, window_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Delegate to engine application
        self.engine_app.about_to_wait(event_loop);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        // Delegate to engine application
        self.engine_app.exiting(event_loop);
    }
}
