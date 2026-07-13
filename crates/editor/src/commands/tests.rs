use super::*;
use ecs::hierarchy::GlobalTransform2D;
use ecs::sprite_components::{Name, Sprite};
use ecs::EntityId;
use glam::Vec2;
use physics::components::{Collider, RigidBody};

fn setup_entity(world: &mut World) -> EntityId {
    let entity = world.create_entity();
    world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();
    world.add_component(&entity, GlobalTransform2D::default()).ok();
    world.add_component(&entity, Name::new("Test")).ok();
    entity
}

// -- CommandHistory basics --

#[test]
fn test_command_history_execute_and_undo() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let old = *world.get::<common::Transform2D>(entity).unwrap();
    let new = common::Transform2D::new(Vec2::new(10.0, 20.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(SetTransformCommand::new(entity, old, new, "position")),
        &mut world,
    );

    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(10.0, 20.0));

    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
}

#[test]
fn test_command_history_redo() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let old = *world.get::<common::Transform2D>(entity).unwrap();
    let new = common::Transform2D::new(Vec2::new(5.0, 5.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(SetTransformCommand::new(entity, old, new, "position")),
        &mut world,
    );
    history.undo(&mut world);
    history.redo(&mut world);

    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(5.0, 5.0));
}

#[test]
fn test_redo_cleared_on_new_command() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let t0 = *world.get::<common::Transform2D>(entity).unwrap();
    let t1 = common::Transform2D::new(Vec2::new(1.0, 0.0));
    let t2 = common::Transform2D::new(Vec2::new(2.0, 0.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(SetTransformCommand::new(entity, t0, t1, "position")),
        &mut world,
    );
    history.undo(&mut world);
    assert!(history.can_redo());

    // New command should clear redo.
    history.execute(
        Box::new(SetTransformCommand::new(entity, t0, t2, "position")),
        &mut world,
    );
    assert!(!history.can_redo());
}

#[test]
fn test_can_undo_and_redo() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let old = *world.get::<common::Transform2D>(entity).unwrap();
    let new = common::Transform2D::new(Vec2::ONE);

    let mut history = CommandHistory::new();
    assert!(!history.can_undo());
    assert!(!history.can_redo());

    history.execute(
        Box::new(SetTransformCommand::new(entity, old, new, "position")),
        &mut world,
    );
    assert!(history.can_undo());
    assert!(!history.can_redo());

    history.undo(&mut world);
    assert!(!history.can_undo());
    assert!(history.can_redo());
}

#[test]
fn test_undo_name_and_redo_name() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let old = *world.get::<common::Transform2D>(entity).unwrap();
    let new = common::Transform2D::new(Vec2::ONE);

    let mut history = CommandHistory::new();
    assert!(history.undo_name().is_none());
    assert!(history.redo_name().is_none());

    history.execute(
        Box::new(SetTransformCommand::new(entity, old, new, "position")),
        &mut world,
    );
    assert_eq!(history.undo_name(), Some("Set Transform"));
    assert!(history.redo_name().is_none());

    history.undo(&mut world);
    assert!(history.undo_name().is_none());
    assert_eq!(history.redo_name(), Some("Set Transform"));
}

// -- CreateEntityCommand --

#[test]
fn test_create_entity_undo_removes() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    assert_eq!(world.entity_count(), 1);

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(CreateEntityCommand::already_created(&world, entity)),
        &mut world,
    );
    assert_eq!(world.entity_count(), 1);

    history.undo(&mut world);
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn test_create_entity_redo_recreates() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let pos = Vec2::new(42.0, 99.0);
    world.get_mut::<common::Transform2D>(entity).unwrap().position = pos;

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(CreateEntityCommand::already_created(&world, entity)),
        &mut world,
    );
    history.undo(&mut world);
    assert_eq!(world.entity_count(), 0);

    history.redo(&mut world);
    assert_eq!(world.entity_count(), 1);

    // The recreated entity should have the same transform data.
    let entities: Vec<EntityId> = world.entities();
    let t = world.get::<common::Transform2D>(entities[0]).unwrap();
    assert_eq!(t.position, pos);
}

// -- DeleteEntityCommand --

