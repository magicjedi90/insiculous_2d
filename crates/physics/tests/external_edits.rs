//! External ECS-side edit detection (PATTERNS_AUDIT.md GPP-09):
//! live `Transform2D` edits teleport rapier bodies, live `Collider`
//! edits rebuild rapier colliders, and the physics writeback is never
//! mistaken for an external edit.

use glam::Vec2;

use ecs::sprite_components::Transform2D;
use ecs::{System, World};

use physics::{Collider, ColliderShape, PhysicsConfig, PhysicsSystem, RigidBody};

#[test]
fn test_external_transform_edit_teleports_live_body() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world
        .add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0))
        .unwrap();
    world.add_component(&entity, Collider::box_collider(16.0, 16.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0); // body synced into rapier
    system.set_velocity(entity, Vec2::new(50.0, 0.0), 0.0);
    system.update(&mut world, 1.0 / 60.0);

    // Game/editor code teleports the entity by writing Transform2D directly
    // (previously a silent no-op — the "sync only ADDS" footgun).
    world.get_mut::<Transform2D>(entity).unwrap().position = Vec2::new(500.0, 300.0);
    system.update(&mut world, 1.0 / 60.0);

    assert_eq!(system.external_edits_pushed_last_update(), 1);
    let pos = world.get::<Transform2D>(entity).unwrap().position;
    assert!(
        (pos - Vec2::new(500.0, 300.0)).length() < 5.0,
        "body must live at the teleport target (got {pos:?}, writeback would have snapped it back)"
    );
    let (vel, _) = system.get_body_velocity(entity).unwrap();
    assert!(
        (vel.x - 50.0).abs() < 1.0,
        "a teleport must preserve the body's velocity (got {vel:?})"
    );
}

#[test]
fn test_physics_writeback_is_not_mistaken_for_external_edit() {
    let mut world = World::new();
    let mut system = PhysicsSystem::new(); // default gravity: the body falls

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(0.0, 100.0))).unwrap();
    world.add_component(&entity, RigidBody::new_dynamic()).unwrap();
    world.add_component(&entity, Collider::box_collider(16.0, 16.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0); // creation frame

    let mut fell = false;
    let mut last_y = 100.0;
    for _ in 0..10 {
        system.update(&mut world, 1.0 / 60.0);
        assert_eq!(
            system.external_edits_pushed_last_update(),
            0,
            "rapier-driven motion written back into the ECS must not read as an external edit"
        );
        let y = world.get::<Transform2D>(entity).unwrap().position.y;
        if y < last_y {
            fell = true;
        }
        last_y = y;
    }
    assert!(fell, "sanity: the body should actually be falling");
}

#[test]
fn test_identical_transform_write_pushes_nothing() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::new(10.0, 20.0))).unwrap();
    world
        .add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0))
        .unwrap();
    world.add_component(&entity, Collider::box_collider(16.0, 16.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    system.update(&mut world, 1.0 / 60.0);

    // Write back the values the transform already holds (the sleeping-body
    // writeback pattern) — value comparison must keep it clean.
    let current = world.get::<Transform2D>(entity).unwrap().position;
    world.get_mut::<Transform2D>(entity).unwrap().position = current;
    system.update(&mut world, 1.0 / 60.0);
    assert_eq!(system.external_edits_pushed_last_update(), 0);
}

#[test]
fn test_collider_edit_rebuilds_live_rapier_collider() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    // Two bodies 100px apart with small colliders: no contact.
    let a = world.create_entity();
    world.add_component(&a, Transform2D::new(Vec2::ZERO)).unwrap();
    world.add_component(&a, RigidBody::new_dynamic().with_gravity_scale(0.0)).unwrap();
    world.add_component(&a, Collider::box_collider(20.0, 20.0)).unwrap();
    let b = world.create_entity();
    world.add_component(&b, Transform2D::new(Vec2::new(100.0, 0.0))).unwrap();
    world.add_component(&b, RigidBody::new_dynamic().with_gravity_scale(0.0)).unwrap();
    world.add_component(&b, Collider::box_collider(20.0, 20.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    assert!(
        system.take_collision_events().is_empty(),
        "sanity: small colliders must not touch"
    );

    // Editor-style live edit: grow A's collider until the two overlap
    // (previously a silent no-op — the editor collider-edit footgun).
    world.get_mut::<Collider>(a).unwrap().shape =
        ColliderShape::box_shape(240.0, 40.0);
    system.update(&mut world, 1.0 / 60.0);
    assert!(
        system.external_edits_pushed_last_update() >= 1,
        "the collider edit must be detected and pushed"
    );
    let started = system
        .take_collision_events()
        .iter()
        .any(|c| c.event.started);
    assert!(
        started,
        "the rebuilt (larger) collider must actually collide in rapier"
    );
}

#[test]
fn test_collider_component_removal_drops_rapier_collider() {
    let mut world = World::new();
    let mut system = PhysicsSystem::with_config(PhysicsConfig::new(Vec2::ZERO));

    let entity = world.create_entity();
    world.add_component(&entity, Transform2D::new(Vec2::ZERO)).unwrap();
    world
        .add_component(&entity, RigidBody::new_dynamic().with_gravity_scale(0.0))
        .unwrap();
    world.add_component(&entity, Collider::box_collider(16.0, 16.0)).unwrap();

    system.initialize(&mut world).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    assert!(system.physics_world().has_collider(entity));

    world.remove_component::<Collider>(&entity).unwrap();
    system.update(&mut world, 1.0 / 60.0);
    assert!(
        !system.physics_world().has_collider(entity),
        "removing the Collider component must remove the rapier collider"
    );
    assert_eq!(system.external_edits_pushed_last_update(), 1);
}
