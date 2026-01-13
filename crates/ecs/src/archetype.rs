//! Archetype-based component storage for optimized ECS performance.
//!
//! This module implements archetype-based component storage, which groups entities
//! with the same component types together for better cache locality and iteration performance.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::vec::Vec;

use crate::component::Component;
use crate::entity::EntityId;


/// An archetype represents a unique combination of component types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeId {
    /// The sorted list of component type IDs that define this archetype
    component_types: Vec<TypeId>,
}

impl ArchetypeId {
    /// Create a new archetype ID from a set of component types
    pub fn new(mut component_types: Vec<TypeId>) -> Self {
        component_types.sort_unstable();
        component_types.dedup();
        Self { component_types }
    }

    /// Get the component types for this archetype
    pub fn component_types(&self) -> &[TypeId] {
        &self.component_types
    }

    /// Check if this archetype contains a specific component type
    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.component_types.binary_search(type_id).is_ok()
    }
}

/// Dense storage for components of a specific type within an archetype
pub struct ComponentColumn {
    /// The component data stored as raw bytes
    data: Vec<u8>,
    /// The size of each component in bytes
    element_size: usize,
    /// The number of components stored
    len: usize,
    /// The capacity of the storage
    capacity: usize,
}

impl ComponentColumn {
    /// Create a new component column for components of the given size
    pub fn new(element_size: usize) -> Self {
        Self {
            data: Vec::new(),
            element_size,
            len: 0,
            capacity: 0,
        }
    }

    /// Get the number of components in this column
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the column is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a pointer to the component at the given index
    pub fn get(&self, index: usize) -> Option<*const u8> {
        if index < self.len {
            Some(unsafe { self.data.as_ptr().add(index * self.element_size) })
        } else {
            None
        }
    }

    /// Get a mutable pointer to the component at the given index
    pub fn get_mut(&mut self, index: usize) -> Option<*mut u8> {
        if index < self.len {
            Some(unsafe { self.data.as_mut_ptr().add(index * self.element_size) })
        } else {
            None
        }
    }

    /// Push a new component to the end of the column
    pub fn push(&mut self, component: &dyn Component) {
        if self.len >= self.capacity {
            self.grow();
        }

        let dest = unsafe { self.data.as_mut_ptr().add(self.len * self.element_size) };
        unsafe {
            std::ptr::copy_nonoverlapping(component as *const dyn Component as *const u8, dest, self.element_size);
        }
        self.len += 1;
    }

    /// Remove the component at the given index and swap with the last element
    pub fn swap_remove(&mut self, index: usize) {
        if index >= self.len {
            return;
        }

        if index != self.len - 1 {
            let src = unsafe { self.data.as_ptr().add((self.len - 1) * self.element_size) };
            let dest = unsafe { self.data.as_mut_ptr().add(index * self.element_size) };
            unsafe {
                std::ptr::copy_nonoverlapping(src, dest, self.element_size);
            }
        }
        self.len -= 1;
    }

    /// Grow the storage capacity
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 { 16 } else { self.capacity * 2 };
        let new_size = new_capacity * self.element_size;
        self.data.resize(new_size, 0);
        self.capacity = new_capacity;
    }

    /// Clear all components from the column
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// An archetype stores entities with the same set of components
pub struct Archetype {
    /// The unique ID for this archetype
    id: ArchetypeId,
    /// The entities in this archetype
    entities: Vec<EntityId>,
    /// Component columns indexed by type ID
    components: HashMap<TypeId, ComponentColumn>,
    /// Mapping from entity ID to index within the archetype
    entity_indices: HashMap<EntityId, usize>,
}

impl Archetype {
    /// Create a new archetype with the given ID
    pub fn new(id: ArchetypeId) -> Self {
        Self {
            id,
            entities: Vec::new(),
            components: HashMap::new(),
            entity_indices: HashMap::new(),
        }
    }

    /// Get the archetype ID
    pub fn id(&self) -> &ArchetypeId {
        &self.id
    }

