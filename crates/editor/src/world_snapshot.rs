//! World state snapshot for play-mode save/restore.
//!
//! Captures all known component types from a `World` into a plain struct
//! and restores them on stop. This uses typed cloning — no serialization
//! required — so it is fast and does not change the `Component` trait.
//!
//! **Known limitation:** Custom component types not in the known list are
//! lost on restore. Acceptable for Phase 1C.

use ecs::{EntityId, World};
use ecs::hierarchy::{Children, GlobalTransform2D, Parent};
use ecs::sprite_components::{Name, Sprite, SpriteAnimation};
use ecs::audio_components::{AudioListener, AudioSource};
use physics::components::{Collider, RigidBody};

/// Snapshot of a single entity's components.
struct EntitySnapshot {
    id: EntityId,
    // Core
    transform: Option<common::Transform2D>,
    global_transform: Option<GlobalTransform2D>,
    camera: Option<common::Camera>,
    name: Option<Name>,
    // Rendering
    sprite: Option<Sprite>,
    sprite_animation: Option<SpriteAnimation>,
    // Physics
    rigid_body: Option<RigidBody>,
    collider: Option<Collider>,
    // Audio
    audio_source: Option<AudioSource>,
    audio_listener: Option<AudioListener>,
    // Hierarchy
    parent: Option<Parent>,
    children: Option<Children>,
}

impl EntitySnapshot {
    /// Capture all known components from a single entity.
    fn capture(world: &World, id: EntityId) -> Self {
        Self {
            id,
            transform: world.get::<common::Transform2D>(id).cloned(),
            global_transform: world.get::<GlobalTransform2D>(id).cloned(),
            camera: world.get::<common::Camera>(id).cloned(),
            name: world.get::<Name>(id).cloned(),
            sprite: world.get::<Sprite>(id).cloned(),
            sprite_animation: world.get::<SpriteAnimation>(id).cloned(),
            rigid_body: world.get::<RigidBody>(id).cloned(),
            collider: world.get::<Collider>(id).cloned(),
            audio_source: world.get::<AudioSource>(id).cloned(),
            audio_listener: world.get::<AudioListener>(id).cloned(),
            parent: world.get::<Parent>(id).cloned(),
            children: world.get::<Children>(id).cloned(),
        }
    }

    /// Restore this snapshot's components onto an existing entity in the world.
    fn restore(self, world: &mut World) {
        let id = self.id;
        if let Some(c) = self.transform { world.add_component(&id, c).ok(); }
        if let Some(c) = self.global_transform { world.add_component(&id, c).ok(); }
        if let Some(c) = self.camera { world.add_component(&id, c).ok(); }
        if let Some(c) = self.name { world.add_component(&id, c).ok(); }
        if let Some(c) = self.sprite { world.add_component(&id, c).ok(); }
        if let Some(c) = self.sprite_animation { world.add_component(&id, c).ok(); }
        if let Some(c) = self.rigid_body { world.add_component(&id, c).ok(); }
        if let Some(c) = self.collider { world.add_component(&id, c).ok(); }
        if let Some(c) = self.audio_source { world.add_component(&id, c).ok(); }
        if let Some(c) = self.audio_listener { world.add_component(&id, c).ok(); }
        if let Some(c) = self.parent { world.add_component(&id, c).ok(); }
        if let Some(c) = self.children { world.add_component(&id, c).ok(); }
    }
}

/// A complete snapshot of all entities and their known components.
///
/// Created by `WorldSnapshot::capture()` before entering play mode and
/// consumed by `WorldSnapshot::restore()` when stopping play mode.
pub struct WorldSnapshot {
    snapshots: Vec<EntitySnapshot>,
}

impl WorldSnapshot {
    /// Capture the current world state.
    pub fn capture(world: &World) -> Self {
        let snapshots = world.entities()
            .into_iter()
            .map(|id| EntitySnapshot::capture(world, id))
            .collect();
        Self { snapshots }
    }

