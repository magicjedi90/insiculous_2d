//! Physics world wrapper for rapier2d integration
//!
//! This module provides a wrapper around rapier2d that integrates with the ECS.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;

use glam::Vec2;
use rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

use ecs::EntityId;

use crate::components::{
    Collider, ColliderShape, CollisionData, CollisionEvent, ContactPoint, RigidBody,
    RigidBodyType,
};

/// A canonical collision pair (entity IDs always in consistent order for comparison)
/// This ensures (A, B) and (B, A) are treated as the same collision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CollisionPair(EntityId, EntityId);

impl CollisionPair {
    /// Create a new collision pair with canonical ordering (smaller ID first)
    fn new(a: EntityId, b: EntityId) -> Self {
        // Use deterministic ordering based on entity ID internals
        // We compare combining id and generation to ensure consistent ordering
        let a_bits = a.value() | (a.generation() << 32);
        let b_bits = b.value() | (b.generation() << 32);
        if a_bits <= b_bits {
            Self(a, b)
        } else {
            Self(b, a)
        }
    }

    /// Get the entities in canonical order
    fn entities(&self) -> (EntityId, EntityId) {
        (self.0, self.1)
    }
}

/// Configuration for the physics world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity vector in units per second squared
    pub gravity: Vec2,
    /// Number of velocity iterations for the solver
    pub velocity_iterations: usize,
    /// Number of position iterations for the solver
    pub position_iterations: usize,
    /// Pixels per meter scale factor
    pub pixels_per_meter: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, -980.0), // 9.8 m/s^2 * 100 pixels/meter
            velocity_iterations: 16,  // Increased for better collision resolution
            position_iterations: 8,   // Increased for better stacking stability
            pixels_per_meter: 100.0,
        }
    }
}

impl PhysicsConfig {
    /// Create a new physics config with the given gravity
    pub fn new(gravity: Vec2) -> Self {
        Self {
            gravity,
            ..Default::default()
        }
    }

    /// Set gravity
    pub fn with_gravity(mut self, gravity: Vec2) -> Self {
        self.gravity = gravity;
        self
    }

    /// Set solver iterations
    pub fn with_iterations(mut self, velocity: usize, position: usize) -> Self {
        self.velocity_iterations = velocity;
        self.position_iterations = position;
        self
    }

    /// Set pixels per meter scale
    pub fn with_scale(mut self, pixels_per_meter: f32) -> Self {
        self.pixels_per_meter = pixels_per_meter;
        self
    }
}

