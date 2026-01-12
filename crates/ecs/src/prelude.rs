//! Prelude module for the ecs crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    component::Component,
    entity::{Entity, EntityId},
    hierarchy::{Children, GlobalTransform2D, Parent},
    hierarchy_system::TransformHierarchySystem,
    init,
    system::{SimpleSystem, System},
    world::World,
    EcsError,
};
