//! Prelude module for the engine_core crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    // Simple game API (recommended)
    game::{Game, run_game},
    contexts::{GameContext, RenderContext, GlyphCacheKey},
    game_config::GameConfig,
    chaos_mode::ChaosMode,
    achievements::{Achievement, AchievementManager, AchievementError},
    // Asset management
    assets::{AssetManager, AssetConfig, AssetError},
    // Scene serialization
    scene_data::{SceneData, PhysicsSettings, PrefabData, EntityData, ComponentData, BehaviorData, SceneLoadError},
    scene_loader::{SceneLoader, SceneInstance},
    // Behavior system
    behavior_runner::{BehaviorRunner, EntityCollected},
    // Particle system (CPU pool; spawn bursts or attach a ParticleEmitter)
    particles::{Particle, ParticleConfig, ParticleEmitter, ParticleManager, ParticleSystem},
    // Spring-mass grid (Geometry-Wars style deforming background)
    grid::{GridImpulse, GridMesh},
    // Debug-draw helpers (collider outlines, etc.)
    debug,
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
pub use ecs::sprite_components::{Name, Sprite};
pub use ecs::audio_components::{AudioSource, AudioListener, PlaySoundEffect};
pub use ecs::hierarchy_system::TransformHierarchySystem;
pub use ecs::WorldHierarchyExt;
pub use ecs::System;
pub use ecs::behavior::Behavior;
pub use ecs::{StateMachine, HierarchicalStateMachine, EventBus, ResourceStorage};

// Re-export input types (KeyCode/MouseButton re-exported through input crate, not directly from winit)
pub use input::prelude::{KeyCode, MouseButton};

// Re-export renderer types
pub use renderer::{TextureHandle, TextureLoadConfig};
pub use renderer::line_pipeline::LineVertex;

// Re-export audio types
pub use audio::{AudioManager, SoundHandle, SoundSettings};

// Re-export UI types (UIRect and UIColor are aliases to common types for backwards compatibility)
pub use ui::{UIContext, Theme as UITheme, WidgetId};
pub use common::Rect as UIRect;
pub use common::Color as UIColor;

// Re-export physics types when the physics feature is enabled
#[cfg(feature = "physics")]
pub use physics::{
    Collider, ColliderShape, CollisionData, CollisionEvent, ContactPoint,
    PhysicsConfig, PhysicsSystem, RigidBody, RigidBodyType,
};