/// Physics world that manages rapier2d simulation
pub struct PhysicsWorld {
    /// Rapier rigid body set
    rigid_body_set: RigidBodySet,
    /// Rapier collider set
    collider_set: ColliderSet,
    /// Physics pipeline
    physics_pipeline: PhysicsPipeline,
    /// Island manager
    island_manager: IslandManager,
    /// Broad phase
    broad_phase: DefaultBroadPhase,
    /// Narrow phase
    narrow_phase: NarrowPhase,
    /// Impulse joint set
    impulse_joint_set: ImpulseJointSet,
    /// Multibody joint set
    multibody_joint_set: MultibodyJointSet,
    /// CCD solver
    ccd_solver: CCDSolver,
    /// Query pipeline for raycasts and shape casts
    query_pipeline: QueryPipeline,
    /// Integration parameters
    integration_parameters: IntegrationParameters,
    /// Configuration
    config: PhysicsConfig,
    /// Mapping from ECS entity to rapier rigid body handle
    entity_to_body: HashMap<EntityId, RigidBodyHandle>,
    /// Mapping from rapier rigid body handle to ECS entity
    body_to_entity: HashMap<RigidBodyHandle, EntityId>,
    /// Mapping from ECS entity to rapier collider handle
    entity_to_collider: HashMap<EntityId, ColliderHandle>,
    /// Mapping from rapier collider handle to ECS entity
    collider_to_entity: HashMap<ColliderHandle, EntityId>,
    /// Collision events from the last step
    collision_events: Vec<CollisionData>,
    /// Active collision pairs from the previous frame (for detecting start/stop)
    previous_collisions: HashSet<CollisionPair>,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(PhysicsConfig::default())
    }
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new(config: PhysicsConfig) -> Self {
        let integration_parameters = IntegrationParameters {
            num_solver_iterations: NonZeroUsize::new(config.velocity_iterations)
                .unwrap_or(NonZeroUsize::new(8).unwrap()),
            num_additional_friction_iterations: config.position_iterations,
            ..IntegrationParameters::default()
        };

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            integration_parameters,
            config,
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_collider: HashMap::new(),
            collider_to_entity: HashMap::new(),
            collision_events: Vec::new(),
            previous_collisions: HashSet::new(),
        }
    }

    /// Get the physics configuration
    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }

    /// Get gravity
    pub fn gravity(&self) -> Vec2 {
        self.config.gravity
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.config.gravity = gravity;
    }

    /// Convert pixels to meters (vector)
    pub fn pixels_to_meters(&self, pixels: Vec2) -> Vec2 {
        pixels / self.config.pixels_per_meter
    }

    /// Convert a scalar from pixels to meters
    pub fn pixels_to_meters_scalar(&self, pixels: f32) -> f32 {
        pixels / self.config.pixels_per_meter
    }

    /// Convert meters to pixels (vector)
    pub fn meters_to_pixels(&self, meters: Vec2) -> Vec2 {
        meters * self.config.pixels_per_meter
    }

    /// Convert a scalar from meters to pixels
    pub fn meters_to_pixels_scalar(&self, meters: f32) -> f32 {
        meters * self.config.pixels_per_meter
    }

    /// Add a rigid body for an entity
    pub fn add_rigid_body(&mut self, entity: EntityId, body: &mut RigidBody, position: Vec2, rotation: f32) {
        // Remove existing body if any
        self.remove_rigid_body(entity);

        // Convert position from pixels to meters
        let pos = self.pixels_to_meters(position);
        let vel = self.pixels_to_meters(body.velocity);

        // Create rapier rigid body
        let rigid_body = match body.body_type {
            RigidBodyType::Dynamic => {
                let mut builder = RigidBodyBuilder::dynamic()
                    .translation(vector![pos.x, pos.y])
                    .rotation(rotation)
                    .linvel(vector![vel.x, vel.y])
                    .angvel(body.angular_velocity)
                    .gravity_scale(body.gravity_scale)
                    .linear_damping(body.linear_damping)
                    .angular_damping(body.angular_damping)
                    .ccd_enabled(body.ccd_enabled);
                if !body.can_rotate {
                    builder = builder.lock_rotations();
                }
                builder.build()
            }
            RigidBodyType::Static => RigidBodyBuilder::fixed()
                .translation(vector![pos.x, pos.y])
                .rotation(rotation)
                .build(),
            RigidBodyType::Kinematic => {
                RigidBodyBuilder::kinematic_position_based()
                    .translation(vector![pos.x, pos.y])
                    .rotation(rotation)
                    .linvel(vector![vel.x, vel.y])
                    .angvel(body.angular_velocity)
                    .ccd_enabled(body.ccd_enabled)
                    .build()
            }
        };

        let handle = self.rigid_body_set.insert(rigid_body);
        body.handle = Some(handle);

        self.entity_to_body.insert(entity, handle);
        self.body_to_entity.insert(handle, entity);

        log::trace!("Added rigid body for entity {:?}", entity);
    }

    /// Add a collider for an entity
    pub fn add_collider(
        &mut self,
        entity: EntityId,
        collider: &mut Collider,
        rigid_body: Option<&RigidBody>,
    ) {
        // Remove existing collider if any
        self.remove_collider(entity);

        let ppm = self.config.pixels_per_meter;

        // Create rapier shape
        let shape: SharedShape = match &collider.shape {
            ColliderShape::Box { half_extents } => {
                let he = *half_extents / ppm;
                SharedShape::cuboid(he.x, he.y)
            }
            ColliderShape::Circle { radius } => {
                SharedShape::ball(*radius / ppm)
            }
            ColliderShape::CapsuleY { half_height, radius } => SharedShape::capsule_y(
                *half_height / ppm,
                *radius / ppm,
            ),
            ColliderShape::CapsuleX { half_height, radius } => SharedShape::capsule_x(
                *half_height / ppm,
                *radius / ppm,
            ),
        };

        // Build collider
        let offset = collider.offset / ppm;
        let mut builder = ColliderBuilder::new(shape)
            .translation(vector![offset.x, offset.y])
            .friction(collider.friction)
            .restitution(collider.restitution)
            .sensor(collider.is_sensor)
            .active_events(ActiveEvents::COLLISION_EVENTS);

        // Set collision groups using InteractionGroups
        let groups = InteractionGroups::new(
            Group::from_bits_truncate(collider.collision_groups),
            Group::from_bits_truncate(collider.collision_filter),
        );
        builder = builder.collision_groups(groups);

        let rapier_collider = builder.build();

        // Insert collider, attached to rigid body if available
        let handle = if let Some(body) = rigid_body {
            if let Some(body_handle) = body.handle {
                self.collider_set.insert_with_parent(
                    rapier_collider,
                    body_handle,
                    &mut self.rigid_body_set,
                )
            } else {
                self.collider_set.insert(rapier_collider)
            }
        } else {
            self.collider_set.insert(rapier_collider)
        };

        collider.handle = Some(handle);
        self.entity_to_collider.insert(entity, handle);
        self.collider_to_entity.insert(handle, entity);

        log::trace!("Added collider for entity {:?}", entity);
    }

    /// Remove a rigid body for an entity
    pub fn remove_rigid_body(&mut self, entity: EntityId) {
        if let Some(handle) = self.entity_to_body.remove(&entity) {
            self.body_to_entity.remove(&handle);
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true, // remove colliders attached to this body
            );
            log::trace!("Removed rigid body for entity {:?}", entity);
        }
    }

    /// Remove a collider for an entity
    pub fn remove_collider(&mut self, entity: EntityId) {
        if let Some(handle) = self.entity_to_collider.remove(&entity) {
            self.collider_to_entity.remove(&handle);
            self.collider_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
            log::trace!("Removed collider for entity {:?}", entity);
        }
    }

    /// Remove all physics objects for an entity
    pub fn remove_entity(&mut self, entity: EntityId) {
        self.remove_rigid_body(entity);
        self.remove_collider(entity);
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;

        let ppm = self.config.pixels_per_meter;
        let gravity = vector![
            self.config.gravity.x / ppm,
            self.config.gravity.y / ppm
        ];

        // Clear previous collision events
        self.collision_events.clear();

        // Create event handler
        let event_handler = ();

        // Step physics
        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &event_handler,
        );

        // Build current collision set and process collision events
        let mut current_collisions = HashSet::new();

        // Process collision events from narrow phase
        for contact_pair in self.narrow_phase.contact_pairs() {
            let collider1 = contact_pair.collider1;
            let collider2 = contact_pair.collider2;

            if let (Some(&entity_a), Some(&entity_b)) = (
                self.collider_to_entity.get(&collider1),
                self.collider_to_entity.get(&collider2),
            ) {
                let has_contact = contact_pair.has_any_active_contact;

                if has_contact {
                    let pair = CollisionPair::new(entity_a, entity_b);
                    current_collisions.insert(pair);

                    // Check if this is a new collision (started)
                    let started = !self.previous_collisions.contains(&pair);
                    let contacts = self.get_contact_points_from_pair(contact_pair);

                    self.collision_events.push(CollisionData {
                        event: CollisionEvent {
                            entity_a,
                            entity_b,
                            started,
                            stopped: false,
                        },
                        contacts,
                    });
                }
            }
        }

        // Find collisions that ended (were in previous but not in current)
        for pair in &self.previous_collisions {
            if !current_collisions.contains(pair) {
                let (entity_a, entity_b) = pair.entities();
                self.collision_events.push(CollisionData {
                    event: CollisionEvent {
                        entity_a,
                        entity_b,
                        started: false,
                        stopped: true,
                    },
                    contacts: Vec::new(), // No contacts for ended collisions
                });
            }
        }

        // Update previous collisions for next frame
        self.previous_collisions = current_collisions;
    }

    /// Get contact points from a contact pair
    fn get_contact_points_from_pair(&self, contact_pair: &ContactPair) -> Vec<ContactPoint> {
        let mut contacts = Vec::new();
        let ppm = self.config.pixels_per_meter;

        for manifold in &contact_pair.manifolds {
            for point in &manifold.points {
                let world_point = manifold.local_n1 * point.dist + point.local_p1.coords;
                contacts.push(ContactPoint {
                    point: Vec2::new(world_point.x * ppm, world_point.y * ppm),
                    normal: Vec2::new(manifold.local_n1.x, manifold.local_n1.y),
                    depth: point.dist * ppm,
                });
            }
        }

        contacts
    }

    /// Get collision events from the last step
    pub fn collision_events(&self) -> &[CollisionData] {
        &self.collision_events
    }

    /// Get the position and rotation of a rigid body
    pub fn get_body_transform(&self, entity: EntityId) -> Option<(Vec2, f32)> {
        let handle = self.entity_to_body.get(&entity)?;
        let body = self.rigid_body_set.get(*handle)?;
        let translation = body.translation();
        let rotation = body.rotation().angle();
        let ppm = self.config.pixels_per_meter;

        Some((
            Vec2::new(translation.x * ppm, translation.y * ppm),
            rotation,
        ))
    }

    /// Get the velocity of a rigid body
    pub fn get_body_velocity(&self, entity: EntityId) -> Option<(Vec2, f32)> {
        let handle = self.entity_to_body.get(&entity)?;
        let body = self.rigid_body_set.get(*handle)?;
        let linvel = body.linvel();
        let angvel = body.angvel();
        let ppm = self.config.pixels_per_meter;

        Some((
            Vec2::new(linvel.x * ppm, linvel.y * ppm),
            angvel,
        ))
    }

    /// Set the position and rotation of a rigid body
    pub fn set_body_transform(&mut self, entity: EntityId, position: Vec2, rotation: f32) {
        let ppm = self.config.pixels_per_meter;
        let pos = position / ppm;

        if let Some(&handle) = self.entity_to_body.get(&entity) {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_translation(vector![pos.x, pos.y], true);
                body.set_rotation(nalgebra::UnitComplex::new(rotation), true);
            }
        }
    }

    /// Set the next kinematic position (for kinematic bodies)
    ///
    /// This is the proper way to move kinematic_position_based bodies.
    /// The body will move to this position during the next physics step,
    /// properly interacting with other bodies along the way.
    pub fn set_kinematic_target(&mut self, entity: EntityId, position: Vec2, rotation: f32) {
        let ppm = self.config.pixels_per_meter;
        let pos = position / ppm;

        if let Some(&handle) = self.entity_to_body.get(&entity) {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_next_kinematic_translation(vector![pos.x, pos.y]);
                body.set_next_kinematic_rotation(nalgebra::UnitComplex::new(rotation));
            }
        }
    }

    /// Set the velocity of a rigid body
    pub fn set_body_velocity(&mut self, entity: EntityId, linear: Vec2, angular: f32) {
        let ppm = self.config.pixels_per_meter;
        let vel = linear / ppm;

        if let Some(&handle) = self.entity_to_body.get(&entity) {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_linvel(vector![vel.x, vel.y], true);
                body.set_angvel(angular, true);
            }
        }
    }

    /// Apply an impulse to a rigid body
    pub fn apply_impulse(&mut self, entity: EntityId, impulse: Vec2) {
        let ppm = self.config.pixels_per_meter;
        let imp = impulse / ppm;

        if let Some(&handle) = self.entity_to_body.get(&entity) {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.apply_impulse(vector![imp.x, imp.y], true);
            }
        }
    }

    /// Apply a force to a rigid body
    pub fn apply_force(&mut self, entity: EntityId, force: Vec2) {
        let ppm = self.config.pixels_per_meter;
        let f = force / ppm;

        if let Some(&handle) = self.entity_to_body.get(&entity) {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.add_force(vector![f.x, f.y], true);
            }
        }
    }

    /// Cast a ray and return the first hit
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_distance: f32) -> Option<(EntityId, Vec2, f32)> {
        let ppm = self.config.pixels_per_meter;
        let origin_m = origin / ppm;
        let ray = Ray::new(
            point![origin_m.x, origin_m.y],
            vector![direction.x, direction.y],
        );
        let max_toi = max_distance / ppm;

        if let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            true,
            QueryFilter::default(),
        ) {
            if let Some(&entity) = self.collider_to_entity.get(&handle) {
                let hit_point = ray.point_at(toi);
                return Some((
                    entity,
                    Vec2::new(hit_point.x * ppm, hit_point.y * ppm),
                    toi * ppm,
                ));
            }
        }

        None
    }

    /// Check if an entity has a rigid body
    pub fn has_rigid_body(&self, entity: EntityId) -> bool {
        self.entity_to_body.contains_key(&entity)
    }

    /// Check if an entity has a collider
    pub fn has_collider(&self, entity: EntityId) -> bool {
        self.entity_to_collider.contains_key(&entity)
    }

    /// Get the number of rigid bodies
    pub fn rigid_body_count(&self) -> usize {
        self.rigid_body_set.len()
    }

    /// Get the number of colliders
    pub fn collider_count(&self) -> usize {
        self.collider_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let world = PhysicsWorld::default();
        assert_eq!(world.rigid_body_count(), 0);
        assert_eq!(world.collider_count(), 0);
    }

    #[test]
    fn test_add_rigid_body() {
        let mut world = PhysicsWorld::default();
        let entity = EntityId::new();
        let mut body = RigidBody::new_dynamic();

        world.add_rigid_body(entity, &mut body, Vec2::ZERO, 0.0);

        assert!(world.has_rigid_body(entity));
        assert_eq!(world.rigid_body_count(), 1);
        assert!(body.handle.is_some());
    }

    #[test]
    fn test_add_collider() {
        let mut world = PhysicsWorld::default();
        let entity = EntityId::new();
        let mut body = RigidBody::new_dynamic();
        let mut collider = Collider::box_collider(32.0, 32.0);

        world.add_rigid_body(entity, &mut body, Vec2::ZERO, 0.0);
        world.add_collider(entity, &mut collider, Some(&body));

        assert!(world.has_collider(entity));
        assert_eq!(world.collider_count(), 1);
        assert!(collider.handle.is_some());
    }

    #[test]
    fn test_remove_entity() {
        let mut world = PhysicsWorld::default();
        let entity = EntityId::new();
        let mut body = RigidBody::new_dynamic();
        let mut collider = Collider::box_collider(32.0, 32.0);

        world.add_rigid_body(entity, &mut body, Vec2::ZERO, 0.0);
        world.add_collider(entity, &mut collider, Some(&body));
        world.remove_entity(entity);

        assert!(!world.has_rigid_body(entity));
        assert!(!world.has_collider(entity));
        assert_eq!(world.rigid_body_count(), 0);
        assert_eq!(world.collider_count(), 0);
    }

    #[test]
    fn test_step_simulation() {
        let mut world = PhysicsWorld::default();
        let entity = EntityId::new();
        let mut body = RigidBody::new_dynamic();
        let mut collider = Collider::box_collider(32.0, 32.0);

        world.add_rigid_body(entity, &mut body, Vec2::new(0.0, 100.0), 0.0);
        world.add_collider(entity, &mut collider, Some(&body));

        // Step simulation
        world.step(1.0 / 60.0);

        // Body should have moved due to gravity
        if let Some((pos, _rotation)) = world.get_body_transform(entity) {
            assert!(pos.y < 100.0, "Body should fall due to gravity");
        }
    }

    #[test]
    fn test_raycast() {
        let mut world = PhysicsWorld::default();
        let entity = EntityId::new();
        let mut body = RigidBody::new_static();
        let mut collider = Collider::box_collider(100.0, 100.0);

        world.add_rigid_body(entity, &mut body, Vec2::new(200.0, 0.0), 0.0);
        world.add_collider(entity, &mut collider, Some(&body));

        // Update query pipeline
        world.step(0.0);

        // Raycast towards the box
        let result = world.raycast(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 500.0);
        assert!(result.is_some());

        let (hit_entity, _hit_point, distance) = result.unwrap();
        assert_eq!(hit_entity, entity);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_collision_started_event() {
        // Create world with no gravity so objects don't fall
        let config = PhysicsConfig::new(Vec2::ZERO);
        let mut world = PhysicsWorld::new(config);

        // Create two entities that will collide
        let entity_a = EntityId::new();
        let entity_b = EntityId::new();

        // Entity A: static floor
        let mut body_a = RigidBody::new_static();
        let mut collider_a = Collider::box_collider(200.0, 20.0);
        world.add_rigid_body(entity_a, &mut body_a, Vec2::new(0.0, 0.0), 0.0);
        world.add_collider(entity_a, &mut collider_a, Some(&body_a));

        // Entity B: dynamic box that will land on the floor
        let mut body_b = RigidBody::new_dynamic().with_gravity_scale(0.0);
        let mut collider_b = Collider::box_collider(20.0, 20.0);
        // Position slightly above but overlapping
        world.add_rigid_body(entity_b, &mut body_b, Vec2::new(0.0, 15.0), 0.0);
        world.add_collider(entity_b, &mut collider_b, Some(&body_b));

        // First step - collision should start
        world.step(1.0 / 60.0);

        let events = world.collision_events();
        assert!(!events.is_empty(), "Should have collision events");

        // Find the collision event between our entities
        let collision = events.iter().find(|e| {
            (e.event.entity_a == entity_a && e.event.entity_b == entity_b) ||
            (e.event.entity_a == entity_b && e.event.entity_b == entity_a)
        });

        assert!(collision.is_some(), "Should have collision between entities");
        let collision = collision.unwrap();
        assert!(collision.event.started, "Collision should be marked as started");
        assert!(!collision.event.stopped, "Collision should not be marked as stopped");
    }

    #[test]
    fn test_collision_ongoing_not_started() {
        // Create world with no gravity
        let config = PhysicsConfig::new(Vec2::ZERO);
        let mut world = PhysicsWorld::new(config);

        // Create two overlapping entities
        let entity_a = EntityId::new();
        let entity_b = EntityId::new();

        let mut body_a = RigidBody::new_static();
        let mut collider_a = Collider::box_collider(100.0, 100.0);
        world.add_rigid_body(entity_a, &mut body_a, Vec2::ZERO, 0.0);
        world.add_collider(entity_a, &mut collider_a, Some(&body_a));

        let mut body_b = RigidBody::new_dynamic().with_gravity_scale(0.0);
        let mut collider_b = Collider::box_collider(50.0, 50.0);
        world.add_rigid_body(entity_b, &mut body_b, Vec2::ZERO, 0.0);
        world.add_collider(entity_b, &mut collider_b, Some(&body_b));

        // First step - collision starts
        world.step(1.0 / 60.0);
        let events = world.collision_events();
        let first_collision = events.iter().find(|e| {
            (e.event.entity_a == entity_a && e.event.entity_b == entity_b) ||
            (e.event.entity_a == entity_b && e.event.entity_b == entity_a)
        });
        assert!(first_collision.is_some());
        assert!(first_collision.unwrap().event.started, "First frame should be started");

        // Second step - collision continues but shouldn't be marked as "started"
        world.step(1.0 / 60.0);
        let events = world.collision_events();
        let ongoing_collision = events.iter().find(|e| {
            (e.event.entity_a == entity_a && e.event.entity_b == entity_b) ||
            (e.event.entity_a == entity_b && e.event.entity_b == entity_a)
        });
        assert!(ongoing_collision.is_some());
        assert!(!ongoing_collision.unwrap().event.started, "Ongoing collision should not be marked as started");
        assert!(!ongoing_collision.unwrap().event.stopped, "Ongoing collision should not be marked as stopped");
    }

    #[test]
    fn test_collision_stopped_event() {
        // Create world with no gravity
        let config = PhysicsConfig::new(Vec2::ZERO);
        let mut world = PhysicsWorld::new(config);

        // Create two overlapping entities
        let entity_a = EntityId::new();
        let entity_b = EntityId::new();

        let mut body_a = RigidBody::new_static();
        let mut collider_a = Collider::box_collider(50.0, 50.0);
        world.add_rigid_body(entity_a, &mut body_a, Vec2::ZERO, 0.0);
        world.add_collider(entity_a, &mut collider_a, Some(&body_a));

        let mut body_b = RigidBody::new_dynamic().with_gravity_scale(0.0);
        let mut collider_b = Collider::box_collider(50.0, 50.0);
        world.add_rigid_body(entity_b, &mut body_b, Vec2::new(10.0, 0.0), 0.0);
        world.add_collider(entity_b, &mut collider_b, Some(&body_b));

        // First step - collision starts
        world.step(1.0 / 60.0);
        assert!(!world.collision_events().is_empty(), "Should have collision");

        // Move entity_b far away to end the collision
        world.set_body_transform(entity_b, Vec2::new(500.0, 0.0), 0.0);

        // Step again - collision should end
        world.step(1.0 / 60.0);
        let events = world.collision_events();

        // Find the stopped collision event
        let stopped_collision = events.iter().find(|e| {
            e.event.stopped &&
            ((e.event.entity_a == entity_a && e.event.entity_b == entity_b) ||
             (e.event.entity_a == entity_b && e.event.entity_b == entity_a))
        });

        assert!(stopped_collision.is_some(), "Should have a stopped collision event");
        let stopped = stopped_collision.unwrap();
        assert!(!stopped.event.started, "Stopped event should not be marked as started");
        assert!(stopped.event.stopped, "Stopped event should be marked as stopped");
    }

    #[test]
    fn test_collision_pair_canonical_order() {
        let entity_a = EntityId::new();
        let entity_b = EntityId::new();

        // Both orderings should produce the same pair
        let pair1 = CollisionPair::new(entity_a, entity_b);
        let pair2 = CollisionPair::new(entity_b, entity_a);

        assert_eq!(pair1, pair2, "CollisionPair should be order-independent");

        // The entities method should return consistent results
        let (e1, e2) = pair1.entities();
        let (e3, e4) = pair2.entities();
        assert_eq!(e1, e3);
        assert_eq!(e2, e4);
    }
}
