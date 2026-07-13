//! Regression tests for the dirty-flagged transform hierarchy propagation
//! (PATTERNS_AUDIT.md GPP-04): clean frames recompute nothing, changes
//! recompute exactly the affected subtree, and the baseline cache stays
//! consistent through reparenting, deletion, disable/enable, and reset.

use ecs::{
    GlobalTransform2D, System, Transform2D, TransformHierarchySystem, World, WorldHierarchyExt,
};
use glam::Vec2;

fn spawn(world: &mut World, pos: Vec2) -> ecs::EntityId {
    let e = world.create_entity();
    world.add_component(&e, Transform2D::new(pos)).unwrap();
    world.add_component(&e, GlobalTransform2D::default()).unwrap();
    e
}

fn global_pos(world: &World, e: ecs::EntityId) -> Vec2 {
    world.get::<GlobalTransform2D>(e).unwrap().position
}

#[test]
fn test_no_change_second_frame_recomputes_zero() {
    let mut world = World::new();
    let parent = spawn(&mut world, Vec2::new(100.0, 0.0));
    let child = spawn(&mut world, Vec2::new(50.0, 0.0));
    world.set_parent(child, parent).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 2, "first frame computes everything");

    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 0, "clean frame must recompute nothing");
    assert_eq!(system.visited_last_update(), 2, "clean entities are still dirty-checked");
    assert_eq!(global_pos(&world, child), Vec2::new(150.0, 0.0), "globals stay correct");
}

#[test]
fn test_leaf_change_recomputes_one() {
    let mut world = World::new();
    let root = spawn(&mut world, Vec2::new(100.0, 0.0));
    let mid = spawn(&mut world, Vec2::new(10.0, 0.0));
    let leaf = spawn(&mut world, Vec2::new(1.0, 0.0));
    let sibling = spawn(&mut world, Vec2::new(2.0, 0.0));
    world.set_parent(mid, root).unwrap();
    world.set_parent(leaf, mid).unwrap();
    world.set_parent(sibling, mid).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);

    world.get_mut::<Transform2D>(leaf).unwrap().position = Vec2::new(5.0, 0.0);
    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 1, "only the mutated leaf recomputes");
    assert_eq!(global_pos(&world, leaf), Vec2::new(115.0, 0.0));
    assert_eq!(global_pos(&world, sibling), Vec2::new(112.0, 0.0), "sibling untouched and correct");
}

#[test]
fn test_parent_change_recomputes_subtree_only() {
    let mut world = World::new();
    let parent = spawn(&mut world, Vec2::new(100.0, 0.0));
    let child_a = spawn(&mut world, Vec2::new(10.0, 0.0));
    let child_b = spawn(&mut world, Vec2::new(20.0, 0.0));
    world.set_parent(child_a, parent).unwrap();
    world.set_parent(child_b, parent).unwrap();
    // Unrelated tree that must stay clean.
    let other_root = spawn(&mut world, Vec2::new(-100.0, 0.0));
    let other_child = spawn(&mut world, Vec2::new(-10.0, 0.0));
    world.set_parent(other_child, other_root).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);

    world.get_mut::<Transform2D>(parent).unwrap().position = Vec2::new(200.0, 0.0);
    system.update(&mut world, 0.016);
    assert_eq!(
        system.recomputed_last_update(),
        3,
        "parent + its two children recompute; the unrelated tree does not"
    );
    assert_eq!(global_pos(&world, child_a), Vec2::new(210.0, 0.0));
    assert_eq!(global_pos(&world, child_b), Vec2::new(220.0, 0.0));
    assert_eq!(global_pos(&world, other_child), Vec2::new(-110.0, 0.0));
}

