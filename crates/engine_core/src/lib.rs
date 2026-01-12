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
pub mod behavior_runner;
mod game;
mod game_loop;
mod timing;
mod scene;
pub mod scene_manager;
pub mod lifecycle;
pub mod assets;
pub mod scene_data;
pub mod scene_loader;
pub mod render_manager;
pub mod window_manager;
pub mod game_loop_manager;
pub mod ui_manager;
pub mod game_config;
pub mod contexts;
pub mod ui_integration;

pub mod prelude;

// Re-export for convenience
pub use application::*;
pub use behavior_runner::*;
pub use game::*;
pub use game_loop::*;
pub use timing::*;
pub use scene::*;
pub use scene_manager::*;
pub use lifecycle::*;
pub use assets::*;
pub use scene_data::*;
pub use scene_loader::*;
pub use render_manager::*;
pub use window_manager::*;
pub use game_loop_manager::*;
pub use ui_manager::*;
pub use game_config::*;

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
