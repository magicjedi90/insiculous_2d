//! Convenience re-exports for physics module
//!
//! Import everything commonly needed with:
//! ```rust,ignore
//! use physics::prelude::*;
//! ```

pub use crate::components::{
    Collider, ColliderShape, CollisionData, CollisionEvent, ContactPoint, RigidBody,
    RigidBodyType,
};
pub use crate::presets::MovementConfig;
pub use crate::system::PhysicsSystem;
pub use crate::world::{PhysicsConfig, PhysicsWorld};
pub use crate::PhysicsError;
