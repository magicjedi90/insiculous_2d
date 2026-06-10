//! WGPU renderer for the insiculous_2d game engine.
//!
//! This crate provides a simple renderer using WGPU.

use std::sync::Arc;
use winit::window::Window;

pub mod atlas;
pub mod bloom;
mod error;
pub mod line_pipeline;
pub mod render_targets;
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
pub use atlas::{AtlasRegion, TextureAtlas, TextureAtlasBuilder};
pub use sprite::{Sprite, SpriteBatch, SpriteBatcher, SpritePipeline};
pub use texture::{TextureManager, TextureLoadConfig, SamplerConfig, TextureError, TextureHandle};

// Re-export Time from common crate (moved from renderer for proper placement)
pub use common::Time;

/// Simplified renderer initialization - no tokio required
pub async fn init(window: Arc<Window>) -> Result<Renderer, RendererError> {
    init_with_config(window, RendererConfig::default()).await
}

/// Renderer initialization with explicit configuration (vsync, etc.)
pub async fn init_with_config(window: Arc<Window>, config: RendererConfig) -> Result<Renderer, RendererError> {
    log::info!("Initializing renderer (vsync: {})", config.vsync);
    Renderer::with_config(window, config).await
}