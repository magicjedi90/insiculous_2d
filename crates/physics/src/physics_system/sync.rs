//! ECS ↔ rapier synchronization and orphaned-state garbage collection.
//!
//! Sync direction matters: `sync_entity_to_physics` only ADDS missing bodies
//! and colliders to rapier — once a body exists, rapier is authoritative and
//! `sync_physics_to_ecs` writes positions/velocities back into the ECS
//! components every frame. Editing `Transform2D` directly on a live physics
//! entity has no effect; use `PhysicsSystem::set_body_transform`.

use std::collections::HashSet;

use glam::Vec2;

use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};

use crate::components::{Collider, RigidBody, RigidBodyType};

use super::PhysicsSystem;

impl PhysicsSystem {
    /// Garbage-collect physics state for entities that were removed from
    /// the ECS directly (`world.remove_entity`) without going through
    /// `destroy_entity` — otherwise their rapier bodies keep simulating
    /// and colliding invisibly forever. Also prunes pending deferred
    /// velocities/resets for those entities.
    pub(super) fn prune_removed_entities(&mut self, alive: &HashSet<EntityId>) {
        for entity in self.physics_world.tracked_entities() {
            if !alive.contains(&entity) {
                log::debug!(
                    "Removing orphaned physics state for despawned entity {:?}",
                    entity
                );
                self.physics_world.remove_entity(entity);
            }
        }
        self.pending_velocities.retain(|(e, _, _)| alive.contains(e));
        self.pending_resets.retain(|(e, _)| alive.contains(e));
    }

    /// Sync a single entity from ECS to physics world
    pub(super) fn sync_entity_to_physics(&mut self, world: &mut World, entity: EntityId) {
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
    pub(super) fn sync_physics_to_ecs(&self, world: &mut World) {
        let entities: Vec<EntityId> = world.entities();

        for entity in entities {
            // Get body type first to avoid borrow conflicts
            let body_type = world.get::<RigidBody>(entity).map(|b| b.body_type);

            if let Some(body_type) = body_type {
                // Sync both Dynamic and Kinematic bodies back to ECS
                // Static bodies don't move, so no need to sync them
                if body_type == RigidBodyType::Dynamic || body_type == RigidBodyType::Kinematic {
                    // Get physics transform
                    if let Some((position, rotation)) = self.physics_world.get_body_transform(entity) {
                        // Update ECS transform
                        if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                            transform.position = position;
                            transform.rotation = rotation;
                        }
                    }

                    // Update velocity in component (for dynamic bodies)
                    if body_type == RigidBodyType::Dynamic {
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
