//! Scene graph management for the engine.
//!
//! This module provides functionality for managing the game scene graph, similar to Godot's scene system.

/// A simple scene graph container
pub struct Scene {
    /// Whether the scene is initialized
    initialized: bool,
    /// Name of the scene
    name: String,
}

impl Scene {
    /// Create a new scene with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            initialized: false,
            name: name.into(),
        }
    }

    /// Initialize the scene
    pub fn initialize(&mut self) {
        self.initialized = true;
        log::info!("Scene '{}' initialized", self.name);
    }

    /// Check if the scene is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the name of the scene
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Update the scene graph
    pub fn update(&mut self, delta_time: f32) {
        if !self.initialized {
            return;
        }

        // In a real implementation, this would update all entities and systems
        log::trace!("Updating scene '{}', delta: {:.4}s", self.name, delta_time);
    }
}