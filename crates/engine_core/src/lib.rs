//! Core functionality for the insiculous_2d game engine.
//!
//! This crate provides the game loop, timing, and scene graph management.
//!
//! # Quick Start
//!
//! The easiest way to create a game is using the `Game` trait:
//!
//! ```no_run
//! use engine_core::prelude::*;
//!
//! struct MyGame;
//!
//! impl Game for MyGame {
//!     fn update(&mut self, _ctx: &mut GameContext) {
//!         // Game logic here
//!     }
//! }
//!
//! fn main() {
//!     run_game(MyGame, GameConfig::default()).unwrap();
//! }
//! ```

pub mod behavior_runner;
mod game;
mod glyph_texture_cache;
mod timing;
mod scene;
pub mod scene_manager;
pub mod lifecycle;
pub mod assets;
pub mod chaos_theme;
pub mod behavior_data;
pub mod scene_data;
pub mod scene_loader;
pub mod scene_serializer;
mod texture_ref;
pub mod render_manager;
pub mod window_manager;
pub mod game_loop_manager;
pub mod ui_manager;
pub mod game_config;
pub mod contexts;
pub mod ui_integration;
pub mod chaos_mode;
pub mod achievements;
#[cfg(feature = "physics")]
pub mod pickups;
pub mod particles;
pub mod grid;
pub mod debug;

pub mod prelude;

// Re-export the public API surface explicitly (no globs) so the crate's
// top-level names are visible at a glance.
pub use behavior_runner::{BehaviorRunner, EntityCollected};
pub use game::{run_game, Game};
pub use timing::Timer;
pub use scene::Scene;
pub use scene_manager::SceneManager;
pub use lifecycle::{Lifecycle, LifecycleManager, LifecycleState};
pub use assets::{AssetConfig, AssetError, AssetManager};
pub use scene_data::{
    BehaviorData, ColliderShapeData, ComponentData, EditorSettings, EntityData, PhysicsSettings,
    PrefabData, RigidBodyTypeData, SceneData, SceneLoadError,
};
pub use chaos_theme::ChaosTheme;
pub use scene_loader::{SceneInstance, SceneLoader};
pub use texture_ref::TextureResolver;

/// The game's root directory for asset/save anchoring (exe dir when shipped
/// with an `assets/` folder beside it, the game crate's directory under
/// `cargo run`). Expands `CARGO_MANIFEST_DIR` at the CALL SITE, so it must
/// be a macro — a plain engine function would bake in the engine's own path.
#[macro_export]
macro_rules! game_root {
    () => {
        $crate::assets::game_root_from(env!("CARGO_MANIFEST_DIR"))
    };
}
pub use scene_serializer::{save_scene_to_file, serialize_to_ron, world_to_scene_data};
pub use render_manager::RenderManager;
pub use window_manager::{WindowConfig, WindowManager};
pub use game_loop_manager::{GameLoopManager, MAX_DELTA_TIME};
pub use ui_manager::UIManager;
pub use game_config::GameConfig;
pub use chaos_mode::ChaosMode;
pub use achievements::{
    Achievement, AchievementError, AchievementManager, ToastStyle, DEFAULT_TOAST_DURATION,
};

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
