//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    window::Window,
};

mod error;
mod renderer;
pub mod sprite;
pub mod sprite_data;
pub mod texture;
mod window;

pub mod prelude;

// Re-export for convenience
pub use error::*;
pub use renderer::*;
pub use sprite_data::*;
pub use window::*;

// Selective re-exports to avoid conflicts
pub use sprite::{Sprite, SpriteBatch, SpriteBatcher, SpritePipeline, TextureAtlas};
pub use texture::{TextureManager, TextureLoadConfig, SamplerConfig, TextureError, TextureHandle, TextureAtlasBuilder};

/// Time resource for tracking delta time
#[derive(Debug, Clone, Copy)]
pub struct Time {
    /// Delta time in seconds
    pub delta_seconds: f32,
    /// Total elapsed time in seconds
    pub elapsed_seconds: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
        }
    }
}

/// Simplified renderer initialization - no tokio required
pub async fn init(window: Arc<Window>) -> Result<Renderer, RendererError> {
    log::info!("Initializing renderer");
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