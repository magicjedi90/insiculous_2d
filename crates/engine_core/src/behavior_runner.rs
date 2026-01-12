//! Behavior runner for processing entity behaviors
//!
//! This module provides the `BehaviorRunner` which processes all entities
//! that have `Behavior` components, handling input-driven movement,
//! AI behaviors, and other game logic.

use std::collections::HashMap;

use glam::Vec2;

use ecs::behavior::{Behavior, BehaviorState, EntityTag};
use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};
use input::GameAction;
use input::InputHandler;
use physics::{PhysicsSystem, RigidBody, RigidBodyType};

/// Processes behavior components for all entities.
///
/// The `BehaviorRunner` iterates over all entities with `Behavior` components
/// and executes the appropriate behavior logic. This should be called from
/// `Game::update()` where input is available.
///
/// # Example
///
/// ```ignore
/// struct MyGame {
///     behaviors: BehaviorRunner,
///     physics: Option<PhysicsSystem>,
/// }
///
/// impl Game for MyGame {
///     fn update(&mut self, ctx: &mut GameContext) {
///         self.behaviors.update(
///             &mut ctx.world,
///             ctx.input,
///             ctx.delta_time,
///             self.physics.as_mut(),
///         );
///     }
/// }
/// ```
pub struct BehaviorRunner {
    /// Named entity lookup (populated from SceneInstance)
    named_entities: HashMap<String, EntityId>,
}

impl Default for BehaviorRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorRunner {
    /// Create a new behavior runner
    pub fn new() -> Self {
        Self {
            named_entities: HashMap::new(),
        }
    }

    /// Set named entities from a SceneInstance
    ///
    /// This allows behaviors like `FollowEntity` to reference entities by name.
    pub fn set_named_entities(&mut self, named: HashMap<String, EntityId>) {
        self.named_entities = named;
    }

