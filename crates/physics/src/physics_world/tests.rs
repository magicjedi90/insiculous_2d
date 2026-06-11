//! Tests for the PhysicsWorld wrapper.

use glam::Vec2;

use ecs::EntityId;

use crate::components::{Collider, RigidBody};

use super::stepping::CollisionPair;
use super::{PhysicsConfig, PhysicsWorld, DEFAULT_PIXELS_PER_METER};

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
    // (step() appends events, so clear the buffer like a frame driver would)
    world.clear_collision_events();
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
    world.clear_collision_events();
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
fn test_clear_removes_all_bodies_and_colliders() {
    let mut world = PhysicsWorld::default();
    let entity = EntityId::new();
    let mut body = RigidBody::new_dynamic();
    let mut collider = Collider::box_collider(32.0, 32.0);

    world.add_rigid_body(entity, &mut body, Vec2::ZERO, 0.0);
    world.add_collider(entity, &mut collider, Some(&body));
    assert!(world.has_rigid_body(entity));
    assert!(world.has_collider(entity));

    world.clear();

    assert!(!world.has_rigid_body(entity));
    assert!(!world.has_collider(entity));
    assert_eq!(world.rigid_body_count(), 0);
    assert_eq!(world.collider_count(), 0);
}

#[test]
fn test_clear_preserves_config() {
    let config = PhysicsConfig::new(Vec2::new(0.0, -500.0)).with_scale(50.0);
    let mut world = PhysicsWorld::new(config);

    world.clear();

    assert_eq!(world.gravity(), Vec2::new(0.0, -500.0));
    assert_eq!(world.config().pixels_per_meter, 50.0);
}

#[test]
fn test_clear_allows_re_adding_same_entity() {
    let mut world = PhysicsWorld::default();
    let entity = EntityId::new();
    let mut body = RigidBody::new_dynamic();
    let mut collider = Collider::box_collider(32.0, 32.0);

    world.add_rigid_body(entity, &mut body, Vec2::new(100.0, 200.0), 0.0);
    world.add_collider(entity, &mut collider, Some(&body));

    // Step to move body
    for _ in 0..10 {
        world.step(1.0 / 60.0);
    }

    world.clear();

    // Re-add at original position
    let mut body2 = RigidBody::new_dynamic();
    let mut collider2 = Collider::box_collider(32.0, 32.0);
    world.add_rigid_body(entity, &mut body2, Vec2::new(100.0, 200.0), 0.0);
    world.add_collider(entity, &mut collider2, Some(&body2));

    let (pos, _) = world.get_body_transform(entity).unwrap();
    assert_eq!(pos, Vec2::new(100.0, 200.0));
}

#[test]
fn test_zero_scale_falls_back_to_default_and_produces_finite_positions() {
    let config = PhysicsConfig::default().with_scale(0.0);
    assert_eq!(config.pixels_per_meter, DEFAULT_PIXELS_PER_METER);

    let mut world = PhysicsWorld::new(config);
    let entity = EntityId::new();
    let mut body = RigidBody::new_dynamic();
    let mut collider = Collider::box_collider(32.0, 32.0);
    world.add_rigid_body(entity, &mut body, Vec2::new(0.0, 100.0), 0.0);
    world.add_collider(entity, &mut collider, Some(&body));

    world.step(1.0 / 60.0);

    let (pos, _) = world.get_body_transform(entity).expect("body exists");
    assert!(pos.x.is_finite() && pos.y.is_finite(), "position must not be NaN, got {pos:?}");
}

#[test]
fn test_invalid_scale_in_struct_literal_is_sanitized_at_world_creation() {
    let config = PhysicsConfig {
        pixels_per_meter: f32::NAN,
        ..PhysicsConfig::default()
    };
    let world = PhysicsWorld::new(config);
    assert_eq!(world.config().pixels_per_meter, DEFAULT_PIXELS_PER_METER);
}

#[test]
fn test_raycast_with_zero_direction_returns_none() {
    let mut world = PhysicsWorld::default();
    let entity = EntityId::new();
    let mut body = RigidBody::new_static();
    let mut collider = Collider::box_collider(100.0, 100.0);
    world.add_rigid_body(entity, &mut body, Vec2::new(200.0, 0.0), 0.0);
    world.add_collider(entity, &mut collider, Some(&body));
    world.step(0.0);

    assert!(world.raycast(Vec2::ZERO, Vec2::ZERO, 500.0).is_none());
}

