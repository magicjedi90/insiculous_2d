//! Scene graph management for the engine.
//!
//! This module provides functionality for managing the game scene graph, similar to Godot's scene system.

use ecs::World;

/// A simple scene graph container
pub struct Scene {
    /// Whether the scene is initialized
    initialized: bool,
    /// Name of the scene
    name: String,
    /// The ECS world
    pub world: World,
}

impl Scene {
    /// Create a new scene with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            initialized: false,
            name: name.into(),
            world: World::default(),
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

        // Update the world
        self.world.update(delta_time);
        log::trace!("Updating scene '{}', delta: {:.4}s", self.name, delta_time);
    }

    /// Update the scene with a borrowed schedule
    pub fn update_with_schedule(&mut self, schedule: &mut ecs::SystemRegistry, dt: f32) {
        if !self.initialized {
            return;
        }

        // Execute the schedule on the world
        schedule.update_all(&mut self.world, dt);
        log::trace!("Updating scene '{}' with schedule, delta: {:.4}s", self.name, dt);
    }
}