    /// Get the number of entities in this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if the archetype is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Add an entity to this archetype with its components
    #[allow(clippy::borrowed_box)]
    pub fn add_entity(&mut self, entity: EntityId, components: Vec<Box<dyn Component>>) {
        let index = self.entities.len();
        self.entities.push(entity);
        self.entity_indices.insert(entity, index);

        // Add components to their respective columns
        for component in components {
            // Get type_id from the concrete type inside the Box
            let type_id = (*component).type_id();
            let column = self.components.entry(type_id).or_insert_with(|| {
                ComponentColumn::new(std::mem::size_of_val(component.as_ref()))
            });
            column.push(component.as_ref());
        }
    }

    /// Remove an entity from this archetype
    pub fn remove_entity(&mut self, entity: &EntityId) -> Option<usize> {
        if let Some(&index) = self.entity_indices.get(entity) {
            // Remove the entity
            self.entities.swap_remove(index);
            self.entity_indices.remove(entity);

            // Update indices for the swapped entity
            if index < self.entities.len() {
                let swapped_entity = self.entities[index];
                self.entity_indices.insert(swapped_entity, index);
            }

            // Remove components from columns
            for column in self.components.values_mut() {
                column.swap_remove(index);
            }

            Some(index)
        } else {
            None
        }
    }

    /// Get the index of an entity within this archetype
    pub fn get_entity_index(&self, entity: &EntityId) -> Option<usize> {
        self.entity_indices.get(entity).copied()
    }

    /// Get a component column by type
    pub fn get_column(&self, type_id: &TypeId) -> Option<&ComponentColumn> {
        self.components.get(type_id)
    }

    /// Get a mutable component column by type
    pub fn get_column_mut(&mut self, type_id: &TypeId) -> Option<&mut ComponentColumn> {
        self.components.get_mut(type_id)
    }

    /// Get all entities in this archetype
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }
}

/// Type-safe query for entities with specific component types
/// Note: Scaffolding for future full query implementation
pub struct Query<T: QueryTypes> {
    #[allow(dead_code)] // Scaffolding for query execution
    archetypes: Vec<ArchetypeId>,
    _phantom: PhantomData<T>,
}

/// Trait for defining query types
pub trait QueryTypes {
    /// Get the component types required by this query
    fn component_types() -> Vec<TypeId>;
}

/// Single component query
pub struct Single<T: Component> {
    _phantom: PhantomData<T>,
}

impl<T: Component> QueryTypes for Single<T> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

/// Two component query
pub struct Pair<T: Component, U: Component> {
    _phantom: PhantomData<(T, U)>,
}

impl<T: Component, U: Component> QueryTypes for Pair<T, U> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>(), TypeId::of::<U>()]
    }
}

/// Three component query
pub struct Triple<T: Component, U: Component, V: Component> {
    _phantom: PhantomData<(T, U, V)>,
}

impl<T: Component, U: Component, V: Component> QueryTypes for Triple<T, U, V> {
    fn component_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>(), TypeId::of::<U>(), TypeId::of::<V>()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestComponent {
        value: i32,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct OtherComponent {
        name: String,
    }

    #[test]
    fn test_archetype_id_creation() {
        let mut types = vec![TypeId::of::<TestComponent>(), TypeId::of::<OtherComponent>()];
        let archetype_id = ArchetypeId::new(types.clone());
        
        types.sort_unstable();
        assert_eq!(archetype_id.component_types(), &types);
        
        assert!(archetype_id.contains(&TypeId::of::<TestComponent>()));
        assert!(archetype_id.contains(&TypeId::of::<OtherComponent>()));
        assert!(!archetype_id.contains(&TypeId::of::<i32>()));
    }

    #[test]
    fn test_component_column() {
        let mut column = ComponentColumn::new(std::mem::size_of::<TestComponent>());
        
        let component = TestComponent { value: 42 };
        column.push(&component);
        
        assert_eq!(column.len(), 1);
        assert!(!column.is_empty());
        
        let ptr = column.get(0).unwrap();
        let retrieved = unsafe { &*(ptr as *const TestComponent) };
        assert_eq!(retrieved.value, 42);
    }
}