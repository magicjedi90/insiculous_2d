//! Prelude module for the engine_core crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    // Simple game API (recommended)
    game::{Game, run_game},
    contexts::{GameContext, RenderContext, GlyphCacheKey},
    game_config::GameConfig,
    // Asset management
    assets::{AssetManager, AssetConfig, AssetError},
    // Scene serialization
    scene_data::{SceneData, PhysicsSettings, PrefabData, EntityData, ComponentData, BehaviorData, SceneLoadError},
    scene_loader::{SceneLoader, SceneInstance},
    // Behavior system
    behavior_runner::BehaviorRunner,
    // Game loop
    game_loop::{GameLoop, GameLoopConfig},
    init,
    timing::Timer,
    scene::Scene,
    EngineError,
};

// Re-export common types (Color, Transform2D, Camera2D, Rect)
pub use common::{Color, Transform2D, Camera, Rect};

// Re-export commonly used types from dependencies
pub use glam::{Vec2, Vec4};

// Re-export ECS types
pub use ecs::{EntityId, World};
pub use ecs::sprite_components::Sprite;
pub use ecs::audio_components::{AudioSource, AudioListener, PlaySoundEffect};

// Re-export input types (KeyCode re-exported through input crate, not directly from winit)
pub use input::prelude::KeyCode;

// Re-export renderer types
pub use renderer::{TextureHandle, TextureLoadConfig};

// Re-export audio types
pub use audio::{AudioManager, SoundHandle, SoundSettings, PlaybackState};

// Re-export UI types (UIRect and UIColor are aliases to common types for backwards compatibility)
pub use ui::{UIContext, Theme as UITheme, WidgetId};
pub use common::Rect as UIRect;
pub use common::Color as UIColor;

// Re-export physics types when the physics feature is enabled
#[cfg(feature = "physics")]
pub use physics::{
    Collider, ColliderShape, CollisionData, CollisionEvent, ContactPoint,
    MovementConfig, PhysicsConfig, PhysicsSystem, RigidBody, RigidBodyType,
};
