//! Prelude module for the renderer crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    init,
    window::{create_window_with_active_loop, WindowConfig},
    sprite_data::{Camera, SpriteVertex, SpriteInstance, CameraUniform, TextureResource, DynamicBuffer},
    sprite::{Sprite, SpriteBatch, SpriteBatcher, SpritePipeline},
    atlas::TextureAtlas,
    texture::{TextureHandle, TextureManager, TextureLoadConfig, TextureError},
    Time,
    Renderer, RendererConfig, RendererError,
};