//! Convenience re-exports for physics module
//!
//! Import everything commonly needed with:
//! ```rust
//! use physics::prelude::*;
//!
//! let physics_system = PhysicsSystem::new();
//! # let _ = physics_system;
//! ```

pub use crate::components::{
    Collider, ColliderShape, CollisionData, CollisionEvent, ContactPoint, RigidBody,
    RigidBodyType,
};
pub use crate::physics_system::PhysicsSystem;
pub use crate::physics_world::{PhysicsConfig, PhysicsWorld};