    /// Restore the captured state, replacing the current world contents.
    ///
    /// Clears all entities and components, then recreates them from the snapshot.
    pub fn restore(self, world: &mut World) {
        world.clear();

        for snapshot in self.snapshots {
            world.create_entity_with_id(snapshot.id);
            snapshot.restore(world);
        }
    }

    /// Number of entities in the snapshot.
    pub fn entity_count(&self) -> usize {
        self.snapshots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_snapshot_empty_world() {
        let world = World::new();
        let snapshot = WorldSnapshot::capture(&world);
        assert_eq!(snapshot.entity_count(), 0);
    }

    #[test]
    fn test_snapshot_captures_entities() {
        let mut world = World::new();
        world.create_entity();
        world.create_entity();
        world.create_entity();

        let snapshot = WorldSnapshot::capture(&world);
        assert_eq!(snapshot.entity_count(), 3);
    }

    #[test]
    fn test_snapshot_restore_preserves_entity_ids() {
        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        world.add_component(&e1, common::Transform2D::new(Vec2::new(10.0, 20.0))).ok();
        world.add_component(&e2, common::Transform2D::new(Vec2::new(30.0, 40.0))).ok();

        let snapshot = WorldSnapshot::capture(&world);

        // Modify world
        world.clear();
        assert_eq!(world.entity_count(), 0);

        // Restore
        snapshot.restore(&mut world);

        assert_eq!(world.entity_count(), 2);
        let t1 = world.get::<common::Transform2D>(e1).unwrap();
        assert_eq!(t1.position, Vec2::new(10.0, 20.0));
        let t2 = world.get::<common::Transform2D>(e2).unwrap();
        assert_eq!(t2.position, Vec2::new(30.0, 40.0));
    }

    #[test]
    fn test_snapshot_restore_discards_play_changes() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();

        let snapshot = WorldSnapshot::capture(&world);

        // Simulate play-mode changes
        if let Some(t) = world.get_mut::<common::Transform2D>(entity) {
            t.position = Vec2::new(999.0, 999.0);
        }
        let new_entity = world.create_entity();
        world.add_component(&new_entity, common::Transform2D::new(Vec2::ONE)).ok();

        // Restore should undo play changes
        snapshot.restore(&mut world);

        assert_eq!(world.entity_count(), 1);
        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.position, Vec2::ZERO);
    }

    #[test]
    fn test_snapshot_preserves_hierarchy() {
        use ecs::WorldHierarchyExt;

        let mut world = World::new();
        let parent = world.create_entity();
        let child = world.create_entity();
        world.set_parent(child, parent).unwrap();

        let snapshot = WorldSnapshot::capture(&world);
        world.clear();
        snapshot.restore(&mut world);

        // Hierarchy components should be restored
        let p = world.get::<Parent>(child).unwrap();
        assert_eq!(p.entity(), parent);
        let c = world.get::<Children>(parent).unwrap();
        assert!(c.entities().contains(&child));
    }

    #[test]
    fn test_snapshot_preserves_physics_components() {
        let mut world = World::new();
        let entity = world.create_entity();
        let mut body = RigidBody::default();
        body.gravity_scale = 0.5;
        body.linear_damping = 2.0;
        world.add_component(&entity, body).ok();

        let mut collider = Collider::default();
        collider.friction = 0.9;
        collider.is_sensor = true;
        world.add_component(&entity, collider).ok();

        let snapshot = WorldSnapshot::capture(&world);
        world.clear();
        snapshot.restore(&mut world);

        let rb = world.get::<RigidBody>(entity).unwrap();
        assert_eq!(rb.gravity_scale, 0.5);
        assert_eq!(rb.linear_damping, 2.0);

        let col = world.get::<Collider>(entity).unwrap();
        assert_eq!(col.friction, 0.9);
        assert!(col.is_sensor);
    }
}
