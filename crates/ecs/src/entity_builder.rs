//! Fluent entity builder for creating entities with components.

use crate::component::Component;
use crate::entity::EntityId;
use crate::world::World;

/// A builder for creating entities with components in a fluent style.
///
/// Created via [`World::spawn()`]. Components are added immediately
/// on each `.with()` call. Call `.id()` to get the entity ID.
///
/// # Example
/// ```
/// # use ecs::{World, Transform2D, Sprite, Name};
/// # use glam::Vec2;
/// # let mut world = World::new();
/// # let (pos, tex) = (Vec2::ZERO, 0);
/// let entity = world.spawn()
///     .with(Transform2D::new(pos))
///     .with(Sprite::new(tex))
///     .with(Name::new("player"))
///     .id();
/// # assert!(world.has_component::<Name>(&entity).unwrap());
/// ```
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity_id: EntityId,
}

impl<'w> EntityBuilder<'w> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        let entity_id = world.create_entity();
        Self { world, entity_id }
    }

    /// Add a component to the entity being built.
    pub fn with<T: Component>(self, component: T) -> Self {
        if let Err(e) = self.world.add_component(&self.entity_id, component) {
            log::error!("EntityBuilder::with failed for entity {}: {}", self.entity_id, e);
        }
        self
    }

    /// Finish building and return the entity ID.
    pub fn id(self) -> EntityId {
        self.entity_id
    }
}
