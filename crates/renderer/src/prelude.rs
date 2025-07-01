//! Prelude module for the renderer crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    init,
    window::{create_window, create_window_with_active_loop, WindowConfig},
    Renderer, RendererError,
};
