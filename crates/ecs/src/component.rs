//! Component management for the ECS.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::entity::EntityId;

/// A trait for components in the ECS
pub trait Component: Any + Send + Sync {
    /// Get the type name of the component
    fn type_name(&self) -> &'static str;
}

impl<T: Any + Send + Sync> Component for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// A storage for components of a specific type
#[derive(Default)]
pub struct ComponentStorage {
    /// The components, indexed by entity ID
    components: HashMap<EntityId, Box<dyn Component>>,
}

impl ComponentStorage {
    /// Create a new component storage
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Add a component for an entity
    pub fn add<T: Component>(&mut self, entity_id: EntityId, component: T) {
        self.components.insert(entity_id, Box::new(component));
    }

    /// Remove a component for an entity
    pub fn remove(&mut self, entity_id: &EntityId) -> Option<Box<dyn Component>> {
        self.components.remove(entity_id)
    }

    /// Get a reference to a component for an entity
    pub fn get(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        self.components.get(entity_id).map(|c| c.as_ref())
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_mut(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        self.components.get_mut(entity_id).map(|c| c.as_mut())
    }

    /// Check if an entity has a component
    pub fn has(&self, entity_id: &EntityId) -> bool {
        self.components.contains_key(entity_id)
    }

    /// Get the number of components
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if there are no components
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// A registry for all component types
#[derive(Default)]
pub struct ComponentRegistry {
    /// The component storages, indexed by component type ID
    storages: HashMap<TypeId, ComponentStorage>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Register a component type
    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.storages.entry(type_id).or_default();
    }

    /// Add a component for an entity
    pub fn add<T: Component>(&mut self, entity_id: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.storages.contains_key(&type_id) {
            self.register::<T>();
        }

        if let Some(storage) = self.storages.get_mut(&type_id) {
            storage.add(entity_id, component);
        }
    }

    /// Remove a component for an entity
    pub fn remove<T: Component>(&mut self, entity_id: &EntityId) -> Option<Box<dyn Component>> {
        let type_id = TypeId::of::<T>();
        self.storages.get_mut(&type_id)?.remove(entity_id)
    }

    /// Get a reference to a component for an entity
    pub fn get<T: Component>(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id)?.get(entity_id)
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        let type_id = TypeId::of::<T>();
        self.storages.get_mut(&type_id)?.get_mut(entity_id)
    }

    /// Check if an entity has a component
    pub fn has<T: Component>(&self, entity_id: &EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id).is_some_and(|s| s.has(entity_id))
    }

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity_id: &EntityId) {
        for storage in self.storages.values_mut() {
            storage.remove(entity_id);
        }
    }

    /// Initialize the component registry
    pub fn initialize(&mut self) -> Result<(), String> {
        log::debug!("Initializing component registry with {} component types", self.storages.len());
        Ok(())
    }

    /// Shutdown the component registry
    pub fn shutdown(&mut self) -> Result<(), String> {
        log::debug!("Shutting down component registry, removing all components");
        self.storages.clear();
        Ok(())
    }
}
