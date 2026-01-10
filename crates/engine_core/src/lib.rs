//! Core functionality for the insiculous_2d game engine.
//!
//! This crate provides the game loop, timing, and scene graph management.
//!
//! # Quick Start
//!
//! The easiest way to create a game is using the `Game` trait:
//!
//! ```ignore
//! use engine_core::prelude::*;
//!
//! struct MyGame;
//!
//! impl Game for MyGame {
//!     fn update(&mut self, ctx: &mut GameContext) {
//!         // Game logic here
//!     }
//! }
//!
//! fn main() {
//!     run_game(MyGame, GameConfig::default()).unwrap();
//! }
//! ```

mod application;
pub mod behavior;
mod game;
mod game_loop;
mod timing;
mod scene;
pub mod lifecycle;
pub mod assets;
pub mod scene_data;
pub mod scene_loader;

pub mod prelude;

// Re-export for convenience
pub use application::*;
pub use behavior::*;
pub use game::*;
pub use game_loop::*;
pub use timing::*;
pub use scene::*;
pub use lifecycle::*;
pub use assets::*;
pub use scene_data::*;
pub use scene_loader::*;

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