#[test]
fn test_raycast_normalizes_direction_so_distance_is_in_pixels() {
    let mut world = PhysicsWorld::default();
    let entity = EntityId::new();
    let mut body = RigidBody::new_static();
    let mut collider = Collider::box_collider(100.0, 100.0);
    world.add_rigid_body(entity, &mut body, Vec2::new(200.0, 0.0), 0.0);
    world.add_collider(entity, &mut collider, Some(&body));
    world.step(0.0);

    let (_, _, dist_unit) = world
        .raycast(Vec2::ZERO, Vec2::new(1.0, 0.0), 500.0)
        .expect("unit-direction ray should hit");
    let (_, _, dist_long) = world
        .raycast(Vec2::ZERO, Vec2::new(100.0, 0.0), 500.0)
        .expect("unnormalized-direction ray should hit");

    assert!(
        (dist_unit - dist_long).abs() < 0.01,
        "distance must be independent of direction magnitude ({dist_unit} vs {dist_long})"
    );
    assert!((dist_unit - 150.0).abs() < 1.0, "box edge is at x=150, got {dist_unit}");
}

#[test]
fn test_contact_points_are_in_world_space() {
    // Two overlapping boxes far from the origin. If contact points were
    // reported in collider-local space (the old bug), they would land
    // within ~25px of the origin instead of near the overlap region.
    let config = PhysicsConfig::new(Vec2::ZERO);
    let mut world = PhysicsWorld::new(config);

    let entity_a = EntityId::new();
    let entity_b = EntityId::new();

    let mut body_a = RigidBody::new_static();
    let mut collider_a = Collider::box_collider(50.0, 50.0);
    world.add_rigid_body(entity_a, &mut body_a, Vec2::new(1000.0, 1000.0), 0.0);
    world.add_collider(entity_a, &mut collider_a, Some(&body_a));

    let mut body_b = RigidBody::new_dynamic().with_gravity_scale(0.0);
    let mut collider_b = Collider::box_collider(50.0, 50.0);
    world.add_rigid_body(entity_b, &mut body_b, Vec2::new(1010.0, 1000.0), 0.0);
    world.add_collider(entity_b, &mut collider_b, Some(&body_b));

    world.step(1.0 / 60.0);

    let collision = world
        .collision_events()
        .iter()
        .find(|e| e.event.involves(entity_a, entity_b) && !e.contacts.is_empty())
        .expect("should have a collision with contact points");

    for contact in &collision.contacts {
        let distance = (contact.point - Vec2::new(1005.0, 1000.0)).length();
        assert!(
            distance < 60.0,
            "contact point {:?} should be near the overlap region around (1005, 1000), \
             not collider-local coordinates (distance: {})",
            contact.point,
            distance
        );
    }
}

#[test]
fn test_sensor_collider_fires_intersection_events() {
    let config = PhysicsConfig::new(Vec2::ZERO);
    let mut world = PhysicsWorld::new(config);

    // Static sensor area
    let sensor_entity = EntityId::new();
    let mut sensor_body = RigidBody::new_static();
    let mut sensor_collider = Collider::box_collider(100.0, 100.0).as_sensor();
    world.add_rigid_body(sensor_entity, &mut sensor_body, Vec2::ZERO, 0.0);
    world.add_collider(sensor_entity, &mut sensor_collider, Some(&sensor_body));

    // Dynamic body overlapping the sensor
    let visitor = EntityId::new();
    let mut visitor_body = RigidBody::new_dynamic().with_gravity_scale(0.0);
    let mut visitor_collider = Collider::box_collider(20.0, 20.0);
    world.add_rigid_body(visitor, &mut visitor_body, Vec2::new(10.0, 0.0), 0.0);
    world.add_collider(visitor, &mut visitor_collider, Some(&visitor_body));

    world.step(1.0 / 60.0);

    let event = world
        .collision_events()
        .iter()
        .find(|e| e.event.involves(sensor_entity, visitor))
        .expect("sensor intersection should produce a collision event");
    assert!(event.event.started, "sensor overlap should be reported as started");
    assert!(event.contacts.is_empty(), "sensors report no contact points");
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
