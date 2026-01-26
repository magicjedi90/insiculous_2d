//! Simple entity-component-system for the insiculous_2d game engine.
//!
//! This crate provides a minimal ECS implementation with archetype-based
//! component storage for optimal performance.
//!
//! # Module Visibility Strategy
//!
//! This crate uses two visibility patterns intentionally:
//!
//! - **Private modules** (`mod` + `pub use *`): Core infrastructure types like
//!   [`EntityId`], [`Component`], [`World`], and archetypes. These are re-exported
//!   at the crate root for convenient access while keeping implementation details hidden.
//!
//! - **Public modules** (`pub mod` + `pub use *`): Domain-specific modules like
//!   [`behavior`], [`hierarchy`], [`sprite_components`], etc. These are publicly
//!   visible for documentation discoverability while also re-exported at the crate root.
//!
//! All public types are accessible from the crate root: `use ecs::EntityId;`

// Core infrastructure - private modules, re-exported at crate root
mod archetype;
mod component;
mod entity;
mod world;

// Domain modules - public for documentation, also re-exported at crate root
pub mod audio_components;
pub mod behavior;
pub mod component_registry;
pub mod generation;
pub mod hierarchy;
pub mod hierarchy_ext;
pub mod hierarchy_system;
pub mod sprite_components;
pub mod sprite_system;
pub mod system;

pub mod prelude;

// Re-export all public items at crate root for convenient access
pub use archetype::*;
pub use audio_components::*;
pub use behavior::*;
pub use component::*;
pub use component_registry::{global_registry, ComponentMeta};
pub use entity::*;
pub use generation::*;
pub use hierarchy::*;
pub use hierarchy_ext::*;
pub use hierarchy_system::*;
pub use sprite_components::*;
pub use sprite_system::*;
pub use system::*;
pub use world::*;

/// Initialize the ECS
pub fn init() -> Result<World, EcsError> {
    log::info!("ECS initialized");
    Ok(World::new())
}

/// Errors that can occur in the ECS
#[derive(Debug, thiserror::Error)]
pub enum EcsError {
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),

    #[error("Component not found for entity {0}")]
    ComponentNotFound(EntityId),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("Entity generation error: {0}")]
    GenerationError(#[from] GenerationError),

    #[error("World not initialized")]
    NotInitialized,

    #[error("World already initialized")]
    AlreadyInitialized,

    #[error("World not running")]
    NotRunning,

    #[error("World already running")]
    AlreadyRunning,
}

impl From<String> for EcsError {
    fn from(error: String) -> Self {
        EcsError::SystemError(error)
    }
}