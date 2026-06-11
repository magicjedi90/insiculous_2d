//! Tests for the PhysicsSystem ECS driver.
use glam::Vec2;

use ecs::sprite_components::Transform2D;
use ecs::{System, World};

use crate::components::{Collider, RigidBody};
use crate::physics_world::PhysicsConfig;

use super::{PhysicsSystem, MAX_STEPS_PER_UPDATE};

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
fn test_direct_world_removal_cleans_up_physics_state() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    assert!(system.physics_world().has_rigid_body(entity));

    // Bypass destroy_entity — remove straight from the ECS, and leave a
    // pending velocity behind to make sure it gets pruned too.
    system.set_velocity(entity, Vec2::new(100.0, 0.0), 0.0);
    world.remove_entity(&entity).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    assert!(
        !system.physics_world().has_rigid_body(entity),
        "orphaned rapier body should be garbage-collected"
    );
    assert!(
        !system.physics_world().has_collider(entity),
        "orphaned rapier collider should be garbage-collected"
    );
    assert!(system.pending_velocities.is_empty());
}

#[test]
fn test_reset_body_is_deferred_for_same_frame_spawns() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    // Spawn and immediately reset + launch, before any update() sync.
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(50.0, 50.0))).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(8.0, 8.0)).unwrap();

    system.reset_body(entity, Vec2::ZERO);
    system.set_velocity(entity, Vec2::new(200.0, 0.0), 0.0);
    assert_eq!(system.pending_resets.len(), 1, "reset should be buffered");

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    // Reset moved the body to origin, then the launch velocity applied.
    let (velocity, _) = system.get_body_velocity(entity).unwrap();
    assert!(
        velocity.x > 100.0,
        "deferred reset must not clobber a deferred launch velocity (got {:?})",
        velocity
    );
    let pos = world.get::<Transform2D>(entity).unwrap().position;
    assert!(pos.x < 25.0, "body should have been reset toward origin (got {:?})", pos);
    assert!(system.pending_resets.is_empty());
}

#[test]
fn test_catch_up_steps_are_capped_after_a_stall() {
    let mut world = World::new();
    // Tiny fixed timestep: a 0.1s update would need 100 catch-up steps
    // uncapped; the cap drops the excess instead of simulating it.
    let mut system = PhysicsSystem::new().with_fixed_timestep(1.0 / 1000.0);

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 0.1);

    // At most MAX_STEPS_PER_UPDATE steps of 1ms ran, so at most 8ms of
    // gravity was simulated (~0.03 units of fall), not 100ms (~4.9 units).
    let y = world.get::<Transform2D>(entity).unwrap().position.y;
    let fallen = 100.0 - y;
    let max_simulated = MAX_STEPS_PER_UPDATE as f32 * (1.0 / 1000.0);
    let max_fall = 0.5 * 980.0 * max_simulated * max_simulated + 1.0;
    assert!(
        fallen <= max_fall,
        "fell {} units; catch-up steps were not capped",
        fallen
    );

    // The dropped backlog must not be simulated later: a follow-up tiny
    // update should run at most one step.
    let y_before = world.get::<Transform2D>(entity).unwrap().position.y;
    system.update(&mut world, 1.0 / 1000.0);
    let y_after = world.get::<Transform2D>(entity).unwrap().position.y;
    assert!(
        (y_before - y_after).abs() < 0.01,
        "accumulated backlog leaked into the next update"
    );
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
fn test_clear_resets_physics_state() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    assert!(system.physics_world().has_rigid_body(entity));

    system.clear();

    assert!(!system.physics_world().has_rigid_body(entity));
    assert_eq!(system.physics_world().rigid_body_count(), 0);
}

#[test]
fn test_clear_allows_resync_from_ecs() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    // Run physics for several frames (body falls due to gravity)
    system.initialize(&mut world).unwrap();
    for _ in 0..30 {
        system.update(&mut world, 1.0 / 60.0);
    }
    let fallen_y = world.get::<Transform2D>(entity).unwrap().position.y;
    assert!(fallen_y < 100.0, "Body should have fallen");

    // Restore original position in ECS (simulating snapshot restore)
    if let Some(t) = world.get_mut::<Transform2D>(entity) {
        t.position = Vec2::new(0.0, 100.0);
    }
    if let Some(rb) = world.get_mut::<RigidBody>(entity) {
        rb.velocity = Vec2::ZERO;
    }

    // Clear physics and update — should re-sync from ECS
    system.clear();
    system.update(&mut world, 0.0); // Zero dt to just sync without stepping

    let pos = world.get::<Transform2D>(entity).unwrap().position;
    assert_eq!(pos, Vec2::new(0.0, 100.0), "Position should match restored ECS state");
}

