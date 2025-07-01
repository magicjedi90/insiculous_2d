//! Entity management for the ECS.

use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

impl EntityId {
    /// Create a new entity ID
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// An entity in the ECS
#[derive(Debug)]
pub struct Entity {
    /// The unique identifier for this entity
    id: EntityId,
    /// Whether this entity is active
    active: bool,
}

impl Entity {
    /// Create a new entity
    pub fn new() -> Self {
        Self {
            id: EntityId::new(),
            active: true,
        }
    }

    /// Get the entity's ID
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Check if the entity is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Set whether the entity is active
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::new()
    }
}
