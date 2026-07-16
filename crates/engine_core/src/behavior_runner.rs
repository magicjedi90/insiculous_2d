//! Behavior runner for processing entity behaviors
//!
//! This module provides the `BehaviorRunner` which processes all entities
//! that have `Behavior` components, handling input-driven movement,
//! AI behaviors, and other game logic.

use std::collections::HashMap;

use glam::Vec2;

use ecs::behavior::{Behavior, BehaviorPhase, BehaviorState, EntityTag, PatrolTarget};
use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};
use input::{GameAction, InputHandler, InputMapping};
use physics::{PhysicsSystem, RigidBody, RigidBodyType};

/// Event emitted when a collectible entity is picked up.
/// Read these via `world.read_events::<EntityCollected>()` to update
/// score, play sounds, spawn particles, etc.
#[derive(Debug, Clone)]
pub struct EntityCollected {
    /// The collectible entity that was collected (may already be despawned)
    pub entity: EntityId,
    /// The score value of the collectible
    pub score_value: u32,
    /// The collector entity's tag
    pub collector_tag: String,
}

/// Commands collected while iterating behaviors, applied after the loop to
/// avoid borrow conflicts with the world.
#[derive(Default)]
struct BehaviorCommands {
    /// Entities to despawn after processing
    to_despawn: Vec<EntityId>,
    /// Velocity commands (applied via physics or direct transform)
    velocities: Vec<(EntityId, Vec2)>,
    /// Absolute position sets (direct transform write, bypasses physics —
    /// used by CameraFollow, whose entities carry no RigidBody)
    positions: Vec<(EntityId, Vec2)>,
    /// Impulse commands, applied AFTER velocity commands
    impulses: Vec<(EntityId, Vec2)>,
    /// Tag assignments
    tags: Vec<(EntityId, String)>,
    /// Collection events to emit
    collected: Vec<EntityCollected>,
}

/// Processes behavior components for all entities.
///
/// The `BehaviorRunner` iterates over all entities with `Behavior` components
/// and executes the appropriate behavior logic. This should be called from
/// `Game::update()` where input is available.
///
/// # Example
///
/// ```
/// use engine_core::prelude::*;
///
/// struct MyGame {
///     behaviors: BehaviorRunner,
///     physics: Option<PhysicsSystem>,
/// }
///
/// impl Game for MyGame {
///     fn update(&mut self, ctx: &mut GameContext) {
///         self.behaviors.update(
///             ctx.world,
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
    /// Action bindings for input-driven behaviors (engine default preset)
    actions: InputMapping<GameAction>,
}

impl Default for BehaviorRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorRunner {
    /// Create a new behavior runner with the engine's default action bindings
    pub fn new() -> Self {
        Self {
            named_entities: HashMap::new(),
            actions: InputMapping::with_default_bindings(),
        }
    }

