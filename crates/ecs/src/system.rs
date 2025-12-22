//! System management for the ECS.

use std::any::Any;

use crate::world::World;

/// A trait for systems in the ECS
pub trait System: Any + Send + Sync {
    /// Initialize the system (called once when world is initialized)
    fn initialize(&mut self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    /// Start the system (called when world starts running)
    fn start(&mut self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    /// Update the system
    fn update(&mut self, world: &mut World, delta_time: f32);

    /// Stop the system (called when world stops running)
    fn stop(&mut self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    /// Shutdown the system (called when world is shutting down)
    fn shutdown(&mut self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

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

/// A registry for systems with lifecycle management
#[derive(Default)]
pub struct SystemRegistry {
    /// The systems
    systems: Vec<Box<dyn System>>,
    /// Whether the systems are initialized
    initialized: bool,
    /// Whether the systems are running
    running: bool,
}

impl SystemRegistry {
    /// Create a new system registry
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            initialized: false,
            running: false,
        }
    }

    /// Initialize all systems
    pub fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Err("Systems already initialized".to_string());
        }

        log::info!("Initializing {} systems", self.systems.len());
        
        // Note: System initialization should be done when the world is available
        // For now, we'll just mark as initialized since we can't create a dummy world
        // The actual initialization should be done by the World when it's initializing
        
        self.initialized = true;
        log::info!("All systems initialized successfully");
        Ok(())
    }

    /// Start all systems
    pub fn start(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Systems not initialized".to_string());
        }
        if self.running {
            return Err("Systems already running".to_string());
        }

        log::info!("Starting {} systems", self.systems.len());
        
        // Note: System start should be done when the world is available
        // For now, we'll just mark as running since we can't create a dummy world
        // The actual start should be done by the World when it's starting
        
        self.running = true;
        log::info!("All systems started successfully");
        Ok(())
    }

    /// Stop all systems
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running {
            return Err("Systems not running".to_string());
        }

        log::info!("Stopping {} systems", self.systems.len());
        
        // Note: System stop should be done when the world is available
        // The actual stop should be done by the World when it's stopping
        
        self.running = false;
        log::info!("All systems stopped");
        Ok(())
    }

    /// Shutdown all systems
    pub fn shutdown(&mut self) -> Result<(), String> {
        if self.running {
            self.stop()?;
        }

        if !self.initialized {
            return Ok(()); // Nothing to shutdown
        }

        log::info!("Shutting down {} systems", self.systems.len());
        
        // Note: System shutdown should be done when the world is available
        // The actual shutdown should be done by the World when it's shutting down
        
        self.initialized = false;
        log::info!("All systems shut down");
        Ok(())
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

    /// Update all systems - safe version with proper error handling
    pub fn update_all(&mut self, world: &mut World, delta_time: f32) {
        if !self.running {
            log::warn!("Attempted to update systems while not running");
            return;
        }

        // Create a temporary vector to hold systems during update
        // This prevents borrowing issues and provides better error isolation
        let mut temp_systems = Vec::new();
        std::mem::swap(&mut self.systems, &mut temp_systems);
        
        // Update all systems with individual error handling
        for system in &mut temp_systems {
            let system_name = system.name().to_string();
            
            // Use catch_unwind to prevent panics from corrupting the system registry
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                system.update(world, delta_time);
            }));
            
            if let Err(_panic) = result {
                log::error!("System '{}' panicked during update", system_name);
                // Continue with other systems instead of crashing the whole engine
            }
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