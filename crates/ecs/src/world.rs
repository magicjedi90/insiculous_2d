//! World management for the ECS.

use std::collections::HashMap;

use crate::component::{Component, ComponentRegistry};
use crate::entity::{Entity, EntityId};
use crate::generation::EntityGeneration;
use crate::system::SystemRegistry;
use crate::archetype::QueryTypes;
use crate::ArchetypeStorage;
use crate::EcsError;

/// Configuration for the ECS world
#[derive(Debug, Clone)]
pub struct WorldConfig {
    /// Whether to use archetype-based component storage for better performance
    pub use_archetype_storage: bool,
    /// Initial capacity for entity storage
    pub entity_capacity: usize,
    /// Initial capacity for component storage
    pub component_capacity: usize,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            use_archetype_storage: false, // Default to legacy for backward compatibility
            entity_capacity: 1024,
            component_capacity: 4096,
        }
    }
}

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
    /// Archetype storage for optimized component access
    archetype_storage: Option<ArchetypeStorage>,
    /// Whether the world is initialized
    initialized: bool,
    /// Whether the world is running
    running: bool,
    /// World configuration
    config: WorldConfig,
}

impl World {
    /// Create a new world with default configuration
    pub fn new() -> Self {
        Self::with_config(WorldConfig::default())
    }

    /// Create a new world with custom configuration
    pub fn with_config(config: WorldConfig) -> Self {
        let components = if config.use_archetype_storage {
            ComponentRegistry::new_archetype_based()
        } else {
            ComponentRegistry::new()
        };

        let archetype_storage = if config.use_archetype_storage {
            Some(ArchetypeStorage::new())
        } else {
            None
        };

        Self {
            entities: HashMap::with_capacity(config.entity_capacity),
            entity_generations: HashMap::with_capacity(config.entity_capacity),
            components,
            systems: SystemRegistry::new(),
            archetype_storage,
            initialized: false,
            running: false,
            config,
        }
    }

    /// Create a new world with archetype-based storage for optimal performance
    pub fn new_optimized() -> Self {
        let mut config = WorldConfig::default();
        config.use_archetype_storage = true;
        Self::with_config(config)
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

    /// Get a reference to a component for an entity (returns trait object)
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

    /// Get a mutable reference to a component for an entity (returns trait object)
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

    /// Get a typed reference to a component for an entity
    pub fn get<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        if !self.entities.contains_key(&entity_id) {
            return None;
        }
        self.components.get_typed::<T>(&entity_id)
    }

    /// Get a typed mutable reference to a component for an entity
    pub fn get_mut<T: Component>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        if !self.entities.contains_key(&entity_id) {
            return None;
        }
        self.components.get_typed_mut::<T>(&entity_id)
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

    /// Get an iterator over all entity IDs
    pub fn entities(&self) -> Vec<EntityId> {
        self.entities.keys().copied().collect()
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

    /// Query for entities with specific component types
    pub fn query<Q: QueryTypes>(&self) -> QueryIterator<'_, Q> {
        QueryIterator::new(self)
    }

    /// Get world configuration
    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    /// Check if using archetype storage
    pub fn uses_archetype_storage(&self) -> bool {
        self.config.use_archetype_storage
    }

    // ============= Hierarchy Management =============

    /// Set an entity's parent, establishing a parent-child relationship
    ///
    /// This will:
    /// 1. Add a Parent component to the child entity
    /// 2. Add/update a Children component on the parent entity
    /// 3. Remove the child from any previous parent's Children component
    ///
    /// # Errors
    /// Returns an error if either entity doesn't exist
    pub fn set_parent(&mut self, child: EntityId, parent: EntityId) -> Result<(), EcsError> {
        // Validate both entities exist
        if !self.entities.contains_key(&child) {
            return Err(EcsError::EntityNotFound(child));
        }
        if !self.entities.contains_key(&parent) {
            return Err(EcsError::EntityNotFound(parent));
        }

        // Prevent self-parenting
        if child == parent {
            return Err(EcsError::SystemError("Cannot set entity as its own parent".to_string()));
        }

        // Remove from previous parent if any
        if let Some(old_parent) = self.get::<crate::hierarchy::Parent>(child) {
            let old_parent_id = old_parent.entity();
            if old_parent_id != parent {
                // Remove child from old parent's Children component
                if let Some(old_children) = self.get_mut::<crate::hierarchy::Children>(old_parent_id) {
                    old_children.remove(&child);
                }
            }
        }

        // Set the Parent component on the child
        self.components.add(child, crate::hierarchy::Parent::new(parent));

        // Add/update Children component on parent
        if let Some(children) = self.get_mut::<crate::hierarchy::Children>(parent) {
            children.add(child);
        } else {
            // Parent doesn't have Children component yet, add it
            self.components.add(parent, crate::hierarchy::Children::with_children(vec![child]));
        }

        log::trace!("Set parent of entity {} to {}", child.value(), parent.value());
        Ok(())
    }

