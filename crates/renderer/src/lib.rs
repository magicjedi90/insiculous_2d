//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

use winit::application::ApplicationHandler;

mod application;
mod error;
mod renderer;
mod window;

pub mod prelude;

// Re-export for convenience
pub use application::*;
pub use error::*;
pub use renderer::*;
pub use window::*;

/// Initialize the renderer
pub async fn init() -> Result<Renderer<'static>, RendererError> {
    log::info!("Renderer initialized");
    Renderer::new().await
}

/// Initialize the renderer with existing world and game loop
pub async fn init_with_engine_app(engine_app: engine_core::EngineApplication) -> Result<Renderer<'static>, RendererError> {
    log::info!("Renderer initialized with existing world and game loop");
    Renderer::new_with_engine_app(engine_app).await
}

/// Run the renderer with a custom application handler
pub fn run_with_app<T>(app: &mut T) -> Result<(), RendererError>
where
    T: ApplicationHandler<()> + 'static
{
    log::info!("Running renderer with custom application handler");
    Renderer::run_with_app(app)
}
