//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

use winit::application::ApplicationHandler;

mod error;
mod renderer;
mod window;

pub mod prelude;

// Re-export for convenience
pub use error::*;
pub use renderer::*;
pub use window::*;

/// Initialize the renderer with an existing window
pub async fn init(window: std::sync::Arc<Window>) -> Result<Renderer<'static>, RendererError> {
    log::info!("Renderer initialized");
    Renderer::new(window).await
}

/// Run the renderer with a custom application handler
pub fn run_with_app<T>(app: &mut T) -> Result<(), RendererError>
where
    T: ApplicationHandler<()> + 'static
{
    log::info!("Running renderer with custom application handler");
    Renderer::run_with_app(app)
}