#[test]
fn test_delete_entity_undo_restores() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    world.add_component(&entity, Sprite::new(7)).ok();
    assert_eq!(world.entity_count(), 1);

    let mut history = CommandHistory::new();
    history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
    assert_eq!(world.entity_count(), 0);

    history.undo(&mut world);
    assert_eq!(world.entity_count(), 1);

    let entities: Vec<EntityId> = world.entities();
    let restored = entities[0];
    assert!(world.get::<common::Transform2D>(restored).is_some());
    assert!(world.get::<Sprite>(restored).is_some());
    assert_eq!(world.get::<Sprite>(restored).unwrap().texture_handle, 7);
}

#[test]
fn test_delete_entity_redo_removes_again() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);

    let mut history = CommandHistory::new();
    history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
    history.undo(&mut world);
    assert_eq!(world.entity_count(), 1);

    history.redo(&mut world);
    assert_eq!(world.entity_count(), 0);
}

// -- AddComponentCommand --

#[test]
fn test_add_component_undo_removes() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(AddComponentCommand::new(entity, ComponentKind::Sprite)),
        &mut world,
    );
    assert!(world.get::<Sprite>(entity).is_some());

    history.undo(&mut world);
    assert!(world.get::<Sprite>(entity).is_none());
}

// -- RemoveComponentCommand --

#[test]
fn test_remove_component_undo_restores() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    world.add_component(&entity, Sprite::new(3)).ok();

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(RemoveComponentCommand::new(entity, ComponentKind::Sprite)),
        &mut world,
    );
    assert!(world.get::<Sprite>(entity).is_none());

    history.undo(&mut world);
    assert!(world.get::<Sprite>(entity).is_some());
    assert_eq!(world.get::<Sprite>(entity).unwrap().texture_handle, 3);
}

#[test]
fn test_remove_rigid_body_cascades_to_collider() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    world.add_component(&entity, RigidBody::default()).ok();
    world.add_component(&entity, Collider::default()).ok();

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(RemoveComponentCommand::new(entity, ComponentKind::RigidBody)),
        &mut world,
    );
    assert!(world.get::<RigidBody>(entity).is_none());
    assert!(world.get::<Collider>(entity).is_none());

    // Undo restores both the rigid body and the cascaded collider.
    history.undo(&mut world);
    assert!(world.get::<RigidBody>(entity).is_some());
    assert!(world.get::<Collider>(entity).is_some());
}

#[test]
fn test_remove_collider_alone_keeps_rigid_body() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    world.add_component(&entity, RigidBody::default()).ok();
    world.add_component(&entity, Collider::default()).ok();

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(RemoveComponentCommand::new(entity, ComponentKind::Collider)),
        &mut world,
    );
    assert!(world.get::<Collider>(entity).is_none());
    assert!(world.get::<RigidBody>(entity).is_some());
}

// -- TransformGizmoCommand --

#[test]
fn test_transform_gizmo_undo() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let initial = *world.get::<common::Transform2D>(entity).unwrap();
    let final_val = common::Transform2D::new(Vec2::new(100.0, 200.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(TransformGizmoCommand::new(entity, initial, final_val)),
        &mut world,
    );
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(100.0, 200.0));

    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
}

#[test]
fn test_transform_gizmo_merge() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let initial = *world.get::<common::Transform2D>(entity).unwrap();
    let mid = common::Transform2D::new(Vec2::new(50.0, 50.0));
    let final_val = common::Transform2D::new(Vec2::new(100.0, 100.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(TransformGizmoCommand::new(entity, initial, mid)),
        &mut world,
    );
    // Second gizmo drag on same entity — should merge.
    history.try_merge_or_execute(
        Box::new(TransformGizmoCommand::new(entity, mid, final_val)),
        &mut world,
    );

    // Should be a single undo entry.
    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
    assert!(!history.can_undo());
}

// -- SetTransformCommand --

#[test]
fn test_set_transform_undo() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let old = *world.get::<common::Transform2D>(entity).unwrap();
    let new = common::Transform2D::new(Vec2::new(7.0, 8.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(SetTransformCommand::new(entity, old, new, "position")),
        &mut world,
    );
    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
}

#[test]
fn test_set_transform_merge() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    let t0 = *world.get::<common::Transform2D>(entity).unwrap();
    let t1 = common::Transform2D::new(Vec2::new(1.0, 0.0));
    let t2 = common::Transform2D::new(Vec2::new(2.0, 0.0));

    let mut history = CommandHistory::new();
    history.execute(
        Box::new(SetTransformCommand::new(entity, t0, t1, "position")),
        &mut world,
    );
    history.try_merge_or_execute(
        Box::new(SetTransformCommand::new(entity, t1, t2, "position")),
        &mut world,
    );

    // Single undo should go back to original.
    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
    assert!(!history.can_undo());
}

