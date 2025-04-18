//! Crate root for Insiculous2D Engine.
//!
//! This library provides the high‑level `launch()` entry point and the
//! `GameState` trait implemented by the game.  Most engine users will
//! `use engine_core::prelude::*;` rather than importing items
//! one by one.  See `prelude.rs` for exactly what is re‑exported.

pub mod engine;         // game loop + winit glue
pub mod time;           // ApplicationClock
pub mod events;         // EventBus wrapper
pub mod prelude;
pub mod game_state;
mod state_stack;

// Re‑export the façade so callers can `Engine::launch()` if they prefer
pub use engine::launch;
pub use game_state::GameState;
