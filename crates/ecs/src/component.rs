//! Component management for the ECS.
//!
//! This module provides both the legacy HashMap-based storage and the new
//! archetype-based storage for optimal performance.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::entity::EntityId;
use crate::archetype::{Archetype, ArchetypeId};

/// A trait for components in the ECS
pub trait Component: Any + Send + Sync {
    /// Get the type name of the component
    fn type_name(&self) -> &'static str;

    /// Get self as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get self as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Send + Sync> Component for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Component storage that supports both legacy HashMap and new archetype-based storage
pub enum ComponentStorage {
    /// Legacy HashMap-based storage (for backward compatibility)
    Legacy(LegacyComponentStorage),
    /// New archetype-based storage (for performance)
    Archetype(ArchetypeStorage),
}

impl Default for ComponentStorage {
    fn default() -> Self {
        Self::Legacy(LegacyComponentStorage::default())
    }
}

impl ComponentStorage {
    /// Create a new component storage in legacy mode
    pub fn new_legacy() -> Self {
        Self::Legacy(LegacyComponentStorage::new())
    }

    /// Create a new component storage in archetype mode
    pub fn new_archetype() -> Self {
        Self::Archetype(ArchetypeStorage::new())
    }

    /// Add a component for an entity
    pub fn add<T: Component>(&mut self, entity_id: EntityId, component: T) {
        match self {
            Self::Legacy(storage) => storage.add(entity_id, component),
            Self::Archetype(storage) => storage.add(entity_id, component),
        }
    }

    /// Remove a component for an entity
    pub fn remove<T: Component>(&mut self, entity_id: &EntityId) -> Option<Box<dyn Component>> {
        match self {
            Self::Legacy(storage) => storage.remove::<T>(entity_id),
            Self::Archetype(storage) => storage.remove::<T>(entity_id),
        }
    }

    /// Get a reference to a component for an entity (returns trait object)
    pub fn get<T: Component>(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        match self {
            Self::Legacy(storage) => storage.get::<T>(entity_id),
            Self::Archetype(storage) => storage.get::<T>(entity_id),
        }
    }

    /// Get a mutable reference to a component for an entity (returns trait object)
    pub fn get_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        match self {
            Self::Legacy(storage) => storage.get_mut::<T>(entity_id),
            Self::Archetype(storage) => storage.get_mut::<T>(entity_id),
        }
    }

    /// Get a typed reference to a component for an entity
    pub fn get_typed<T: Component>(&self, entity_id: &EntityId) -> Option<&T> {
        self.get::<T>(entity_id)?.as_any().downcast_ref::<T>()
    }

    /// Get a typed mutable reference to a component for an entity
    pub fn get_typed_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut T> {
        self.get_mut::<T>(entity_id)?.as_any_mut().downcast_mut::<T>()
    }

    /// Check if an entity has a component
    pub fn has<T: Component>(&self, entity_id: &EntityId) -> bool {
        match self {
            Self::Legacy(storage) => storage.has::<T>(entity_id),
            Self::Archetype(storage) => storage.has::<T>(entity_id),
        }
    }

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity_id: &EntityId) {
        match self {
            Self::Legacy(storage) => storage.remove_all(entity_id),
            Self::Archetype(storage) => storage.remove_all(entity_id),
        }
    }

    /// Get the number of components (legacy only)
    pub fn len(&self) -> usize {
        match self {
            Self::Legacy(storage) => storage.len(),
            Self::Archetype(_) => 0, // Not meaningful for archetype storage
        }
    }

    /// Check if there are no components (legacy only)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Legacy(storage) => storage.is_empty(),
            Self::Archetype(storage) => storage.is_empty(),
        }
    }
}

/// Legacy HashMap-based component storage (for backward compatibility)
#[derive(Default)]
pub struct LegacyComponentStorage {
    /// The components, indexed by entity ID
    components: HashMap<EntityId, Box<dyn Component>>,
}

impl LegacyComponentStorage {
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
    pub fn remove<T: Component>(&mut self, entity_id: &EntityId) -> Option<Box<dyn Component>> {
        self.components.remove(entity_id)
    }

    /// Get a reference to a component for an entity
    pub fn get<T: Component>(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        self.components.get(entity_id).map(|c| c.as_ref())
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        self.components.get_mut(entity_id).map(|c| c.as_mut())
    }

    /// Check if an entity has a component
    pub fn has<T: Component>(&self, entity_id: &EntityId) -> bool {
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

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity_id: &EntityId) {
        self.components.remove(entity_id);
    }
}

/// New archetype-based component storage for optimal performance
#[derive(Default)]
pub struct ArchetypeStorage {
    /// Archetypes indexed by their ID
    archetypes: HashMap<ArchetypeId, Archetype>,
    /// Mapping from entity to its current archetype and index
    entity_locations: HashMap<EntityId, (ArchetypeId, usize)>,
}

impl ArchetypeStorage {
    /// Create a new archetype storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a component for an entity (moves entity between archetypes if needed)
    pub fn add<T: Component>(&mut self, entity_id: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        
        // Get current archetype for this entity, or create a new one
        let current_archetype_id = self.get_or_create_archetype_for_entity(&entity_id, type_id);
        
        // Add component to the appropriate archetype
        if let Some(archetype) = self.archetypes.get_mut(&current_archetype_id) {
            if let Some(_index) = archetype.get_entity_index(&entity_id) {
                // Entity already exists in this archetype, update component
                if let Some(_column) = archetype.get_column_mut(&type_id) {
                    // This is a simplified version - in a full implementation,
                    // we'd need to handle component replacement properly
                    log::trace!("Updating component for entity {:?} in archetype", entity_id);
                }
            } else {
                // New entity for this archetype
                let components = vec![Box::new(component) as Box<dyn Component>];
                archetype.add_entity(entity_id, components);
                let index = archetype.len() - 1;
                self.entity_locations.insert(entity_id, (current_archetype_id, index));
            }
        }
    }