    /// Update all entities with behaviors
    ///
    /// This processes all entities that have a `Behavior` component,
    /// executing the appropriate behavior logic for each.
    ///
    /// Should be called from `Game::update()` before physics simulation.
    pub fn update(
        &mut self,
        world: &mut World,
        input: &InputHandler,
        delta_time: f32,
        mut physics: Option<&mut PhysicsSystem>,
    ) {
        // Track entities to despawn after processing
        let mut to_despawn: Vec<EntityId> = Vec::new();

        // Collect velocity commands to apply after iteration (avoids borrow conflicts)
        let mut velocity_commands: Vec<(EntityId, Vec2)> = Vec::new();

        // Collect impulse commands to apply AFTER velocity commands
        let mut impulse_commands: Vec<(EntityId, Vec2)> = Vec::new();

        // Collect tag assignments to apply after iteration
        let mut tag_assignments: Vec<(EntityId, String)> = Vec::new();

        // Process all entities with behaviors directly - avoid cloning
        for entity in world.entities() {
            // Get behavior component by reference to avoid cloning
            if let Some(behavior) = world.get::<Behavior>(entity) {
                // Clone state only when needed (much smaller than Behavior)
                let mut state = world
                    .get::<BehaviorState>(entity)
                    .cloned()
                    .unwrap_or_default();

                match behavior {
                    Behavior::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
                        // Update cooldown timer
                        if state.timer > 0.0 {
                            state.timer -= delta_time;
                        }

                        // Calculate horizontal velocity only - let physics handle Y (gravity + jumps)
                        let mut vel_x = 0.0;
                        if input.is_action_active(&GameAction::MoveLeft) { vel_x = -move_speed; }
                        if input.is_action_active(&GameAction::MoveRight) { vel_x = *move_speed; }

                        // For platformers, only set X velocity - preserve Y for physics
                        if let Some(ref physics) = physics {
                            let current_vel = physics.physics_world()
                                .get_body_velocity(entity)
                                .map(|(v, _)| v)
                                .unwrap_or(Vec2::ZERO);
                            // Set X to input, keep Y from physics (gravity/jumps)
                            let vel = Vec2::new(vel_x, current_vel.y);
                            velocity_commands.push((entity, vel));
                        }

                        // Jump - collect impulse to apply AFTER velocity commands
                        if input.is_key_just_pressed(winit::keyboard::KeyCode::Space) && state.timer <= 0.0 {
                            impulse_commands.push((entity, Vec2::new(0.0, *jump_impulse)));
                            state.timer = *jump_cooldown;
                        }

                        tag_assignments.push((entity, tag.clone()));
                        Self::update_state(world, entity, state);
                }

                Behavior::PlayerTopDown { move_speed, tag } => {
                        // Calculate movement velocity from input
                        let mut vel = Vec2::ZERO;
                        if input.is_action_active(&GameAction::MoveUp) { vel.y += *move_speed; }
                        if input.is_action_active(&GameAction::MoveDown) { vel.y -= *move_speed; }
                        if input.is_action_active(&GameAction::MoveLeft) { vel.x -= *move_speed; }
                        if input.is_action_active(&GameAction::MoveRight) { vel.x += *move_speed; }

                        // Normalize diagonal movement
                        if vel.length_squared() > 0.0 {
                            vel = vel.normalize() * *move_speed;
                        }

                        velocity_commands.push((entity, vel));
                        tag_assignments.push((entity, tag.clone()));
                }

                Behavior::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
                        if let Some(target_pos) = Self::find_nearest_tagged_position(world, entity, target_tag) {
                            if let Some(entity_pos) = Self::get_position(world, entity) {
                                let distance = (target_pos - entity_pos).length();

                                // Update chase state
                                if !state.is_chasing && distance < *detection_range {
                                    state.is_chasing = true;
                                } else if state.is_chasing && distance > *lose_interest_range {
                                    state.is_chasing = false;
                                }

                                if state.is_chasing {
                                    let vel = (target_pos - entity_pos).normalize_or_zero() * *chase_speed;
                                    velocity_commands.push((entity, vel));
                                } else {
                                    velocity_commands.push((entity, Vec2::ZERO));
                                }
                            }
                        } else {
                            state.is_chasing = false;
                            velocity_commands.push((entity, Vec2::ZERO));
                        }
                        Self::update_state(world, entity, state);
                }

                Behavior::Patrol { point_a, point_b, speed, wait_time } => {
                        let pt_a = Vec2::new(point_a.0, point_a.1);
                        let pt_b = Vec2::new(point_b.0, point_b.1);

                        if state.is_waiting {
                            state.timer -= delta_time;
                            if state.timer <= 0.0 {
                                state.is_waiting = false;
                                state.patrol_toward_b = !state.patrol_toward_b;
                            }
                            velocity_commands.push((entity, Vec2::ZERO));
                        } else if let Some(entity_pos) = Self::get_position(world, entity) {
                            let target = if state.patrol_toward_b { pt_b } else { pt_a };

                            if (target - entity_pos).length() < 5.0 {
                                state.is_waiting = true;
                                state.timer = *wait_time;
                                velocity_commands.push((entity, Vec2::ZERO));
                            } else {
                                let vel = (target - entity_pos).normalize() * *speed;
                                velocity_commands.push((entity, vel));
                            }
                        }
                        Self::update_state(world, entity, state);
                }

                Behavior::FollowEntity { target_name, follow_distance, follow_speed } => {
                        let mut vel = Vec2::ZERO;
                        if let Some(&target_entity) = self.named_entities.get(target_name) {
                            if let (Some(target_pos), Some(entity_pos)) = (
                                Self::get_position(world, target_entity),
                                Self::get_position(world, entity),
                            ) {
                                let to_target = target_pos - entity_pos;
                                if to_target.length() > *follow_distance {
                                    vel = to_target.normalize() * *follow_speed;
                                }
                            }
                        }
                        velocity_commands.push((entity, vel));
                }

                Behavior::FollowTagged { target_tag, follow_distance, follow_speed } => {
                        let mut vel = Vec2::ZERO;
                        if let Some(target_pos) = Self::find_nearest_tagged_position(world, entity, target_tag) {
                            if let Some(entity_pos) = Self::get_position(world, entity) {
                                let to_target = target_pos - entity_pos;
                                if to_target.length() > *follow_distance {
                                    vel = to_target.normalize() * *follow_speed;
                                }
                            }
                        }
                        velocity_commands.push((entity, vel));
                }

                Behavior::Collectible { score_value, despawn_on_collect, collector_tag } => {
                        if Self::check_tagged_overlap(world, entity, collector_tag, 40.0) {
                            log::info!("Collected! +{} points", score_value);
                            if *despawn_on_collect {
                                to_despawn.push(entity);
                            }
                        }
                }
            }
            }
        }

        // Apply tag assignments
        for (entity, tag) in tag_assignments {
            Self::add_entity_tag(world, entity, &tag);
        }

        // Apply velocity commands (either via physics or direct transform)
        if let Some(ref mut physics) = physics {
            for (entity, vel) in velocity_commands {
                // Check if entity is kinematic - kinematic bodies need position-based movement
                let is_kinematic = world
                    .get::<RigidBody>(entity)
                    .map(|rb| rb.body_type == RigidBodyType::Kinematic)
                    .unwrap_or(false);

                if is_kinematic {
                    // Kinematic bodies: use set_kinematic_target for proper physics interaction
                    if let Some(current_pos) = Self::get_position(world, entity) {
                        let new_pos = current_pos + vel * delta_time;
                        physics.physics_world_mut().set_kinematic_target(entity, new_pos, 0.0);
                    }
                } else {
                    // Dynamic bodies: set velocity and let physics handle movement
                    physics.physics_world_mut().set_body_velocity(entity, vel, 0.0);
                }
            }
        } else {
            // Fallback: direct transform modification (no physics)
            for (entity, vel) in velocity_commands {
                if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                    transform.position += vel * delta_time;
                }
            }
        }

        // Apply impulses AFTER velocity commands (so jumps aren't overwritten)
        if let Some(ref mut physics) = physics {
            for (entity, impulse) in impulse_commands {
                physics.apply_impulse(entity, impulse);
            }
        }

        // Remove collected entities
        for entity in to_despawn {
            let _ = world.remove_entity(&entity);
        }
    }

    /// Find the position of the nearest entity with a specific tag (excluding self)
    fn find_nearest_tagged_position(world: &World, exclude: EntityId, tag: &str) -> Option<Vec2> {
        let exclude_pos = Self::get_position(world, exclude)?;

        world.entities().into_iter()
            .filter(|e| *e != exclude)
            .filter(|e| world.get::<EntityTag>(*e).map(|t| t.matches(tag)).unwrap_or(false))
            .filter_map(|e| Self::get_position(world, e))
            .min_by(|a, b| {
                let dist_a = (*a - exclude_pos).length_squared();
                let dist_b = (*b - exclude_pos).length_squared();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Check if any entity with a specific tag overlaps with the given entity
    fn check_tagged_overlap(world: &World, entity: EntityId, tag: &str, radius: f32) -> bool {
        let Some(entity_pos) = Self::get_position(world, entity) else { return false };

        world.entities().into_iter()
            .filter(|e| *e != entity)
            .filter(|e| world.get::<EntityTag>(*e).map(|t| t.matches(tag)).unwrap_or(false))
            .filter_map(|e| Self::get_position(world, e))
            .any(|pos| (pos - entity_pos).length() < radius)
    }

    /// Get entity position (common operation)
    fn get_position(world: &World, entity: EntityId) -> Option<Vec2> {
        world.get::<Transform2D>(entity).map(|t| t.position)
    }

    /// Add or update entity tag
    fn add_entity_tag(world: &mut World, entity: EntityId, tag: &str) {
        if let Some(existing) = world.get::<EntityTag>(entity) {
            if !existing.matches(tag) {
                // Update tag if different
                let _ = world.add_component(&entity, EntityTag::new(tag));
            }
        } else {
            let _ = world.add_component(&entity, EntityTag::new(tag));
        }
    }

    /// Update or add BehaviorState for an entity
    fn update_state(world: &mut World, entity: EntityId, state: BehaviorState) {
        if let Some(existing) = world.get_mut::<BehaviorState>(entity) {
            *existing = state;
        } else {
            let _ = world.add_component(&entity, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_runner_creation() {
        let runner = BehaviorRunner::new();
        assert!(runner.named_entities.is_empty());
    }

    #[test]
    fn test_set_named_entities() {
        let mut runner = BehaviorRunner::new();
        let mut named = HashMap::new();
        named.insert("player".to_string(), EntityId::with_generation(1, 1));
        runner.set_named_entities(named);
        assert!(runner.named_entities.contains_key("player"));
    }
}