#[test]
fn test_reparent_recomputes() {
    let mut world = World::new();
    let a = spawn(&mut world, Vec2::new(100.0, 0.0));
    let b = spawn(&mut world, Vec2::new(500.0, 0.0));
    let child = spawn(&mut world, Vec2::new(10.0, 0.0));
    world.set_parent(child, a).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);
    assert_eq!(global_pos(&world, child), Vec2::new(110.0, 0.0));

    world.set_parent(child, b).unwrap();
    system.update(&mut world, 0.016);
    assert!(system.recomputed_last_update() >= 1, "reparented child must be dirty");
    assert_eq!(global_pos(&world, child), Vec2::new(510.0, 0.0));
}

#[test]
fn test_parent_deletion_orphans_recompute_and_cache_prunes() {
    let mut world = World::new();
    let parent = spawn(&mut world, Vec2::new(100.0, 0.0));
    let child = spawn(&mut world, Vec2::new(10.0, 0.0));
    world.set_parent(child, parent).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);
    assert_eq!(system.tracked_entity_count(), 2);
    assert_eq!(global_pos(&world, child), Vec2::new(110.0, 0.0));

    // remove_entity auto-detaches hierarchy links (child orphans to root).
    world.remove_entity(&parent).unwrap();
    system.update(&mut world, 0.016);
    assert_eq!(
        system.recomputed_last_update(),
        1,
        "orphan recomputes as a root (its parent link changed)"
    );
    assert_eq!(global_pos(&world, child), Vec2::new(10.0, 0.0), "orphan global = its local");
    assert_eq!(system.tracked_entity_count(), 1, "removed entity pruned from cache");
}

#[test]
fn test_identical_write_stays_clean() {
    let mut world = World::new();
    let entity = spawn(&mut world, Vec2::new(100.0, 50.0));

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);

    // The physics-writeback pattern: get_mut + write the same values (a
    // sleeping body). Value comparison must keep the entity clean.
    let t = world.get_mut::<Transform2D>(entity).unwrap();
    t.position = Vec2::new(100.0, 50.0);
    t.rotation = 0.0;

    system.update(&mut world, 0.016);
    assert_eq!(
        system.recomputed_last_update(),
        0,
        "writing identical values must not dirty the entity"
    );
}

#[test]
fn test_reenable_after_disable_catches_stale() {
    let mut world = World::new();
    let entity = spawn(&mut world, Vec2::new(100.0, 0.0));

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);

    system.set_enabled(false);
    world.get_mut::<Transform2D>(entity).unwrap().position = Vec2::new(300.0, 0.0);
    system.update(&mut world, 0.016); // no-op while disabled
    assert_eq!(global_pos(&world, entity), Vec2::new(100.0, 0.0), "disabled = stale global");

    system.set_enabled(true);
    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 1, "drift detected on re-enable");
    assert_eq!(global_pos(&world, entity), Vec2::new(300.0, 0.0));
}

#[test]
fn test_global_transform_readded_when_removed() {
    let mut world = World::new();
    let entity = spawn(&mut world, Vec2::new(100.0, 0.0));

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);

    world.remove_component::<GlobalTransform2D>(&entity).unwrap();
    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 1, "missing global must be restored");
    assert_eq!(global_pos(&world, entity), Vec2::new(100.0, 0.0));
}

#[test]
fn test_reset_forces_full_recompute() {
    let mut world = World::new();
    let parent = spawn(&mut world, Vec2::new(100.0, 0.0));
    let child = spawn(&mut world, Vec2::new(10.0, 0.0));
    world.set_parent(child, parent).unwrap();

    let mut system = TransformHierarchySystem::new();
    system.update(&mut world, 0.016);
    system.update(&mut world, 0.016);
    assert_eq!(system.recomputed_last_update(), 0);

    system.reset();
    assert_eq!(system.tracked_entity_count(), 0);
    system.update(&mut world, 0.016);
    assert_eq!(
        system.recomputed_last_update(),
        2,
        "reset() discards every baseline — next frame recomputes all"
    );
    assert_eq!(global_pos(&world, child), Vec2::new(110.0, 0.0));
}
