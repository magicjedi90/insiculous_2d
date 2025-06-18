//! Insiculous2D Engine Core
//!
//! This crate provides the core engine functionality,
//! including the game loop, event handling, and state management.

pub mod engine;
pub mod time;
pub mod events;
pub mod prelude;
pub mod game_state;

mod state_stack;
mod game_object;
mod scene;
pub use engine::launch;
pub use game_state::GameState;
