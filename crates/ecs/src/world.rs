//! World management for the ECS.

use std::collections::HashMap;

use crate::component::{Component, ComponentRegistry};
use crate::entity::{Entity, EntityId};
use crate::system::SystemRegistry;
use crate::EcsError;

/// The main world struct for the ECS
pub struct World {
    /// The entities in the world
    entities: HashMap<EntityId, Entity>,
    /// The component registry
    components: ComponentRegistry,
    /// The system registry
    systems: SystemRegistry,
}

impl World {
    /// Create a new world
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            components: ComponentRegistry::new(),
            systems: SystemRegistry::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> EntityId {
        let entity = Entity::new();
        let id = entity.id();
        self.entities.insert(id, entity);
        id
    }

    /// Remove an entity
    pub fn remove_entity(&mut self, entity_id: &EntityId) -> Result<(), EcsError> {
        if self.entities.remove(entity_id).is_none() {
            return Err(EcsError::EntityNotFound(*entity_id));
        }
        Ok(())
    }

    /// Get a reference to an entity
    pub fn get_entity(&self, entity_id: &EntityId) -> Result<&Entity, EcsError> {
        self.entities
            .get(entity_id)
            .ok_or(EcsError::EntityNotFound(*entity_id))
    }

    /// Get a mutable reference to an entity
    pub fn get_entity_mut(&mut self, entity_id: &EntityId) -> Result<&mut Entity, EcsError> {
        self.entities
            .get_mut(entity_id)
            .ok_or(EcsError::EntityNotFound(*entity_id))
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(
        &mut self,
        entity_id: &EntityId,
        component: T,
    ) -> Result<(), EcsError> {
        if !self.entities.contains_key(entity_id) {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        self.components.add(*entity_id, component);
        Ok(())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity_id: &EntityId) -> Result<(), EcsError> {
        if !self.entities.contains_key(entity_id) {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        if self.components.remove::<T>(entity_id).is_none() {
            return Err(EcsError::ComponentNotFound(*entity_id));
        }

        Ok(())
    }

    /// Get a reference to a component for an entity
    pub fn get_component<T: Component>(
        &self,
        entity_id: &EntityId,
    ) -> Result<&dyn Component, EcsError> {
        if !self.entities.contains_key(entity_id) {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        self.components
            .get::<T>(entity_id)
            .ok_or(EcsError::ComponentNotFound(*entity_id))
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_component_mut<T: Component>(
        &mut self,
        entity_id: &EntityId,
    ) -> Result<&mut dyn Component, EcsError> {
        if !self.entities.contains_key(entity_id) {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        self.components
            .get_mut::<T>(entity_id)
            .ok_or(EcsError::ComponentNotFound(*entity_id))
    }

    /// Check if an entity has a component
    pub fn has_component<T: Component>(&self, entity_id: &EntityId) -> Result<bool, EcsError> {
        if !self.entities.contains_key(entity_id) {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        Ok(self.components.has::<T>(entity_id))
    }

    /// Add a system
    pub fn add_system<S: crate::system::System + 'static>(&mut self, system: S) {
        self.systems.add(system);
    }

    /// Update all systems - safe version without borrowing issues
    pub fn update(&mut self, delta_time: f32) {
        // Extract systems into a temporary variable to avoid borrowing conflicts
        let mut temp_systems = SystemRegistry::new();
        std::mem::swap(&mut self.systems, &mut temp_systems);
        
        // Update systems with the world
        temp_systems.update_all(self, delta_time);
        
        // Move systems back
        std::mem::swap(&mut self.systems, &mut temp_systems);
    }

    /// Get the number of entities
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the number of systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}