//! Scene graph management for the engine.
//!
//! This module provides functionality for managing the game scene graph, similar to Godot's scene system.

use ecs::World;
use crate::lifecycle::{LifecycleManager, LifecycleState};

/// A simple scene graph container
pub struct Scene {
    /// Lifecycle manager for scene state
    lifecycle: LifecycleManager,
    /// Name of the scene
    name: String,
    /// The ECS world
    pub world: World,
}

impl Scene {
    /// Create a new scene with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            lifecycle: LifecycleManager::new(),
            name: name.into(),
            world: World::default(),
        }
    }

    /// Initialize the scene with proper lifecycle management
    pub fn initialize(&mut self) -> Result<(), String> {
        self.lifecycle.begin_initialization()?;
        
        // Initialize the world
        self.world.initialize().map_err(|e| format!("Failed to initialize world: {}", e))?;
        
        self.lifecycle.complete_initialization()?;
        log::info!("Scene '{}' initialized successfully", self.name);
        Ok(())
    }

    /// Start the scene (transition to running state)
    pub fn start(&mut self) -> Result<(), String> {
        self.lifecycle.start()?;
        self.world.start().map_err(|e| format!("Failed to start world: {}", e))?;
        log::info!("Scene '{}' started", self.name);
        Ok(())
    }

    /// Stop the scene (transition from running to initialized)
    pub fn stop(&mut self) -> Result<(), String> {
        self.lifecycle.stop()?;
        self.world.stop().map_err(|e| format!("Failed to stop world: {}", e))?;
        log::info!("Scene '{}' stopped", self.name);
        Ok(())
    }

    /// Shutdown the scene
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.lifecycle.begin_shutdown()?;
        self.world.shutdown().map_err(|e| format!("Failed to shutdown world: {}", e))?;
        self.lifecycle.complete_shutdown()?;
        log::info!("Scene '{}' shut down successfully", self.name);
        Ok(())
    }

    /// Check if the scene is operational (initialized or running)
    pub fn is_operational(&self) -> bool {
        self.lifecycle.is_operational()
    }

    /// Check if the scene is initialized (can be either Initialized or Running)
    pub fn is_initialized(&self) -> bool {
        matches!(self.lifecycle.current_state(), LifecycleState::Initialized | LifecycleState::Running)
    }

    /// Check if the scene is running
    pub fn is_running(&self) -> bool {
        matches!(self.lifecycle.current_state(), LifecycleState::Running)
    }

    /// Get the current lifecycle state
    pub fn lifecycle_state(&self) -> LifecycleState {
        self.lifecycle.current_state()
    }

    /// Get the name of the scene
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Update the scene graph (only if running)
    pub fn update(&mut self, delta_time: f32) -> Result<(), String> {
        if !self.lifecycle.is_operational() {
            return Err(format!("Scene '{}' is not operational (current state: {:?})", 
                             self.name, self.lifecycle.current_state()));
        }

        // Update the world
        self.world.update(delta_time).map_err(|e| format!("Failed to update world: {}", e))?;
        log::trace!("Updating scene '{}', delta: {:.4}s", self.name, delta_time);
        Ok(())
    }

    /// Update the scene with a borrowed schedule (only if running)
    pub fn update_with_schedule(&mut self, schedule: &mut ecs::SystemRegistry, dt: f32) -> Result<(), String> {
        if !self.lifecycle.is_operational() {
            return Err(format!("Scene '{}' is not operational (current state: {:?})", 
                             self.name, self.lifecycle.current_state()));
        }

        // Execute the schedule on the world
        schedule.update_all(&mut self.world, dt);
        log::trace!("Updating scene '{}' with schedule, delta: {:.4}s", self.name, dt);
        Ok(())
    }
}
