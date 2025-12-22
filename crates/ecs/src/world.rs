//! World management for the ECS.

use std::collections::HashMap;

use crate::component::{Component, ComponentRegistry};
use crate::entity::{Entity, EntityId};
use crate::generation::EntityGeneration;
use crate::system::SystemRegistry;
use crate::EcsError;

/// The main world struct for the ECS
pub struct World {
    /// The entities in the world
    entities: HashMap<EntityId, Entity>,
    /// Entity generation tracking for detecting stale references
    entity_generations: HashMap<EntityId, EntityGeneration>,
    /// The component registry
    components: ComponentRegistry,
    /// The system registry
    systems: SystemRegistry,
    /// Whether the world is initialized
    initialized: bool,
    /// Whether the world is running
    running: bool,
}

impl World {
    /// Create a new world
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            entity_generations: HashMap::new(),
            components: ComponentRegistry::new(),
            systems: SystemRegistry::new(),
            initialized: false,
            running: false,
        }
    }

    /// Initialize the world
    pub fn initialize(&mut self) -> Result<(), EcsError> {
        if self.initialized {
            return Err(EcsError::AlreadyInitialized);
        }

        // Initialize system registry
        self.systems.initialize()?;

        self.initialized = true;
        log::info!("World initialized with {} entities and {} systems", 
                  self.entities.len(), self.systems.len());
        Ok(())
    }

    /// Start the world (begin running systems)
    pub fn start(&mut self) -> Result<(), EcsError> {
        if !self.initialized {
            return Err(EcsError::NotInitialized);
        }
        if self.running {
            return Err(EcsError::AlreadyRunning);
        }

        self.running = true;
        self.systems.start()?;
        log::info!("World started with {} active systems", self.systems.len());
        Ok(())
    }

    /// Stop the world (pause systems)
    pub fn stop(&mut self) -> Result<(), EcsError> {
        if !self.running {
            return Err(EcsError::NotRunning);
        }

        self.running = false;
        self.systems.stop()?;
        log::info!("World stopped");
        Ok(())
    }

    /// Shutdown the world
    pub fn shutdown(&mut self) -> Result<(), EcsError> {
        if self.running {
            self.stop()?;
        }

        if self.initialized {
            self.systems.shutdown()?;
            self.initialized = false;
        }

        log::info!("World shut down");
        Ok(())
    }

    /// Check if the world is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Check if the world is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Create a new entity with generation tracking
    pub fn create_entity(&mut self) -> EntityId {
        let entity = Entity::new();
        let id = entity.id();
        
        // Track the entity generation
        let generation = EntityGeneration::with_generation(id.generation());
        self.entity_generations.insert(id, generation);
        self.entities.insert(id, entity);
        
        log::trace!("Created entity {} with generation {}", id.value(), id.generation());
        id
    }

    /// Remove an entity (with generation tracking)
    pub fn remove_entity(&mut self, entity_id: &EntityId) -> Result<(), EcsError> {
        // Validate entity generation first
        if let Some(generation) = self.entity_generations.get_mut(entity_id) {
            entity_id.validate(generation)?;
            generation.mark_dead();
        } else {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        if self.entities.remove(entity_id).is_none() {
            return Err(EcsError::EntityNotFound(*entity_id));
        }

        // Remove components for this entity
        self.components.remove_all(entity_id);

        log::trace!("Removed entity {} with generation {}", entity_id.value(), entity_id.generation());
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

    /// Update all systems - safe version with proper error handling
    pub fn update(&mut self, delta_time: f32) -> Result<(), EcsError> {
        if !self.initialized {
            return Err(EcsError::NotInitialized);
        }
        if !self.running {
            return Err(EcsError::NotRunning);
        }

        // Extract systems into a temporary variable to avoid borrowing conflicts
        let mut temp_systems = SystemRegistry::new();
        std::mem::swap(&mut self.systems, &mut temp_systems);
        
        // Update systems with the world
        temp_systems.update_all(self, delta_time);
        
        // Move systems back
        std::mem::swap(&mut self.systems, &mut temp_systems);
        
        Ok(())
    }

    /// Get the number of entities
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the number of systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Validate that an entity exists and is alive
    pub fn validate_entity(&self, entity_id: &EntityId) -> Result<(), EcsError> {
        if let Some(generation) = self.entity_generations.get(entity_id) {
            entity_id.validate(generation)?;
        } else {
            return Err(EcsError::EntityNotFound(*entity_id));
        }
        Ok(())
    }

    /// Get the current generation for an entity
    pub fn get_entity_generation(&self, entity_id: &EntityId) -> Option<EntityGeneration> {
        self.entity_generations.get(entity_id).copied()
    }

    /// Check if an entity reference is still valid
    pub fn is_entity_reference_valid(&self, reference: &crate::generation::EntityReference) -> bool {
        if let Some(entity) = self.entities.get(&EntityId::with_generation(reference.id(), reference.generation())) {
            entity.is_active() && entity.matches_reference(reference)
        } else {
            false
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}