    /// Remove an entity's parent, making it a root entity
    ///
    /// This will:
    /// 1. Remove the Parent component from the entity
    /// 2. Remove the entity from its parent's Children component
    ///
    /// # Errors
    /// Returns an error if the entity doesn't exist
    pub fn remove_parent(&mut self, entity: EntityId) -> Result<(), EcsError> {
        if !self.entities.contains_key(&entity) {
            return Err(EcsError::EntityNotFound(entity));
        }

        // Get and remove the parent reference
        if let Some(parent) = self.get::<crate::hierarchy::Parent>(entity) {
            let parent_id = parent.entity();

            // Remove from parent's Children component
            if let Some(children) = self.get_mut::<crate::hierarchy::Children>(parent_id) {
                children.remove(&entity);
            }
        }

        // Remove the Parent component
        let _ = self.components.remove::<crate::hierarchy::Parent>(&entity);

        log::trace!("Removed parent from entity {}", entity.value());
        Ok(())
    }

    /// Get an entity's parent
    pub fn get_parent(&self, entity: EntityId) -> Option<EntityId> {
        self.get::<crate::hierarchy::Parent>(entity).map(|p| p.entity())
    }

    /// Get an entity's children
    pub fn get_children(&self, entity: EntityId) -> Option<&[EntityId]> {
        self.get::<crate::hierarchy::Children>(entity).map(|c| c.entities())
    }

    /// Get all root entities (entities without a parent)
    pub fn get_root_entities(&self) -> Vec<EntityId> {
        self.entities
            .keys()
            .filter(|id| !self.components.has::<crate::hierarchy::Parent>(id))
            .copied()
            .collect()
    }

    /// Recursively get all descendants of an entity
    pub fn get_descendants(&self, entity: EntityId) -> Vec<EntityId> {
        let mut descendants = Vec::new();
        self.collect_descendants(entity, &mut descendants);
        descendants
    }

    /// Helper function to recursively collect descendants
    fn collect_descendants(&self, entity: EntityId, descendants: &mut Vec<EntityId>) {
        if let Some(children) = self.get_children(entity) {
            for &child in children {
                descendants.push(child);
                self.collect_descendants(child, descendants);
            }
        }
    }

    /// Get all ancestors of an entity (parent, grandparent, etc.)
    pub fn get_ancestors(&self, entity: EntityId) -> Vec<EntityId> {
        let mut ancestors = Vec::new();
        let mut current = entity;

        while let Some(parent) = self.get_parent(current) {
            ancestors.push(parent);
            current = parent;
        }

        ancestors
    }

    /// Check if an entity is an ancestor of another entity
    pub fn is_ancestor_of(&self, potential_ancestor: EntityId, entity: EntityId) -> bool {
        self.get_ancestors(entity).contains(&potential_ancestor)
    }

    /// Check if an entity is a descendant of another entity
    pub fn is_descendant_of(&self, potential_descendant: EntityId, entity: EntityId) -> bool {
        self.get_descendants(entity).contains(&potential_descendant)
    }

    /// Remove an entity and all its descendants from the hierarchy
    ///
    /// This recursively removes all children and their children, etc.
    pub fn remove_entity_hierarchy(&mut self, entity: &EntityId) -> Result<(), EcsError> {
        // First, collect all descendants (we need to do this before we start removing)
        let descendants = self.get_descendants(*entity);

        // Remove descendants in reverse order (deepest first)
        for descendant in descendants.into_iter().rev() {
            self.remove_entity(&descendant)?;
        }

        // Remove the parent relationship
        self.remove_parent(*entity)?;

        // Remove the entity itself
        self.remove_entity(entity)?;

        Ok(())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator for querying entities with specific component types
pub struct QueryIterator<'w, Q: QueryTypes> {
    #[allow(dead_code)]
    world: &'w World,
    #[allow(dead_code)]
    archetype_ids: Vec<crate::archetype::ArchetypeId>,
    #[allow(dead_code)]
    current_archetype: usize,
    #[allow(dead_code)]
    current_entity: usize,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'w, Q: QueryTypes> QueryIterator<'w, Q> {
    fn new(world: &'w World) -> Self {
        // For now, this is a simplified implementation
        // In a full implementation, we'd filter archetypes based on the query types
        let archetype_ids = if let Some(_storage) = &world.archetype_storage {
            // Get archetype IDs that contain all required component types
            let _required_types = Q::component_types();
            Vec::new() // Placeholder - would filter archetypes
        } else {
            Vec::new()
        };

        Self {
            world,
            archetype_ids,
            current_archetype: 0,
            current_entity: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'w, Q: QueryTypes> Iterator for QueryIterator<'w, Q> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        // Simplified implementation - would iterate through matching archetypes and entities
        None
    }
}