    /// Remove a component from an entity
    pub fn remove<T: Component>(&mut self, entity_id: &EntityId) -> Option<Box<dyn Component>> {
        let _type_id = TypeId::of::<T>();
        
        if let Some((archetype_id, _index)) = self.entity_locations.get(entity_id).cloned() {
            if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
                // This is simplified - in a full implementation, we'd move the entity
                // to a new archetype without this component type
                archetype.remove_entity(entity_id);
                self.entity_locations.remove(entity_id);
                Some(Box::new(()) as Box<dyn Component>) // Placeholder
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a reference to a component for an entity
    pub fn get<T: Component>(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        let type_id = TypeId::of::<T>();
        
        if let Some((archetype_id, index)) = self.entity_locations.get(entity_id).cloned() {
            if let Some(archetype) = self.archetypes.get(&archetype_id) {
                if let Some(column) = archetype.get_column(&type_id) {
                    if let Some(ptr) = column.get(index) {
                        return Some(unsafe { &*(ptr as *const dyn Component) });
                    }
                }
            }
        }
        None
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        let type_id = TypeId::of::<T>();
        
        if let Some((archetype_id, index)) = self.entity_locations.get(entity_id).cloned() {
            if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
                if let Some(column) = archetype.get_column_mut(&type_id) {
                    if let Some(ptr) = column.get_mut(index) {
                        return Some(unsafe { &mut *(ptr as *mut dyn Component) });
                    }
                }
            }
        }
        None
    }

    /// Check if an entity has a component
    pub fn has<T: Component>(&self, entity_id: &EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        
        if let Some((archetype_id, _)) = self.entity_locations.get(entity_id).cloned() {
            archetype_id.contains(&type_id)
        } else {
            false
        }
    }

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity_id: &EntityId) {
        if let Some((archetype_id, _)) = self.entity_locations.remove(entity_id) {
            if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
                archetype.remove_entity(entity_id);
            }
        }
    }

    /// Check if the storage is empty
    pub fn is_empty(&self) -> bool {
        self.entity_locations.is_empty()
    }

    /// Get or create the appropriate archetype for an entity
    fn get_or_create_archetype_for_entity(&mut self, entity_id: &EntityId, new_component_type: TypeId) -> ArchetypeId {
        if let Some((current_archetype_id, _)) = self.entity_locations.get(entity_id).cloned() {
            // Entity already exists, we need to create a new archetype with the additional component
            let mut component_types = current_archetype_id.component_types().to_vec();
            component_types.push(new_component_type);
            let new_archetype_id = ArchetypeId::new(component_types);
            
            // Create the new archetype if it doesn't exist
            if !self.archetypes.contains_key(&new_archetype_id) {
                self.archetypes.insert(new_archetype_id.clone(), Archetype::new(new_archetype_id.clone()));
            }
            
            new_archetype_id
        } else {
            // New entity, create archetype with just this component
            let archetype_id = ArchetypeId::new(vec![new_component_type]);
            if !self.archetypes.contains_key(&archetype_id) {
                self.archetypes.insert(archetype_id.clone(), Archetype::new(archetype_id.clone()));
            }
            archetype_id
        }
    }
}

/// A registry for all component types
#[derive(Default)]
pub struct ComponentRegistry {
    /// The component storages, indexed by component type ID
    storages: HashMap<TypeId, ComponentStorage>,
    /// Whether to use archetype storage (true) or legacy storage (false)
    use_archetypes: bool,
}

impl ComponentRegistry {
    /// Create a new component registry with legacy storage
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            use_archetypes: false, // Default to legacy for backward compatibility
        }
    }

    /// Create a new component registry with archetype storage
    pub fn new_archetype_based() -> Self {
        Self {
            storages: HashMap::new(),
            use_archetypes: true,
        }
    }

    /// Register a component type
    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.storages.contains_key(&type_id) {
            let storage = if self.use_archetypes {
                ComponentStorage::new_archetype()
            } else {
                ComponentStorage::new_legacy()
            };
            self.storages.insert(type_id, storage);
        }
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
        self.storages.get_mut(&type_id)?.remove::<T>(entity_id)
    }

    /// Get a reference to a component for an entity (returns trait object)
    pub fn get<T: Component>(&self, entity_id: &EntityId) -> Option<&dyn Component> {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id)?.get::<T>(entity_id)
    }

    /// Get a mutable reference to a component for an entity (returns trait object)
    pub fn get_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut dyn Component> {
        let type_id = TypeId::of::<T>();
        self.storages.get_mut(&type_id)?.get_mut::<T>(entity_id)
    }

    /// Get a typed reference to a component for an entity
    pub fn get_typed<T: Component>(&self, entity_id: &EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id)?.get_typed::<T>(entity_id)
    }

    /// Get a typed mutable reference to a component for an entity
    pub fn get_typed_mut<T: Component>(&mut self, entity_id: &EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.storages.get_mut(&type_id)?.get_typed_mut::<T>(entity_id)
    }

    /// Check if an entity has a component
    pub fn has<T: Component>(&self, entity_id: &EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id).is_some_and(|s| s.has::<T>(entity_id))
    }

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity_id: &EntityId) {
        for storage in self.storages.values_mut() {
            storage.remove_all(entity_id);
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
