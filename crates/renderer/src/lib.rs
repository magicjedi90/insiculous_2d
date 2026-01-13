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

// Re-export wgpu for use by dependent crates
pub use wgpu;

// Re-export for convenience
pub use error::*;
pub use renderer::*;
pub use sprite_data::*;
pub use window::*;

// Selective re-exports to avoid conflicts
// TextureHandle is the canonical definition in texture.rs
pub use sprite::{Sprite, SpriteBatch, SpriteBatcher, SpritePipeline, TextureAtlas};
pub use texture::{TextureManager, TextureLoadConfig, SamplerConfig, TextureError, TextureHandle, TextureAtlasBuilder};

// Re-export Time from common crate (moved from renderer for proper placement)
pub use common::Time;

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