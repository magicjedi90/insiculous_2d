//! Physics system for ECS integration
//!
//! This module provides a system that synchronizes ECS components with the physics world.
//!
//! # API Design: Pass-Through Methods
//!
//! [`PhysicsSystem`] provides several methods that delegate directly to [`PhysicsWorld`]:
//! - `set_gravity()` / `gravity()`
//! - `apply_impulse()` / `apply_force()`
//! - `raycast()`
//! - `collision_events()`
//!
//! These pass-through methods exist intentionally for **API ergonomics**:
//!
//! ```ignore
//! // With pass-through (cleaner):
//! physics_system.apply_impulse(entity, Vec2::new(0.0, 100.0));
//!
//! // Without pass-through:
//! physics_system.physics_world_mut().apply_impulse(entity, Vec2::new(0.0, 100.0));
//! ```
//!
//! Users who need advanced physics operations can still access the underlying
//! [`PhysicsWorld`] via [`physics_world()`](PhysicsSystem::physics_world) and
//! [`physics_world_mut()`](PhysicsSystem::physics_world_mut).

use glam::Vec2;

use ecs::{EntityId, System, World};
use ecs::sprite_components::Transform2D;

use crate::components::{Collider, RigidBody, CollisionData};
use crate::physics_world::{PhysicsConfig, PhysicsWorld};

/// Type alias for collision callback to reduce complexity
type CollisionCallback = Box<dyn FnMut(&CollisionData) + Send + Sync>;

/// Physics system that steps the simulation and syncs transforms
pub struct PhysicsSystem {
    /// The physics world
    physics_world: PhysicsWorld,
    /// Accumulated time for fixed timestep
    time_accumulator: f32,
    /// Fixed timestep for physics updates (1/60 second by default)
    fixed_timestep: f32,
    /// Maximum delta time to prevent spiral of death
    max_delta_time: f32,
    /// Callbacks for collision events (supports multiple listeners)
    collision_callbacks: Vec<CollisionCallback>,
}

impl PhysicsSystem {
    /// Create a new physics system with default configuration
    pub fn new() -> Self {
        Self::with_config(PhysicsConfig::default())
    }