#[test]
fn test_clear_preserves_callbacks() {
    let mut system = PhysicsSystem::new();
    system.add_collision_callback(|_| {});
    system.add_collision_callback(|_| {});
    assert_eq!(system.collision_callback_count(), 2);

    system.clear();

    assert_eq!(system.collision_callback_count(), 2, "Callbacks should survive clear");
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

// === Collision event delivery tests ===

/// Create a world with two overlapping no-gravity bodies and a started-event counter.
fn overlapping_pair_with_started_counter() -> (
    World,
    PhysicsSystem,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    use std::sync::{Arc, atomic::AtomicUsize};

    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    for x in [0.0, 10.0] {
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(x, 0.0))).unwrap();
        world
            .add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0))
            .unwrap();
        world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();
    }

    let started_count = Arc::new(AtomicUsize::new(0));
    let counter = started_count.clone();
    system.add_collision_callback(move |collision| {
        if collision.event.started {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    });

    (world, system, started_count)
}

#[test]
fn test_started_event_is_delivered_exactly_once_across_zero_step_updates() {
    use std::sync::atomic::Ordering;

    let (mut world, mut system, started_count) = overlapping_pair_with_started_counter();
    system.initialize(&mut world).unwrap();

    // First update runs exactly one fixed step: collision starts.
    system.update(&mut world, 1.0 / 60.0);
    assert_eq!(started_count.load(Ordering::SeqCst), 1);

    // These updates are too small to produce a physics step. Stale events
    // from the last step must NOT be re-delivered.
    system.update(&mut world, 0.001);
    system.update(&mut world, 0.001);
    assert_eq!(
        started_count.load(Ordering::SeqCst),
        1,
        "started event must be delivered exactly once, not re-emitted on zero-step frames"
    );
}

#[test]
fn test_zero_step_update_emits_no_collision_events() {
    let (mut world, mut system, _) = overlapping_pair_with_started_counter();
    system.initialize(&mut world).unwrap();

    system.update(&mut world, 1.0 / 60.0);
    assert!(!system.collision_events().is_empty(), "stepped frame should have events");

    system.update(&mut world, 0.001); // zero steps
    assert!(
        system.collision_events().is_empty(),
        "a frame with zero physics steps must emit no collision events"
    );
}

#[test]
fn test_events_from_all_sub_steps_in_one_update_survive() {
    let (mut world, mut system, _) = overlapping_pair_with_started_counter();
    system.initialize(&mut world).unwrap();

    // First update: collision starts.
    system.update(&mut world, 1.0 / 60.0);

    // Second update runs two catch-up sub-steps; each emits an ongoing
    // event for the still-overlapping pair. Both must survive — the
    // second sub-step must not wipe the first sub-step's events.
    system.update(&mut world, 2.0 / 60.0);
    let ongoing = system
        .collision_events()
        .iter()
        .filter(|e| !e.event.stopped)
        .count();
    assert_eq!(
        ongoing, 2,
        "events from every sub-step in a single update must all be delivered"
    );
}

#[test]
fn test_collision_callbacks_fire_on_real_collision() {
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    for x in [0.0, 10.0] {
        let entity = world.create_entity();
        world.add_component(&entity, Transform2D::new(Vec2::new(x, 0.0))).unwrap();
        world
            .add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0))
            .unwrap();
        world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();
    }

    let event_count = Arc::new(AtomicUsize::new(0));
    let counter = event_count.clone();
    system.add_collision_callback(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    });

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    assert!(
        event_count.load(Ordering::SeqCst) > 0,
        "registered callback must be invoked when two bodies actually collide"
    );
}

// === Force lifetime tests ===

#[test]
fn test_apply_force_lasts_exactly_one_update() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0); // sync body into rapier

    // Apply a force and step once: velocity increases.
    system.apply_force(entity, Vec2::new(10_000.0, 0.0));
    system.update(&mut world, 1.0 / 60.0);
    let (vel_after_force, _) = system.get_body_velocity(entity).expect("body exists");
    assert!(vel_after_force.x > 0.0, "force should accelerate the body");

    // No force applied this frame: velocity must not keep increasing
    // (no gravity, no damping — a persisting force would accelerate).
    system.update(&mut world, 1.0 / 60.0);
    let (vel_next, _) = system.get_body_velocity(entity).expect("body exists");
    assert!(
        (vel_next.x - vel_after_force.x).abs() < 0.5,
        "force must not persist past one update (was {}, now {})",
        vel_after_force.x,
        vel_next.x
    );
}

