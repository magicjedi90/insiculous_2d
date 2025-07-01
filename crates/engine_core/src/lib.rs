//! Core functionality for the insiculous_2d game engine.
//!
//! This crate provides the game loop, timing, and world state management.

mod application;
mod game_loop;
mod timing;
mod world;

pub mod prelude;

// Re-export for convenience
pub use application::*;
pub use game_loop::*;
pub use timing::*;
pub use world::*;

/// Initialize the engine core
pub fn init() -> Result<(), EngineError> {
    log::info!("Engine core initialized");
    Ok(())
}

/// Errors that can occur in the engine core
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to initialize engine: {0}")]
    InitializationError(String),

    #[error("Game loop error: {0}")]
    GameLoopError(String),
}
