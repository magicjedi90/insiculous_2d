//! Editor integration layer for the insiculous_2d game engine.
//!
//! This crate provides `run_game_with_editor()` — a single function call that
//! wraps any `Game` implementation with the full editor UI (menu bar, toolbar,
//! dock panels, hierarchy, inspector, gizmo, tool shortcuts).
//!
//! # Example
//! ```ignore
//! use engine_core::prelude::*;
//! use editor_integration::run_game_with_editor;
//!
//! struct MyGame;
//! impl Game for MyGame {
//!     fn update(&mut self, ctx: &mut GameContext) { /* game logic */ }
//! }
//!
//! fn main() {
//!     run_game_with_editor(MyGame, GameConfig::new("My Game")).unwrap();
//! }
//! ```

mod editor_game;
mod panel_renderer;

pub use editor_game::run_game_with_editor;
