//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

mod error;
mod renderer;
mod window;

pub mod prelude;

// Re-export for convenience
pub use error::*;
pub use renderer::*;
pub use window::*;

/// Initialize the renderer
pub async fn init() -> Result<Renderer<'static>, RendererError> {
    log::info!("Renderer initialized");
    Renderer::new().await
}