// -- MacroCommand --

#[test]
fn test_macro_command_undo() {
    let mut world = World::new();
    let e1 = setup_entity(&mut world);
    let e2 = setup_entity(&mut world);
    let t1_old = *world.get::<common::Transform2D>(e1).unwrap();
    let t2_old = *world.get::<common::Transform2D>(e2).unwrap();
    let t1_new = common::Transform2D::new(Vec2::new(10.0, 0.0));
    let t2_new = common::Transform2D::new(Vec2::new(0.0, 10.0));

    let macro_cmd = MacroCommand::new("Move Two", vec![
        Box::new(SetTransformCommand::new(e1, t1_old, t1_new, "position")),
        Box::new(SetTransformCommand::new(e2, t2_old, t2_new, "position")),
    ]);

    let mut history = CommandHistory::new();
    history.execute(Box::new(macro_cmd), &mut world);

    assert_eq!(world.get::<common::Transform2D>(e1).unwrap().position, Vec2::new(10.0, 0.0));
    assert_eq!(world.get::<common::Transform2D>(e2).unwrap().position, Vec2::new(0.0, 10.0));

    history.undo(&mut world);
    assert_eq!(world.get::<common::Transform2D>(e1).unwrap().position, Vec2::ZERO);
    assert_eq!(world.get::<common::Transform2D>(e2).unwrap().position, Vec2::ZERO);
}

// -- Max history limit --

#[test]
fn test_max_history_limit() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);

    let mut history = CommandHistory::new();
    // Push 105 commands (limit is 100).
    for i in 0..105 {
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::new(i as f32, 0.0));
        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
    }

    // Should have at most 100 undo entries.
    let mut undo_count = 0;
    while history.can_undo() {
        history.undo(&mut world);
        undo_count += 1;
    }
    assert_eq!(undo_count, 100);
}

#[test]
fn test_max_history_drops_oldest_and_preserves_undo_order() {
    // GPP-L5: enforce_limit drops from the FRONT (oldest), so undoing back to
    // the bottom of the stack lands on the state captured by the oldest
    // *surviving* command, not the very first one — and undo runs LIFO.
    let mut world = World::new();
    let entity = setup_entity(&mut world);

    let mut history = CommandHistory::new();
    // Command i (1..=102) moves position from i-1 to i, starting at 0.
    // With a limit of 100, commands 1 and 2 (oldest) are evicted; the stack
    // holds commands 3..=102, whose oldest `old` is position 2.
    for i in 1..=102 {
        let old = common::Transform2D::new(Vec2::new((i - 1) as f32, 0.0));
        let new = common::Transform2D::new(Vec2::new(i as f32, 0.0));
        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
    }

    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(102.0, 0.0));

    // Undo runs newest-first: first undo returns to 101, and after draining the
    // whole (100-entry) stack we land on the oldest survivor's `old` == 2,
    // proving commands 1 and 2 were dropped from the front, not the back.
    assert!(history.undo(&mut world));
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(101.0, 0.0));

    let mut remaining = 1;
    while history.undo(&mut world) {
        remaining += 1;
    }
    assert_eq!(remaining, 100);
    assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(2.0, 0.0));
    assert!(!history.can_undo());
}

// -- StoredComponent round-trip (via commands) --

#[test]
fn test_stored_component_capture_and_restore() {
    let mut world = World::new();
    let entity = setup_entity(&mut world);
    world.add_component(&entity, Sprite::new(42)).ok();
    world.add_component(&entity, RigidBody::default()).ok();
    world.add_component(&entity, Collider::default()).ok();

    // Delete captures all components.
    let mut history = CommandHistory::new();
    history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
    assert_eq!(world.entity_count(), 0);

    // Undo restores them.
    history.undo(&mut world);
    assert_eq!(world.entity_count(), 1);

    let entities: Vec<EntityId> = world.entities();
    let restored = entities[0];
    assert!(world.get::<common::Transform2D>(restored).is_some());
    assert!(world.get::<Name>(restored).is_some());
    assert!(world.get::<Sprite>(restored).is_some());
    assert!(world.get::<RigidBody>(restored).is_some());
    assert!(world.get::<Collider>(restored).is_some());
}
