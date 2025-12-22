//! System management for the ECS.

use std::any::Any;

use crate::world::World;

/// A trait for systems in the ECS
pub trait System: Any + Send + Sync {
    /// Update the system
    fn update(&mut self, world: &mut World, delta_time: f32);

    /// Get the name of the system
    fn name(&self) -> &str;
}

/// A simple system that can be created from a closure
pub struct SimpleSystem<F>
where
    F: FnMut(&mut World, f32) + Send + Sync + 'static,
{
    /// The name of the system
    name: String,
    /// The update function
    update_fn: F,
}

impl<F> SimpleSystem<F>
where
    F: FnMut(&mut World, f32) + Send + Sync + 'static,
{
    /// Create a new simple system
    pub fn new(name: impl Into<String>, update_fn: F) -> Self {
        Self {
            name: name.into(),
            update_fn,
        }
    }
}

impl<F> System for SimpleSystem<F>
where
    F: FnMut(&mut World, f32) + Send + Sync + 'static,
{
    fn update(&mut self, world: &mut World, delta_time: f32) {
        (self.update_fn)(world, delta_time);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A registry for systems
#[derive(Default)]
pub struct SystemRegistry {
    /// The systems
    systems: Vec<Box<dyn System>>,
}

impl SystemRegistry {
    /// Create a new system registry
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system
    pub fn add<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    /// Add a simple system from a closure
    pub fn add_simple<F>(&mut self, name: impl Into<String>, update_fn: F)
    where
        F: FnMut(&mut World, f32) + Send + Sync + 'static,
    {
        self.add(SimpleSystem::new(name, update_fn));
    }

    /// Update all systems - safe version that doesn't borrow self mutably twice
    pub fn update_all(&mut self, world: &mut World, delta_time: f32) {
        // Create a temporary vector to hold systems during update
        // This prevents borrowing issues
        let mut temp_systems = Vec::new();
        std::mem::swap(&mut self.systems, &mut temp_systems);
        
        // Update all systems
        for system in &mut temp_systems {
            system.update(world, delta_time);
        }
        
        // Move systems back
        self.systems = temp_systems;
    }

    /// Get the number of systems
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Check if there are no systems
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}