    /// Get a mutable reference to the action bindings used by input-driven
    /// behaviors (e.g. to rebind `PlayerControlled` movement keys)
    pub fn actions_mut(&mut self) -> &mut InputMapping<GameAction> {
        &mut self.actions
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
        // Commands are collected during iteration and applied afterwards
        // (avoids borrow conflicts with the world).
        let mut commands = BehaviorCommands::default();

        // Process all entities with behaviors directly - avoid cloning
        for entity in world.entities() {
            // Get behavior component by reference to avoid cloning
            let Some(behavior) = world.get::<Behavior>(entity) else { continue };

            // Clone state only when needed (much smaller than Behavior)
            let mut state = world
                .get::<BehaviorState>(entity)
                .cloned()
                .unwrap_or_default();

            match behavior {
                Behavior::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
                    self.update_player_platformer(
                        entity, input, delta_time, physics.as_deref(),
                        *move_speed, *jump_impulse, *jump_cooldown, tag,
                        &mut state, &mut commands,
                    );
                    Self::update_state(world, entity, state);
                }

                Behavior::PlayerTopDown { move_speed, tag } => {
                    self.update_player_top_down(entity, input, *move_speed, tag, &mut commands);
                }

                Behavior::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
                    Self::update_chase_tagged(
                        world, entity, delta_time, target_tag,
                        *detection_range, *chase_speed, *lose_interest_range,
                        &mut state, &mut commands,
                    );
                    Self::update_state(world, entity, state);
                }

                Behavior::Patrol { point_a, point_b, speed, wait_time } => {
                    Self::update_patrol(
                        world, entity, delta_time,
                        Vec2::new(point_a.0, point_a.1), Vec2::new(point_b.0, point_b.1),
                        *speed, *wait_time,
                        &mut state, &mut commands,
                    );
                    Self::update_state(world, entity, state);
                }

                Behavior::FollowEntity { target_name, follow_distance, follow_speed } => {
                    self.update_follow_entity(
                        world, entity, target_name, *follow_distance, *follow_speed, &mut commands,
                    );
                }

                Behavior::FollowTagged { target_tag, follow_distance, follow_speed } => {
                    Self::update_follow_tagged(
                        world, entity, target_tag, *follow_distance, *follow_speed, &mut commands,
                    );
                }

                Behavior::Collectible { score_value, despawn_on_collect, collector_tag } => {
                    Self::update_collectible(
                        world, entity, *score_value, *despawn_on_collect, collector_tag, &mut commands,
                    );
                }

                Behavior::CameraFollow { target_tag, lerp_speed, offset, dead_zone } => {
                    Self::update_camera_follow(
                        world, entity, delta_time, target_tag,
                        *lerp_speed, *offset, *dead_zone, &mut commands,
                    );
                }
            }
        }

        Self::apply_commands(world, physics, delta_time, commands);
    }

    /// `Behavior::PlayerPlatformer` — input-driven horizontal movement plus a
    /// cooldown-gated jump impulse; Y velocity stays with physics (gravity).
    #[allow(clippy::too_many_arguments)]
    fn update_player_platformer(
        &self,
        entity: EntityId,
        input: &InputHandler,
        delta_time: f32,
        physics: Option<&PhysicsSystem>,
        move_speed: f32,
        jump_impulse: f32,
        jump_cooldown: f32,
        tag: &str,
        state: &mut BehaviorState,
        commands: &mut BehaviorCommands,
    ) {
        // Update cooldown timer
        if state.timer > 0.0 {
            state.timer -= delta_time;
        }

        // Calculate horizontal velocity only - let physics handle Y (gravity + jumps)
        let mut vel_x = 0.0;
        if self.actions.is_active(GameAction::MoveLeft, input) { vel_x = -move_speed; }
        if self.actions.is_active(GameAction::MoveRight, input) { vel_x = move_speed; }

        // For platformers, only set X velocity - preserve Y for physics
        if let Some(physics) = physics {
            let current_vel = physics.physics_world()
                .get_body_velocity(entity)
                .map(|(v, _)| v)
                .unwrap_or(Vec2::ZERO);
            // Set X to input, keep Y from physics (gravity/jumps)
            let vel = Vec2::new(vel_x, current_vel.y);
            commands.velocities.push((entity, vel));
        }

        // Jump - collect impulse to apply AFTER velocity commands.
        // Action1 (Space / pad-0 A / mouse left in the default preset) so
        // rebinds and gamepad jump work — never a raw key read.
        if self.actions.just_activated(GameAction::Action1, input) && state.timer <= 0.0 {
            commands.impulses.push((entity, Vec2::new(0.0, jump_impulse)));
            state.timer = jump_cooldown;
        }

        commands.tags.push((entity, tag.to_string()));
    }

    /// `Behavior::PlayerTopDown` — input-driven movement on both axes with
    /// normalized diagonals.
    fn update_player_top_down(
        &self,
        entity: EntityId,
        input: &InputHandler,
        move_speed: f32,
        tag: &str,
        commands: &mut BehaviorCommands,
    ) {
        // Calculate movement velocity from input
        let mut vel = Vec2::ZERO;
        if self.actions.is_active(GameAction::MoveUp, input) { vel.y += move_speed; }
        if self.actions.is_active(GameAction::MoveDown, input) { vel.y -= move_speed; }
        if self.actions.is_active(GameAction::MoveLeft, input) { vel.x -= move_speed; }
        if self.actions.is_active(GameAction::MoveRight, input) { vel.x += move_speed; }

        // Normalize diagonal movement
        if vel.length_squared() > 0.0 {
            vel = vel.normalize() * move_speed;
        }

        commands.velocities.push((entity, vel));
        commands.tags.push((entity, tag.to_string()));
    }

    /// `Behavior::ChaseTagged` — chase the nearest tagged entity once it is
    /// inside detection range, give up beyond lose-interest range.
    ///
    /// Phase FSM: `Idle` ⇄ `Chasing` — enter on `distance < detection_range`,
    /// leave on `distance > lose_interest_range` or when no target exists.
    #[allow(clippy::too_many_arguments)]
    fn update_chase_tagged(
        world: &World,
        entity: EntityId,
        delta_time: f32,
        target_tag: &str,
        detection_range: f32,
        chase_speed: f32,
        lose_interest_range: f32,
        state: &mut BehaviorState,
        commands: &mut BehaviorCommands,
    ) {
        state.phase.tick(delta_time);

        if let Some(target_pos) = Self::find_nearest_tagged_position(world, entity, target_tag) {
            if let Some(entity_pos) = Self::get_position(world, entity) {
                let distance = (target_pos - entity_pos).length();

                let chasing = state.phase.is(&BehaviorPhase::Chasing);
                if !chasing && distance < detection_range {
                    state.phase.transition_to(BehaviorPhase::Chasing);
                } else if chasing && distance > lose_interest_range {
                    state.phase.transition_to(BehaviorPhase::Idle);
                }

                if state.phase.is(&BehaviorPhase::Chasing) {
                    let vel = (target_pos - entity_pos).normalize_or_zero() * chase_speed;
                    commands.velocities.push((entity, vel));
                } else {
                    commands.velocities.push((entity, Vec2::ZERO));
                }
            }
        } else {
            state.phase.transition_to(BehaviorPhase::Idle);
            commands.velocities.push((entity, Vec2::ZERO));
        }
    }

    /// `Behavior::Patrol` — walk back and forth between two points, pausing
    /// at each end for `wait_time` seconds.
    ///
    /// Phase FSM: `Idle` → `Patrolling { toward }` → (on arrival)
    /// `Waiting { then_toward }` → (after `wait_time`, via the machine's
    /// `elapsed()` clock) → `Patrolling` toward the other endpoint.
    #[allow(clippy::too_many_arguments)]
    fn update_patrol(
        world: &World,
        entity: EntityId,
        delta_time: f32,
        point_a: Vec2,
        point_b: Vec2,
        speed: f32,
        wait_time: f32,
        state: &mut BehaviorState,
        commands: &mut BehaviorCommands,
    ) {
        state.phase.tick(delta_time);

        if let BehaviorPhase::Waiting { then_toward } = *state.phase.current() {
            if state.phase.elapsed() >= wait_time {
                state.phase.transition_to(BehaviorPhase::Patrolling { toward: then_toward });
            }
            commands.velocities.push((entity, Vec2::ZERO));
        } else if let Some(entity_pos) = Self::get_position(world, entity) {
            // Idle (first update) starts the patrol toward A, matching the
            // pre-FSM default direction.
            let toward = match *state.phase.current() {
                BehaviorPhase::Patrolling { toward } => toward,
                _ => {
                    state.phase.transition_to(BehaviorPhase::Patrolling { toward: PatrolTarget::A });
                    PatrolTarget::A
                }
            };
            let target = match toward {
                PatrolTarget::A => point_a,
                PatrolTarget::B => point_b,
            };

            if (target - entity_pos).length() < 5.0 {
                state.phase.transition_to(BehaviorPhase::Waiting { then_toward: toward.other() });
                commands.velocities.push((entity, Vec2::ZERO));
            } else {
                let vel = (target - entity_pos).normalize() * speed;
                commands.velocities.push((entity, vel));
            }
        }
    }

    /// `Behavior::FollowEntity` — move toward a named entity while farther
    /// away than `follow_distance`.
    fn update_follow_entity(
        &self,
        world: &World,
        entity: EntityId,
        target_name: &str,
        follow_distance: f32,
        follow_speed: f32,
        commands: &mut BehaviorCommands,
    ) {
        let mut vel = Vec2::ZERO;
        if let Some(&target_entity) = self.named_entities.get(target_name) {
            if let (Some(target_pos), Some(entity_pos)) = (
                Self::get_position(world, target_entity),
                Self::get_position(world, entity),
            ) {
                let to_target = target_pos - entity_pos;
                if to_target.length() > follow_distance {
                    vel = to_target.normalize() * follow_speed;
                }
            }
        }
        commands.velocities.push((entity, vel));
    }

    /// `Behavior::FollowTagged` — move toward the nearest tagged entity while
    /// farther away than `follow_distance`.
    fn update_follow_tagged(
        world: &World,
        entity: EntityId,
        target_tag: &str,
        follow_distance: f32,
        follow_speed: f32,
        commands: &mut BehaviorCommands,
    ) {
        let mut vel = Vec2::ZERO;
        if let Some(target_pos) = Self::find_nearest_tagged_position(world, entity, target_tag) {
            if let Some(entity_pos) = Self::get_position(world, entity) {
                let to_target = target_pos - entity_pos;
                if to_target.length() > follow_distance {
                    vel = to_target.normalize() * follow_speed;
                }
            }
        }
        commands.velocities.push((entity, vel));
    }

    /// `lerp_speed` is calibrated as fraction-per-frame at this frame rate;
    /// `update_camera_follow` dt-corrects so other frame rates converge at
    /// the same wall-clock speed.
    const CAMERA_LERP_REFERENCE_FPS: f32 = 60.0;

    /// `Behavior::CameraFollow` — exponentially lerp this entity toward the
    /// nearest tagged entity (plus `offset`), optionally ignoring movement
    /// while the focus point stays inside a dead-zone box centered on the
    /// entity (the camera then moves just far enough to keep the focus on
    /// the box edge).
    #[allow(clippy::too_many_arguments)]
    fn update_camera_follow(
        world: &World,
        entity: EntityId,
        delta_time: f32,
        target_tag: &str,
        lerp_speed: f32,
        offset: (f32, f32),
        dead_zone: Option<(f32, f32)>,
        commands: &mut BehaviorCommands,
    ) {
        let Some(target_pos) = Self::find_nearest_tagged_position(world, entity, target_tag) else { return };
        let Some(pos) = Self::get_position(world, entity) else { return };

        let focus = target_pos + Vec2::new(offset.0, offset.1);
        let desired = match dead_zone {
            Some((w, h)) => {
                let delta = focus - pos;
                let excess = Vec2::new(
                    delta.x - delta.x.clamp(-w * 0.5, w * 0.5),
                    delta.y - delta.y.clamp(-h * 0.5, h * 0.5),
                );
                pos + excess
            }
            None => focus,
        };

        let lerp = lerp_speed.clamp(0.0, 1.0);
        let alpha = 1.0 - (1.0 - lerp).powf(delta_time * Self::CAMERA_LERP_REFERENCE_FPS);
        let new_pos = pos + (desired - pos) * alpha;
        if new_pos != pos {
            commands.positions.push((entity, new_pos));
        }
    }

    /// `Behavior::Collectible` — emit a collection event (and optionally
    /// despawn) when an entity with the collector tag overlaps.
    fn update_collectible(
        world: &World,
        entity: EntityId,
        score_value: u32,
        despawn_on_collect: bool,
        collector_tag: &str,
        commands: &mut BehaviorCommands,
    ) {
        if Self::check_tagged_overlap(world, entity, collector_tag, 40.0) {
            log::info!("Collected! +{} points", score_value);
            commands.collected.push(EntityCollected {
                entity,
                score_value,
                collector_tag: collector_tag.to_string(),
            });
            if despawn_on_collect {
                commands.to_despawn.push(entity);
            }
        }
    }

    /// Apply the commands collected during behavior iteration: tags first,
    /// then velocities, then impulses, then events and despawns.
    fn apply_commands(
        world: &mut World,
        mut physics: Option<&mut PhysicsSystem>,
        delta_time: f32,
        commands: BehaviorCommands,
    ) {
        // Apply tag assignments
        for (entity, tag) in commands.tags {
            Self::add_entity_tag(world, entity, &tag);
        }

        // Apply velocity commands (either via physics or direct transform)
        if let Some(ref mut physics) = physics {
            for (entity, vel) in commands.velocities {
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
                    physics.set_velocity(entity, vel, 0.0);
                }
            }
        } else {
            // Fallback: direct transform modification (no physics)
            for (entity, vel) in commands.velocities {
                if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                    transform.position += vel * delta_time;
                }
            }
        }

        // Apply absolute position sets (CameraFollow) — direct transform
        // writes, independent of physics.
        for (entity, pos) in commands.positions {
            if let Some(transform) = world.get_mut::<Transform2D>(entity) {
                transform.position = pos;
            }
        }

        // Apply impulses AFTER velocity commands (so jumps aren't overwritten).
        // This is one of the few places true impulse semantics matter (a jump
        // adds to existing horizontal velocity rather than clobbering it), so
        // we reach down to PhysicsWorld::apply_impulse directly — there is no
        // PhysicsSystem-level impulse API (games use set_velocity as the
        // universal "move this body" call).
        if let Some(ref mut physics) = physics {
            for (entity, impulse) in commands.impulses {
                physics.physics_world_mut().apply_impulse(entity, impulse);
            }
        }

        // Emit collection events before despawning
        for event in commands.collected {
            world.emit_event(event);
        }

        // Remove collected entities
        for entity in commands.to_despawn {
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

    fn phase_of(world: &World, entity: EntityId) -> BehaviorPhase {
        *world
            .get::<BehaviorState>(entity)
            .expect("stateful behavior should have a BehaviorState")
            .phase
            .current()
    }

    #[test]
    fn test_patrol_arrival_enters_waiting_then_reverses_direction() {
        let mut world = World::new();
        let mut runner = BehaviorRunner::new();
        let input = InputHandler::new();
        let dt = 0.016;

        // Entity starts exactly at point A, so the first update arrives
        // immediately (< 5.0 arrival threshold) and begins the wait.
        let patroller = world.create_entity();
        world
            .add_component(&patroller, Transform2D::new(Vec2::ZERO))
            .unwrap();
        world
            .add_component(
                &patroller,
                Behavior::Patrol {
                    point_a: (0.0, 0.0),
                    point_b: (100.0, 0.0),
                    speed: 50.0,
                    wait_time: 0.1,
                },
            )
            .unwrap();

        runner.update(&mut world, &input, dt, None);
        assert_eq!(
            phase_of(&world, patroller),
            BehaviorPhase::Waiting { then_toward: PatrolTarget::B },
            "arriving at A should enter Waiting headed for B"
        );

        // Wait out wait_time (0.1s) via the FSM's elapsed clock: 7 more
        // updates at 16ms = 0.112s in the Waiting state.
        for _ in 0..7 {
            runner.update(&mut world, &input, dt, None);
        }
        assert_eq!(
            phase_of(&world, patroller),
            BehaviorPhase::Patrolling { toward: PatrolTarget::B },
            "after wait_time elapses the patrol should head for B"
        );

        // Without physics the entity never moves, so it stays far from B
        // and remains in the Patrolling phase.
        runner.update(&mut world, &input, dt, None);
        assert_eq!(
            phase_of(&world, patroller),
            BehaviorPhase::Patrolling { toward: PatrolTarget::B }
        );
    }

    #[test]
    fn test_chase_enters_and_leaves_chasing_phase_on_range() {
        let mut world = World::new();
        let mut runner = BehaviorRunner::new();
        let input = InputHandler::new();
        let dt = 0.016;

        let chaser = world.create_entity();
        world
            .add_component(&chaser, Transform2D::new(Vec2::ZERO))
            .unwrap();
        world
            .add_component(
                &chaser,
                Behavior::ChaseTagged {
                    target_tag: "player".to_string(),
                    detection_range: 50.0,
                    chase_speed: 100.0,
                    lose_interest_range: 80.0,
                },
            )
            .unwrap();

        let target = world.create_entity();
        world
            .add_component(&target, Transform2D::new(Vec2::new(30.0, 0.0)))
            .unwrap();
        world.add_component(&target, EntityTag::new("player")).unwrap();

        // Target within detection range -> Chasing.
        runner.update(&mut world, &input, dt, None);
        assert_eq!(phase_of(&world, chaser), BehaviorPhase::Chasing);

        // Inside lose-interest range the chase persists (hysteresis band).
        world.get_mut::<Transform2D>(target).unwrap().position = Vec2::new(70.0, 0.0);
        runner.update(&mut world, &input, dt, None);
        assert_eq!(
            phase_of(&world, chaser),
            BehaviorPhase::Chasing,
            "chase persists between detection and lose-interest ranges"
        );

        // Beyond lose-interest range -> back to Idle.
        world.get_mut::<Transform2D>(target).unwrap().position = Vec2::new(200.0, 0.0);
        runner.update(&mut world, &input, dt, None);
        assert_eq!(phase_of(&world, chaser), BehaviorPhase::Idle);
    }

    #[test]
    fn test_chase_with_no_target_stays_idle() {
        let mut world = World::new();
        let mut runner = BehaviorRunner::new();
        let input = InputHandler::new();

        let chaser = world.create_entity();
        world
            .add_component(&chaser, Transform2D::new(Vec2::ZERO))
            .unwrap();
        world
            .add_component(
                &chaser,
                Behavior::ChaseTagged {
                    target_tag: "player".to_string(),
                    detection_range: 50.0,
                    chase_speed: 100.0,
                    lose_interest_range: 80.0,
                },
            )
            .unwrap();

        runner.update(&mut world, &input, 0.016, None);
        assert_eq!(phase_of(&world, chaser), BehaviorPhase::Idle);
    }
}
