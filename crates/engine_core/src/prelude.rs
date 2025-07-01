//! Prelude module for the engine_core crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    application::EngineApplication,
    game_loop::{GameLoop, GameLoopConfig},
    init,
    timing::Timer,
    scene::Scene,
    EngineError,
};