#[test]
fn test_force_applied_on_zero_step_frame_acts_on_next_stepped_frame() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0); // sync body into rapier

    // Apply a force, then run an update too small to step physics.
    system.apply_force(entity, Vec2::new(10_000.0, 0.0));
    system.update(&mut world, 0.001); // zero steps — force must survive

    // The next stepped frame must still feel the force.
    system.update(&mut world, 1.0 / 60.0);
    let (vel, _) = system.get_body_velocity(entity).expect("body exists");
    assert!(
        vel.x > 0.0,
        "force applied during a zero-step frame should act on the next stepped frame"
    );
}

// === Promoted PhysicsWorld method tests ===

#[test]
fn test_set_and_get_body_velocity() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0)).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    system.set_velocity(entity, Vec2::new(200.0, 100.0), 0.0);
    let (vel, _) = system.get_body_velocity(entity).expect("body should exist");
    assert!((vel.x - 200.0).abs() < 1.0, "x velocity should be ~200, got {}", vel.x);
    assert!((vel.y - 100.0).abs() < 1.0, "y velocity should be ~100, got {}", vel.y);
}

#[test]
fn test_set_body_transform_updates_position() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0)).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    system.set_body_transform(entity, Vec2::new(500.0, 300.0), 0.0);
    system.update(&mut world, 1.0 / 60.0);

    let pos = world.get::<Transform2D>(entity).unwrap().position;
    assert!((pos.x - 500.0).abs() < 2.0, "x should be ~500, got {}", pos.x);
    assert!((pos.y - 300.0).abs() < 2.0, "y should be ~300, got {}", pos.y);
}

#[test]
fn test_reset_body_zeros_velocity_and_sets_position() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new();

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0)).unwrap();
    world.add_component(&entity, Collider::box_collider(32.0, 32.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    system.set_velocity(entity, Vec2::new(999.0, 999.0), 0.0);
    system.reset_body(entity, Vec2::new(100.0, 200.0));

    let (vel, _) = system.get_body_velocity(entity).expect("body should exist");
    assert!(vel.length() < 1.0, "velocity should be ~zero after reset, got {:?}", vel);
}

#[test]
fn test_get_body_velocity_returns_none_for_unknown_entity() {
    let system = PhysicsSystem::new();
    let fake_entity = ecs::EntityId::new();
    assert!(system.get_body_velocity(fake_entity).is_none());
}

// === Hierarchy interaction (pinned behavior) ===

#[test]
fn test_parented_entity_with_rigid_body_is_treated_as_world_space() {
    // Pins current behavior: physics reads/writes Transform2D as
    // world-space and ignores ECS parent-child hierarchy entirely.
    // A child's LOCAL transform is used as the body's WORLD position,
    // and the rapier result is written straight back into the local
    // transform (where hierarchy propagation would re-interpret it).
    // Rule: physics entities must be root entities.
    use ecs::WorldHierarchyExt;

    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let parent = world.create_entity();
    world.add_component(&parent, Transform2D::new(Vec2::new(100.0, 0.0))).unwrap();

    let child = world.create_entity();
    world.add_component(&child, Transform2D::new(Vec2::new(0.0, 50.0))).unwrap();
    world
        .add_component(&child, RigidBody::new_dynamic().with_gravity_scale(0.0))
        .unwrap();
    world.add_component(&child, Collider::box_collider(16.0, 16.0)).unwrap();
    world.set_parent(child, parent).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);

    // The rapier body was created at the child's LOCAL position (0, 50);
    // the parent's (100, 0) offset is NOT applied.
    let (body_pos, _) = system
        .physics_world()
        .get_body_transform(child)
        .expect("child body exists");
    assert!(
        (body_pos - Vec2::new(0.0, 50.0)).length() < 1.0,
        "physics ignores the parent transform (body at {:?})",
        body_pos
    );

    // And the body position is written back into the child's local
    // Transform2D unchanged.
    let local = world.get::<Transform2D>(child).unwrap().position;
    assert!(
        (local - Vec2::new(0.0, 50.0)).length() < 1.0,
        "physics writes world-space position into the local transform (got {:?})",
        local
    );
}
