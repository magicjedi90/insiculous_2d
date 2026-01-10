//! Prelude module for the engine_core crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    // Simple game API (recommended)
    game::{Game, GameConfig, GameContext, RenderContext, run_game},
    // Asset management
    assets::{AssetManager, AssetConfig, AssetError},
    // Advanced API
    application::EngineApplication,
    game_loop::{GameLoop, GameLoopConfig},
    init,
    timing::Timer,
    scene::Scene,
    EngineError,
};

// Re-export commonly used types from dependencies
pub use glam::{Vec2, Vec4};
pub use winit::keyboard::KeyCode;
pub use ecs::{EntityId, World};
pub use ecs::sprite_components::{Sprite, Transform2D};
pub use renderer::{TextureHandle, TextureLoadConfig};
