//! Typed global resources for cross-system game state.
//!
//! Resources are singleton values stored in the World, accessed by type.
//! Use them for game-wide state like score, settings, or repositories
//! that multiple systems need to read/write.
//!
//! # Example
//! ```ignore
//! use ecs::World;
//!
//! struct Score { value: u32 }
//!
//! let mut world = World::new();
//! world.insert_resource(Score { value: 0 });
//!
//! // Read
//! let score = world.resource::<Score>().unwrap();
//! assert_eq!(score.value, 0);
//!
//! // Write
//! world.resource_mut::<Score>().unwrap().value += 10;
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Storage for typed singleton resources.
///
/// Each resource type can have at most one instance. Resources are
/// accessed by their concrete type via `TypeId`.
pub struct ResourceStorage {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceStorage {
    /// Create an empty resource storage.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Insert a resource, replacing any previous value of the same type.
    /// Returns the previous value if one existed.
    pub fn insert<T: Send + Sync + 'static>(&mut self, resource: T) -> Option<T> {
        let previous = self.resources.insert(
            TypeId::of::<T>(),
            Box::new(resource),
        );
        previous.and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Get an immutable reference to a resource by type.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a resource by type.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a resource by type, returning it if it existed.
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Check if a resource of the given type exists.
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// Remove all resources.
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    /// Get the number of stored resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if storage is empty.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for ResourceStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Score {
        value: u32,
    }

    #[derive(Debug, PartialEq)]
    struct Lives {
        count: i32,
    }

    #[derive(Debug, PartialEq)]
    struct GameSettings {
        difficulty: String,
        volume: f32,
    }

    #[test]
    fn test_insert_and_get_resource() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 100 });

        let score = storage.get::<Score>().unwrap();
        assert_eq!(score.value, 100);
    }

    #[test]
    fn test_get_mut_resource() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 0 });

        storage.get_mut::<Score>().unwrap().value += 50;

        let score = storage.get::<Score>().unwrap();
        assert_eq!(score.value, 50);
    }

    #[test]
    fn test_insert_replaces_previous() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 10 });

        let previous = storage.insert(Score { value: 20 });
        assert_eq!(previous, Some(Score { value: 10 }));

        let score = storage.get::<Score>().unwrap();
        assert_eq!(score.value, 20);
    }

    #[test]
    fn test_remove_resource() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 42 });

        let removed = storage.remove::<Score>();
        assert_eq!(removed, Some(Score { value: 42 }));
        assert!(storage.get::<Score>().is_none());
    }

    #[test]
    fn test_remove_nonexistent_returns_none() {
        let mut storage = ResourceStorage::new();
        assert!(storage.remove::<Score>().is_none());
    }

    #[test]
    fn test_contains_resource() {
        let mut storage = ResourceStorage::new();
        assert!(!storage.contains::<Score>());

        storage.insert(Score { value: 0 });
        assert!(storage.contains::<Score>());
    }

    #[test]
    fn test_multiple_resource_types() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 100 });
        storage.insert(Lives { count: 3 });
        storage.insert(GameSettings {
            difficulty: "hard".to_string(),
            volume: 0.8,
        });

        assert_eq!(storage.get::<Score>().unwrap().value, 100);
        assert_eq!(storage.get::<Lives>().unwrap().count, 3);
        assert_eq!(storage.get::<GameSettings>().unwrap().volume, 0.8);
        assert_eq!(storage.len(), 3);
    }

    #[test]
    fn test_get_nonexistent_returns_none() {
        let storage = ResourceStorage::new();
        assert!(storage.get::<Score>().is_none());
    }

    #[test]
    fn test_clear_resources() {
        let mut storage = ResourceStorage::new();
        storage.insert(Score { value: 1 });
        storage.insert(Lives { count: 5 });

        storage.clear();
        assert!(storage.is_empty());
        assert!(storage.get::<Score>().is_none());
        assert!(storage.get::<Lives>().is_none());
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut storage = ResourceStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);

        storage.insert(Score { value: 0 });
        assert!(!storage.is_empty());
        assert_eq!(storage.len(), 1);
    }
}
