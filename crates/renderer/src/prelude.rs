//! Prelude module for the renderer crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    init, run_with_app,
    window::{create_window_with_active_loop, WindowConfig},
    sprite::{Camera2D, SpriteBatch, SpritePipeline},
    Time,
    Renderer, RendererError,
};