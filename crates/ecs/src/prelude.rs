//! Prelude module for the ecs crate.
//!
//! This module re-exports the most commonly used items from the crate
//! for ergonomic imports.

pub use crate::{
    component::Component,
    entity::{Entity, EntityId},
    event::EventBus,
    hierarchy::{Children, GlobalTransform2D, Parent},
    hierarchy_extension::WorldHierarchyExt,
    hierarchy_system::TransformHierarchySystem,
    init,
    resource::ResourceStorage,
    state_machine::{HierarchicalStateMachine, StateMachine},
    system::{SimpleSystem, System},
    world::World,
    EcsError,
};
