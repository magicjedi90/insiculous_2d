//! Type-safe component storage for undo/redo commands.
//!
//! `StoredComponent` captures component values as concrete enum variants,
//! avoiding trait-object storage while covering all known component types.

use ecs::audio_components::{AudioListener, AudioSource};
use ecs::hierarchy::GlobalTransform2D;
use ecs::sprite_components::{Name, Sprite, SpriteAnimation};
use ecs::{EntityId, World};
use physics::components::{Collider, RigidBody};

/// A captured component value for undo/redo storage.
///
/// Each variant stores a cloned concrete component type, avoiding the need
/// for trait objects and enabling type-safe restore operations.
#[derive(Debug, Clone)]
pub enum StoredComponent {
    Transform2D(common::Transform2D),
    GlobalTransform2D(GlobalTransform2D),
    Name(Name),
    Camera(common::Camera),
    Sprite(Sprite),
    SpriteAnimation(SpriteAnimation),
    RigidBody(RigidBody),
    Collider(Collider),
    AudioSource(AudioSource),
    AudioListener(AudioListener),
}

impl StoredComponent {
    /// Add this stored component to an entity in the world.
    pub fn apply_to(&self, world: &mut World, entity: EntityId) {
        match self {
            StoredComponent::Transform2D(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::GlobalTransform2D(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::Name(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::Camera(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::Sprite(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::SpriteAnimation(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::RigidBody(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::Collider(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::AudioSource(c) => { world.add_component(&entity, c.clone()).ok(); }
            StoredComponent::AudioListener(c) => { world.add_component(&entity, c.clone()).ok(); }
        }
    }
}

/// Capture all known component types from an entity into a `Vec<StoredComponent>`.
///
/// This reads every known component type and stores any that are present.
/// Hierarchy components (Parent, Children) are deliberately excluded —
/// hierarchy is managed separately by the command implementations.
pub fn capture_all_components(world: &World, entity: EntityId) -> Vec<StoredComponent> {
    let mut components = Vec::new();

    if let Some(c) = world.get::<common::Transform2D>(entity) {
        components.push(StoredComponent::Transform2D(c.clone()));
    }
    if let Some(c) = world.get::<GlobalTransform2D>(entity) {
        components.push(StoredComponent::GlobalTransform2D(c.clone()));
    }
    if let Some(c) = world.get::<Name>(entity) {
        components.push(StoredComponent::Name(c.clone()));
    }
    if let Some(c) = world.get::<common::Camera>(entity) {
        components.push(StoredComponent::Camera(c.clone()));
    }
    if let Some(c) = world.get::<Sprite>(entity) {
        components.push(StoredComponent::Sprite(c.clone()));
    }
    if let Some(c) = world.get::<SpriteAnimation>(entity) {
        components.push(StoredComponent::SpriteAnimation(c.clone()));
    }
    if let Some(c) = world.get::<RigidBody>(entity) {
        components.push(StoredComponent::RigidBody(c.clone()));
    }
    if let Some(c) = world.get::<Collider>(entity) {
        components.push(StoredComponent::Collider(c.clone()));
    }
    if let Some(c) = world.get::<AudioSource>(entity) {
        components.push(StoredComponent::AudioSource(c.clone()));
    }
    if let Some(c) = world.get::<AudioListener>(entity) {
        components.push(StoredComponent::AudioListener(c.clone()));
    }

    components
}

/// Restore a set of stored components onto an entity.
pub fn restore_components(world: &mut World, entity: EntityId, components: &[StoredComponent]) {
    for component in components {
        component.apply_to(world, entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_capture_empty_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        let captured = capture_all_components(&world, entity);
        assert!(captured.is_empty());
    }

    #[test]
    fn test_capture_and_restore_round_trip() {
        let mut world = World::new();
        let entity = world.create_entity();
        let pos = Vec2::new(42.0, 99.0);
        world.add_component(&entity, common::Transform2D::new(pos)).ok();
        world.add_component(&entity, GlobalTransform2D::default()).ok();
        world.add_component(&entity, Name::new("TestEntity")).ok();
        world.add_component(&entity, Sprite::new(5)).ok();
        world.add_component(&entity, RigidBody::default()).ok();

        let captured = capture_all_components(&world, entity);
        assert_eq!(captured.len(), 5);

        // Create a fresh entity and restore onto it
        let new_entity = world.create_entity();
        restore_components(&mut world, new_entity, &captured);

        let t = world.get::<common::Transform2D>(new_entity).unwrap();
        assert_eq!(t.position, pos);
        assert!(world.get::<Name>(new_entity).is_some());
        assert!(world.get::<Sprite>(new_entity).is_some());
        assert!(world.get::<RigidBody>(new_entity).is_some());
        assert!(world.get::<GlobalTransform2D>(new_entity).is_some());
    }

    #[test]
    fn test_capture_includes_all_component_types() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::default()).ok();
        world.add_component(&entity, GlobalTransform2D::default()).ok();
        world.add_component(&entity, Name::new("All")).ok();
        world.add_component(&entity, common::Camera::default()).ok();
        world.add_component(&entity, Sprite::default()).ok();
        world.add_component(&entity, SpriteAnimation::default()).ok();
        world.add_component(&entity, RigidBody::default()).ok();
        world.add_component(&entity, Collider::default()).ok();
        world.add_component(&entity, AudioSource::default()).ok();
        world.add_component(&entity, AudioListener::default()).ok();

        let captured = capture_all_components(&world, entity);
        assert_eq!(captured.len(), 10);
    }
}