    /// Create a new physics system with custom configuration
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self {
            physics_world: PhysicsWorld::new(config),
            time_accumulator: 0.0,
            fixed_timestep: 1.0 / 60.0,
            max_delta_time: 0.1,
            collision_callbacks: Vec::new(),
        }
    }

    /// Set the fixed timestep for physics updates
    pub fn with_fixed_timestep(mut self, timestep: f32) -> Self {
        self.fixed_timestep = timestep;
        self
    }

    /// Add a collision callback (builder pattern)
    ///
    /// Multiple callbacks can be registered. They will all be invoked
    /// for each collision event in the order they were added.
    pub fn with_collision_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&CollisionData) + Send + Sync + 'static,
    {
        self.collision_callbacks.push(Box::new(callback));
        self
    }

    /// Add a collision callback (mutable method)
    ///
    /// Multiple callbacks can be registered. They will all be invoked
    /// for each collision event in the order they were added.
    pub fn add_collision_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&CollisionData) + Send + Sync + 'static,
    {
        self.collision_callbacks.push(Box::new(callback));
    }

    /// Remove all collision callbacks
    pub fn clear_collision_callbacks(&mut self) {
        self.collision_callbacks.clear();
    }

    /// Get the number of registered collision callbacks
    pub fn collision_callback_count(&self) -> usize {
        self.collision_callbacks.len()
    }

    /// Get a reference to the physics world
    pub fn physics_world(&self) -> &PhysicsWorld {
        &self.physics_world
    }

    /// Get a mutable reference to the physics world
    pub fn physics_world_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.physics_world
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.physics_world.set_gravity(gravity);
    }

    /// Get gravity
    pub fn gravity(&self) -> Vec2 {
        self.physics_world.gravity()
    }

    /// Apply an impulse to an entity
    pub fn apply_impulse(&mut self, entity: EntityId, impulse: Vec2) {
        self.physics_world.apply_impulse(entity, impulse);
    }

    /// Apply a force to an entity
    pub fn apply_force(&mut self, entity: EntityId, force: Vec2) {
        self.physics_world.apply_force(entity, force);
    }

    /// Cast a ray and return the first hit
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_distance: f32) -> Option<(EntityId, Vec2, f32)> {
        self.physics_world.raycast(origin, direction, max_distance)
    }

    /// Get collision events from the last physics step
    pub fn collision_events(&self) -> &[CollisionData] {
        self.physics_world.collision_events()
    }

    /// Sync a single entity from ECS to physics world
    fn sync_entity_to_physics(&mut self, world: &mut World, entity: EntityId) {
        // Get transform for position
        let transform = world.get::<Transform2D>(entity).cloned();

        // Check if entity has rigid body component
        if let Some(mut rigid_body) = world.get::<RigidBody>(entity).cloned() {
            let (position, rotation) = transform
                .as_ref()
                .map(|t| (t.position, t.rotation))
                .unwrap_or((Vec2::ZERO, 0.0));

            // Add rigid body to physics world if not already present
            if !self.physics_world.has_rigid_body(entity) {
                self.physics_world.add_rigid_body(entity, &mut rigid_body, position, rotation);
                // Update the component with the handle
                if let Some(body) = world.get_mut::<RigidBody>(entity) {
                    body.handle = rigid_body.handle;
                }
            }

            // Check for collider
            if let Some(mut collider) = world.get::<Collider>(entity).cloned() {
                if !self.physics_world.has_collider(entity) {
                    let body_ref = world.get::<RigidBody>(entity);
                    self.physics_world.add_collider(entity, &mut collider, body_ref);
                    // Update the component with the handle
                    if let Some(coll) = world.get_mut::<Collider>(entity) {
                        coll.handle = collider.handle;
                    }
                }
            }
        } else if let Some(mut collider) = world.get::<Collider>(entity).cloned() {
            // Collider without rigid body (static collision geometry)
            if !self.physics_world.has_collider(entity) {
                self.physics_world.add_collider(entity, &mut collider, None);
                if let Some(coll) = world.get_mut::<Collider>(entity) {
                    coll.handle = collider.handle;
                }
            }
        }
    }

    /// Sync physics results back to ECS transforms
    fn sync_physics_to_ecs(&self, world: &mut World) {
        let entities: Vec<EntityId> = world.entities();

        for entity in entities {
            // Get body type first to avoid borrow conflicts
            let body_type = world.get::<RigidBody>(entity).map(|b| b.body_type);

            if let Some(body_type) = body_type {
                // Sync both Dynamic and Kinematic bodies back to ECS
                // Static bodies don't move, so no need to sync them
                if body_type == crate::components::RigidBodyType::Dynamic
                    || body_type == crate::components::RigidBodyType::Kinematic
                {
                    // Get physics transform
                    if let Some((position, rotation)) = self.physics_world.get_body_transform(entity) {
                        // Update ECS transform
                        if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                            transform.position = position;
                            transform.rotation = rotation;
                        }
                    }

                    // Update velocity in component (for dynamic bodies)
                    if body_type == crate::components::RigidBodyType::Dynamic {
                        if let Some((linear_vel, angular_vel)) = self.physics_world.get_body_velocity(entity) {
                            if let Some(rigid_body) = world.get_mut::<RigidBody>(entity) {
                                rigid_body.velocity = linear_vel;
                                rigid_body.angular_velocity = angular_vel;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for PhysicsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for PhysicsSystem {
    fn initialize(&mut self, _world: &mut World) -> Result<(), String> {
        log::info!("PhysicsSystem initialized with gravity: {:?}", self.physics_world.gravity());
        Ok(())
    }

    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Clamp delta time to prevent instability
        let dt = delta_time.min(self.max_delta_time);

        // Get all entities and sync new ones to physics
        let entities: Vec<EntityId> = world.entities();
        for entity in entities {
            self.sync_entity_to_physics(world, entity);
        }

        // Fixed timestep physics updates
        self.time_accumulator += dt;

        while self.time_accumulator >= self.fixed_timestep {
            self.physics_world.step(self.fixed_timestep);
            self.time_accumulator -= self.fixed_timestep;
        }

        // Sync physics results back to ECS
        self.sync_physics_to_ecs(world);

        // Process collision callbacks (all registered callbacks receive each event)
        if !self.collision_callbacks.is_empty() {
            let events = self.physics_world.collision_events();
            for collision in events {
                for callback in &mut self.collision_callbacks {
                    callback(collision);
                }
            }
        }
    }

    fn shutdown(&mut self, _world: &mut World) -> Result<(), String> {
        log::info!("PhysicsSystem shutting down");
        Ok(())
    }

    fn name(&self) -> &str {
        "PhysicsSystem"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::sprite_components::Transform2D;

    #[test]
    fn test_physics_system_creation() {
        let system = PhysicsSystem::new();
        assert_eq!(system.fixed_timestep, 1.0 / 60.0);
        assert_eq!(system.gravity(), Vec2::new(0.0, -980.0));
    }

    #[test]
    fn test_physics_system_custom_config() {
        let config = PhysicsConfig::new(Vec2::new(0.0, -500.0));
        let system = PhysicsSystem::with_config(config);
        assert_eq!(system.gravity(), Vec2::new(0.0, -500.0));
    }

    #[test]
    fn test_entity_sync() {
        let mut world = World::new();
        let mut system = PhysicsSystem::new();

        // Create entity with physics components
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(100.0, 200.0))).unwrap();
        world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
        world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

        // Initialize and update system
        system.initialize(&mut world).unwrap();
        system.update(&mut world, 1.0 / 60.0);

        // Check physics world has the entity
        assert!(system.physics_world().has_rigid_body(entity));
        assert!(system.physics_world().has_collider(entity));
    }

    #[test]
    fn test_gravity_affects_dynamic_body() {
        let mut world = World::new();
        let mut system = PhysicsSystem::new();

        // Create falling entity
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
        world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
        world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

        let initial_y = world.get::<Transform2D>(entity).unwrap().position.y;

        // Run physics for several frames
        system.initialize(&mut world).unwrap();
        for _ in 0..10 {
            system.update(&mut world, 1.0 / 60.0);
        }

        // Check that entity has fallen
        let final_y = world.get::<Transform2D>(entity).unwrap().position.y;
        assert!(final_y < initial_y, "Entity should have fallen due to gravity");
    }

    #[test]
    fn test_static_body_does_not_move() {
        let mut world = World::new();
        let mut system = PhysicsSystem::new();

        // Create static entity
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
        world.add_component(&entity, RigidBody::new_static()).unwrap();
        world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

        let initial_pos = world.get::<Transform2D>(entity).unwrap().position;

        // Run physics
        system.initialize(&mut world).unwrap();
        for _ in 0..10 {
            system.update(&mut world, 1.0 / 60.0);
        }

        // Check that entity has not moved
        let final_pos = world.get::<Transform2D>(entity).unwrap().position;
        assert_eq!(initial_pos, final_pos, "Static body should not move");
    }

    #[test]
    fn test_multiple_collision_callbacks() {
        use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();

        let system = PhysicsSystem::new()
            .with_collision_callback(move |_| {
                counter1_clone.fetch_add(1, Ordering::SeqCst);
            })
            .with_collision_callback(move |_| {
                counter2_clone.fetch_add(1, Ordering::SeqCst);
            });

        // Verify both callbacks are registered
        assert_eq!(system.collision_callback_count(), 2);

        // Note: Without actual collisions, the callbacks won't be invoked,
        // but this test verifies the API works correctly
    }

    #[test]
    fn test_add_collision_callback() {
        let mut system = PhysicsSystem::new();
        assert_eq!(system.collision_callback_count(), 0);

        system.add_collision_callback(|_| {});
        assert_eq!(system.collision_callback_count(), 1);

        system.add_collision_callback(|_| {});
        assert_eq!(system.collision_callback_count(), 2);

        system.clear_collision_callbacks();
        assert_eq!(system.collision_callback_count(), 0);
    }
}
