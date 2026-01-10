//! Behavior runner for processing entity behaviors
//!
//! This module provides the `BehaviorRunner` which processes all entities
//! that have `Behavior` components, handling input-driven movement,
//! AI behaviors, and other game logic.

use std::collections::HashMap;

use glam::Vec2;

use ecs::behavior::{Behavior, BehaviorState, PlayerTag};
use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};
use input::GameAction;
use input::InputHandler;
use physics::PhysicsSystem;

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
        physics: Option<&mut PhysicsSystem>,
    ) {
        // Collect entities with behaviors to avoid borrow conflicts
        let entities_with_behaviors: Vec<(EntityId, Behavior, BehaviorState)> = world
            .entities()
            .into_iter()
            .filter_map(|entity| {
                world.get::<Behavior>(entity).cloned().map(|behavior| {
                    let state = world
                        .get::<BehaviorState>(entity)
                        .cloned()
                        .unwrap_or_default();
                    (entity, behavior, state)
                })
            })
            .collect();

        // Process player platformer behaviors (need physics)
        if let Some(physics) = physics {
            for (entity, behavior, state) in &entities_with_behaviors {
                let mut state = state.clone();
                if let Behavior::PlayerPlatformer {
                    move_speed,
                    jump_impulse,
                    jump_cooldown,
                } = behavior
                {
                    // Update cooldown timer
                    if state.timer > 0.0 {
                        state.timer -= delta_time;
                    }

                    // Horizontal movement using GameAction (remappable)
                    let mut target_vel_x = 0.0;
                    if input.is_action_active(&GameAction::MoveLeft) {
                        target_vel_x = -move_speed;
                    }
                    if input.is_action_active(&GameAction::MoveRight) {
                        target_vel_x = *move_speed;
                    }

                    // Get current velocity
                    let current_vel = physics
                        .physics_world()
                        .get_body_velocity(*entity)
                        .map(|(v, _)| v)
                        .unwrap_or(Vec2::ZERO);

                    // Set horizontal velocity, preserve vertical
                    let new_vel = Vec2::new(target_vel_x, current_vel.y);
                    physics
                        .physics_world_mut()
                        .set_body_velocity(*entity, new_vel, 0.0);

                    // Jump (Action1 = Space by default)
                    if input.is_action_active(&GameAction::Action1) && state.timer <= 0.0 {
                        physics.apply_impulse(*entity, Vec2::new(0.0, *jump_impulse));
                        state.timer = *jump_cooldown;
                    }

                    // Mark entity as player for AI behaviors
                    Self::mark_as_player(world, *entity);

                    // Update state
                    Self::update_state(world, *entity, state);
                }
            }
        }

        // Process non-physics behaviors
        for (entity, behavior, mut state) in entities_with_behaviors {
            match &behavior {
                Behavior::PlayerPlatformer { .. } => {
                    // Already handled above with physics
                }
                Behavior::PlayerTopDown { move_speed } => {
                    self.run_player_topdown(entity, *move_speed, world, input, delta_time);
                }
                Behavior::ChasePlayer {
                    detection_range,
                    chase_speed,
                    lose_interest_range,
                } => {
                    self.run_chase_player(
                        entity,
                        *detection_range,
                        *chase_speed,
                        *lose_interest_range,
                        &mut state,
                        world,
                        delta_time,
                    );
                }
                Behavior::Patrol {
                    point_a,
                    point_b,
                    speed,
                    wait_time,
                } => {
                    self.run_patrol(
                        entity,
                        Vec2::new(point_a.0, point_a.1),
                        Vec2::new(point_b.0, point_b.1),
                        *speed,
                        *wait_time,
                        &mut state,
                        world,
                        delta_time,
                    );
                }
                Behavior::FollowEntity {
                    target_name,
                    follow_distance,
                    follow_speed,
                } => {
                    self.run_follow_entity(
                        entity,
                        target_name,
                        *follow_distance,
                        *follow_speed,
                        world,
                        delta_time,
                    );
                }
                Behavior::Collectible { .. } => {
                    // Collectibles are handled by collision events, not update logic
                }
            }

            // Update behavior state (skip platformer - already handled above)
            if !matches!(behavior, Behavior::PlayerPlatformer { .. }) {
                Self::update_state(world, entity, state);
            }
        }
    }

    /// Player top-down behavior (movement in all directions using GameAction)
    fn run_player_topdown(
        &self,
        entity: EntityId,
        move_speed: f32,
        world: &mut World,
        input: &InputHandler,
        delta_time: f32,
    ) {
        let mut velocity = Vec2::ZERO;

        // Use GameAction for remappable controls (WASD/Arrow keys by default)
        if input.is_action_active(&GameAction::MoveUp) {
            velocity.y += move_speed;
        }
        if input.is_action_active(&GameAction::MoveDown) {
            velocity.y -= move_speed;
        }
        if input.is_action_active(&GameAction::MoveLeft) {
            velocity.x -= move_speed;
        }
        if input.is_action_active(&GameAction::MoveRight) {
            velocity.x += move_speed;
        }

        // Normalize diagonal movement
        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * move_speed;
        }

        // Update transform directly (top-down typically doesn't use physics gravity)
        if let Some(transform) = world.get_mut::<Transform2D>(entity) {
            transform.position += velocity * delta_time;
        }

        // Mark entity as player for AI behaviors
        Self::mark_as_player(world, entity);
    }

    /// Chase player AI behavior
    fn run_chase_player(
        &self,
        entity: EntityId,
        detection_range: f32,
        chase_speed: f32,
        lose_interest_range: f32,
        state: &mut BehaviorState,
        world: &mut World,
        delta_time: f32,
    ) {
        let Some(player_pos) = self.find_player_position(world) else {
            state.is_chasing = false;
            return;
        };

        let Some(entity_pos) = Self::get_position(world, entity) else { return };
        let distance = (player_pos - entity_pos).length();

        // Update chase state based on distance
        if !state.is_chasing && distance < detection_range {
            state.is_chasing = true;
        } else if state.is_chasing && distance > lose_interest_range {
            state.is_chasing = false;
        }

        if state.is_chasing {
            Self::move_toward(world, entity, player_pos, chase_speed, delta_time);
        }
    }

    /// Patrol behavior (back and forth between two points)
    fn run_patrol(
        &self,
        entity: EntityId,
        point_a: Vec2,
        point_b: Vec2,
        speed: f32,
        wait_time: f32,
        state: &mut BehaviorState,
        world: &mut World,
        delta_time: f32,
    ) {
        // Handle waiting at patrol points
        if state.is_waiting {
            state.timer -= delta_time;
            if state.timer <= 0.0 {
                state.is_waiting = false;
                state.patrol_toward_b = !state.patrol_toward_b;
            }
            return;
        }

        let target = if state.patrol_toward_b { point_b } else { point_a };
        let Some(entity_pos) = Self::get_position(world, entity) else { return };

        // Check if reached target (within 5 units)
        if (target - entity_pos).length() < 5.0 {
            state.is_waiting = true;
            state.timer = wait_time;
            return;
        }

        Self::move_toward(world, entity, target, speed, delta_time);
    }

    /// Follow entity behavior
    fn run_follow_entity(
        &self,
        entity: EntityId,
        target_name: &str,
        follow_distance: f32,
        follow_speed: f32,
        world: &mut World,
        delta_time: f32,
    ) {
        let Some(&target_entity) = self.named_entities.get(target_name) else { return };
        let Some(target_pos) = Self::get_position(world, target_entity) else { return };
        let Some(entity_pos) = Self::get_position(world, entity) else { return };

        // Only move if outside follow distance
        if (target_pos - entity_pos).length() > follow_distance {
            Self::move_toward(world, entity, target_pos, follow_speed, delta_time);
        }
    }

    /// Find the position of a player entity
    fn find_player_position(&self, world: &World) -> Option<Vec2> {
        world.entities().into_iter()
            .filter(|e| world.get::<PlayerTag>(*e).is_some())
            .find_map(|e| world.get::<Transform2D>(e).map(|t| t.position))
    }

    /// Get entity position (common operation)
    fn get_position(world: &World, entity: EntityId) -> Option<Vec2> {
        world.get::<Transform2D>(entity).map(|t| t.position)
    }

    /// Move entity toward a target position at given speed
    fn move_toward(world: &mut World, entity: EntityId, target: Vec2, speed: f32, delta_time: f32) {
        let Some(entity_pos) = Self::get_position(world, entity) else { return };
        let to_target = target - entity_pos;
        if to_target.length_squared() < 0.01 { return; } // Already there

        let velocity = to_target.normalize() * speed;
        if let Some(transform) = world.get_mut::<Transform2D>(entity) {
            transform.position += velocity * delta_time;
        }
    }

    /// Mark entity as player-controlled (for AI targeting)
    fn mark_as_player(world: &mut World, entity: EntityId) {
        if world.get::<PlayerTag>(entity).is_none() {
            let _ = world.add_component(&entity, PlayerTag);
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
