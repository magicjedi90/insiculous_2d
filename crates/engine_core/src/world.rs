//! World state management for the engine.
//!
//! This module provides functionality for managing the game world state.

/// A simple world state container
pub struct World {
    /// Whether the world is initialized
    initialized: bool,
    /// Name of the world
    name: String,
}

impl World {
    /// Create a new world with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            initialized: false,
            name: name.into(),
        }
    }

    /// Initialize the world
    pub fn initialize(&mut self) {
        self.initialized = true;
        log::info!("World '{}' initialized", self.name);
    }

    /// Check if the world is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the name of the world
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Update the world state
    pub fn update(&mut self, delta_time: f32) {
        if !self.initialized {
            return;
        }

        // In a real implementation, this would update all entities and systems
        log::trace!("Updating world '{}', delta: {:.4}s", self.name, delta_time);
    }
}
