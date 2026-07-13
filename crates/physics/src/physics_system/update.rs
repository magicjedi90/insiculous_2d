//! The `System` trait implementation: fixed-timestep driver and event emission.

use std::collections::HashSet;

use glam::Vec2;

use ecs::{EntityId, System, World};

use super::{DeferredBodyOp, PhysicsSystem, MAX_STEPS_PER_UPDATE};

impl System for PhysicsSystem {
    fn initialize(&mut self, _world: &mut World) -> Result<(), String> {
        log::info!("PhysicsSystem initialized with gravity: {:?}", self.physics_world.gravity());
        Ok(())
    }

    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Clamp delta time to prevent instability
        let dt = delta_time.min(self.max_delta_time);

        // Get all entities, garbage-collect physics state for entities no
        // longer in the ECS, and sync new ones to physics.
        let entities: Vec<EntityId> = world.entities();
        let alive: HashSet<EntityId> = entities.iter().copied().collect();
        self.prune_removed_entities(&alive);

        for entity in entities {
            self.sync_entity_to_physics(world, entity);
        }

        // Flush deferred body ops in call order: the documented "reset then
        // launch" pattern works because the launch velocity is queued after
        // the reset that would otherwise zero it.
        for (entity, op) in self.pending_ops.drain(..) {
            match op {
                DeferredBodyOp::Reset { position } => {
                    self.physics_world.set_body_transform(entity, position, 0.0);
                    self.physics_world.set_velocity(entity, Vec2::ZERO, 0.0);
                }
                DeferredBodyOp::SetVelocity { linear, angular } => {
                    self.physics_world.set_velocity(entity, linear, angular);
                }
            }
        }

        // Fixed timestep physics updates, capped to avoid a death spiral
        // where catch-up steps make the frame even slower.
        self.time_accumulator += dt;

        // Clear last frame's events once, before stepping. Each step()
        // APPENDS its events, so multiple sub-steps all contribute and a
        // frame with zero steps emits nothing (no stale re-delivery).
        self.physics_world.clear_collision_events();

        let mut steps = 0;
        while self.time_accumulator >= self.fixed_timestep && steps < MAX_STEPS_PER_UPDATE {
            self.physics_world.step(self.fixed_timestep);
            self.time_accumulator -= self.fixed_timestep;
            steps += 1;
        }
        if steps == MAX_STEPS_PER_UPDATE && self.time_accumulator >= self.fixed_timestep {
            log::warn!(
                "Physics fell behind: dropping {:.3}s of accumulated time after {} steps",
                self.time_accumulator,
                steps
            );
            self.time_accumulator = 0.0;
        }

        // Forces from apply_force() last one update. Skip the reset on
        // zero-step frames so a force applied then still acts on the next
        // frame that actually steps.
        if steps > 0 {
            self.physics_world.reset_forces();
        }

        // Sync physics results back to ECS
        self.sync_physics_to_ecs(world);

        // Emit collision events to the world event bus (available to any
        // system). The buffer itself stays available for game code to drain
        // via `take_collision_events()` after this update returns.
        let events = self.physics_world.collision_events();
        for collision in events {
            world.emit_event(collision.clone());
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
