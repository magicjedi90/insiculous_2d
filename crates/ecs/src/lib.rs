//! Simple entity-component-system for the insiculous_2d game engine.
//!
//! This crate provides a minimal ECS implementation with archetype-based
//! component storage for optimal performance.

mod archetype;
mod component;
mod entity;
pub mod generation;
pub mod hierarchy;
pub mod hierarchy_system;
pub mod system;
mod world;
pub mod sprite_components;
pub mod sprite_system;

pub mod prelude;

// Re-export for convenience
pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use generation::*;
pub use hierarchy::*;
pub use hierarchy_system::*;
pub use system::*;
pub use world::*;
pub use sprite_components::*;
pub use sprite_system::*;

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