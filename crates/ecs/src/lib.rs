//! Simple entity-component-system for the insiculous_2d game engine.
//!
//! This crate provides a minimal ECS implementation.

mod component;
mod entity;
mod system;
mod world;

pub mod prelude;

// Re-export for convenience
pub use component::*;
pub use entity::*;
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
}
