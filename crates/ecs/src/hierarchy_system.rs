//! Transform hierarchy propagation system
//!
//! This module provides a system that propagates transforms through the entity hierarchy,
//! updating GlobalTransform2D components based on the local Transform2D and parent transforms.

use crate::entity::EntityId;
use crate::hierarchy::{GlobalTransform2D, Parent};
use crate::sprite_components::Transform2D;
use crate::system::System;
use crate::world::World;

/// System that propagates transforms through the entity hierarchy
///
/// This system updates GlobalTransform2D components for all entities with Transform2D.
/// For root entities (no Parent component), GlobalTransform2D equals Transform2D.
/// For child entities, GlobalTransform2D is computed by multiplying the parent's
/// GlobalTransform2D with the child's local Transform2D.
///
/// # Usage
///
/// Add this system to your world and ensure entities have:
/// - `Transform2D` - local transform relative to parent (or world if no parent)
/// - `GlobalTransform2D` - will be computed automatically
/// - `Parent` (optional) - if present, transform is relative to parent
///
/// # Example
///
/// ```ignore
/// use ecs::prelude::*;
/// use ecs::hierarchy_system::TransformHierarchySystem;
///
/// let mut world = World::new();
/// world.add_system(TransformHierarchySystem::new());
///
/// // Create parent
/// let parent = world.create_entity();
/// world.add_component(&parent, Transform2D::new(Vec2::new(100.0, 0.0))).ok();
/// world.add_component(&parent, GlobalTransform2D::default()).ok();
///
/// // Create child
/// let child = world.create_entity();
/// world.add_component(&child, Transform2D::new(Vec2::new(50.0, 0.0))).ok();
/// world.add_component(&child, GlobalTransform2D::default()).ok();
/// world.set_parent(child, parent).ok();
///
/// // After update, child's GlobalTransform2D.position will be (150.0, 0.0)
/// ```
pub struct TransformHierarchySystem {
    /// Whether the system is enabled
    enabled: bool,
}

impl TransformHierarchySystem {
    /// Create a new transform hierarchy system
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Enable or disable the system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for TransformHierarchySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TransformHierarchySystem {
    fn name(&self) -> &str {
        "TransformHierarchySystem"
    }

    fn update(&mut self, world: &mut World, _delta_time: f32) {
        if !self.enabled {
            return;
        }

        // Get all entities - we'll process them in hierarchical order
        let entities: Vec<EntityId> = world.entities();

        // First, update root entities (entities without Parent component)
        for &entity in &entities {
            if world.get::<Parent>(entity).is_none() {
                // Root entity - GlobalTransform equals local Transform
                if let Some(local_transform) = world.get::<Transform2D>(entity) {
                    let global = GlobalTransform2D::from_transform(local_transform);

                    // Update or add GlobalTransform2D
                    if world.get::<GlobalTransform2D>(entity).is_some() {
                        if let Some(global_transform) = world.get_mut::<GlobalTransform2D>(entity) {
                            *global_transform = global;
                        }
                    } else {
                        world.add_component(&entity, global).ok();
                    }
                }
            }
        }

        // Then, propagate to children in hierarchical order
        // We need to process entities level by level to ensure parents are updated before children
        let root_entities = world.get_root_entities();
        for root in root_entities {
            self.propagate_transforms(world, root);
        }
    }

    fn initialize(&mut self, _world: &mut World) -> Result<(), String> {
        log::debug!("TransformHierarchySystem initialized");
        Ok(())
    }

    fn shutdown(&mut self, _world: &mut World) -> Result<(), String> {
        log::debug!("TransformHierarchySystem shut down");
        Ok(())
    }
}

impl TransformHierarchySystem {
    /// Recursively propagate transforms from parent to children
    fn propagate_transforms(&self, world: &mut World, entity: EntityId) {
        // Get the parent's global transform (if this entity has children)
        let parent_global = world
            .get::<GlobalTransform2D>(entity)
            .cloned()
            .unwrap_or_default();

        // Get children of this entity
        let children: Vec<EntityId> = world
            .get_children(entity)
            .map(|c| c.to_vec())
            .unwrap_or_default();

        // Update each child's global transform
        for child in children {
            if let Some(local_transform) = world.get::<Transform2D>(child) {
                let child_global = parent_global.mul_transform(local_transform);

                // Update or add GlobalTransform2D on child
                if world.get::<GlobalTransform2D>(child).is_some() {
                    if let Some(global_transform) = world.get_mut::<GlobalTransform2D>(child) {
                        *global_transform = child_global;
                    }
                } else {
                    world.add_component(&child, child_global).ok();
                }
            }

            // Recursively propagate to grandchildren
            self.propagate_transforms(world, child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_root_entity_transform_propagation() {
        let mut world = World::new();

        // Create a root entity
        let root = world.create_entity();
        world
            .add_component(&root, Transform2D::new(Vec2::new(100.0, 50.0)))
            .unwrap();
        world
            .add_component(&root, GlobalTransform2D::default())
            .unwrap();

        // Run the system
        let mut system = TransformHierarchySystem::new();
        system.update(&mut world, 0.016);

        // Check global transform equals local transform for root
        let global = world.get::<GlobalTransform2D>(root).unwrap();
        assert_eq!(global.position, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_child_entity_transform_propagation() {
        let mut world = World::new();

        // Create parent at (100, 50)
        let parent = world.create_entity();
        world
            .add_component(&parent, Transform2D::new(Vec2::new(100.0, 50.0)))
            .unwrap();
        world
            .add_component(&parent, GlobalTransform2D::default())
            .unwrap();

        // Create child at local (20, 10)
        let child = world.create_entity();
        world
            .add_component(&child, Transform2D::new(Vec2::new(20.0, 10.0)))
            .unwrap();
        world
            .add_component(&child, GlobalTransform2D::default())
            .unwrap();

        // Set up hierarchy
        world.set_parent(child, parent).unwrap();

        // Run the system
        let mut system = TransformHierarchySystem::new();
        system.update(&mut world, 0.016);

        // Check child's global transform: (100, 50) + (20, 10) = (120, 60)
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.position.x - 120.0).abs() < 0.001);
        assert!((child_global.position.y - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_grandchild_transform_propagation() {
        let mut world = World::new();

        // Create grandparent at (100, 0)
        let grandparent = world.create_entity();
        world
            .add_component(&grandparent, Transform2D::new(Vec2::new(100.0, 0.0)))
            .unwrap();
        world
            .add_component(&grandparent, GlobalTransform2D::default())
            .unwrap();

        // Create parent at local (50, 0)
        let parent = world.create_entity();
        world
            .add_component(&parent, Transform2D::new(Vec2::new(50.0, 0.0)))
            .unwrap();
        world
            .add_component(&parent, GlobalTransform2D::default())
            .unwrap();
        world.set_parent(parent, grandparent).unwrap();

        // Create child at local (25, 0)
        let child = world.create_entity();
        world
            .add_component(&child, Transform2D::new(Vec2::new(25.0, 0.0)))
            .unwrap();
        world
            .add_component(&child, GlobalTransform2D::default())
            .unwrap();
        world.set_parent(child, parent).unwrap();

        // Run the system
        let mut system = TransformHierarchySystem::new();
        system.update(&mut world, 0.016);

        // Check grandparent: (100, 0)
        let gp_global = world.get::<GlobalTransform2D>(grandparent).unwrap();
        assert!((gp_global.position.x - 100.0).abs() < 0.001);

        // Check parent: (100, 0) + (50, 0) = (150, 0)
        let p_global = world.get::<GlobalTransform2D>(parent).unwrap();
        assert!((p_global.position.x - 150.0).abs() < 0.001);

        // Check child: (150, 0) + (25, 0) = (175, 0)
        let c_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((c_global.position.x - 175.0).abs() < 0.001);
    }

    #[test]
    fn test_scaled_parent_transform_propagation() {
        let mut world = World::new();

        // Create parent at (0, 0) with scale 2x
        let parent = world.create_entity();
        world
            .add_component(
                &parent,
                Transform2D::new(Vec2::ZERO).with_scale(Vec2::new(2.0, 2.0)),
            )
            .unwrap();
        world
            .add_component(&parent, GlobalTransform2D::default())
            .unwrap();

        // Create child at local (10, 10)
        let child = world.create_entity();
        world
            .add_component(&child, Transform2D::new(Vec2::new(10.0, 10.0)))
            .unwrap();
        world
            .add_component(&child, GlobalTransform2D::default())
            .unwrap();
        world.set_parent(child, parent).unwrap();

        // Run the system
        let mut system = TransformHierarchySystem::new();
        system.update(&mut world, 0.016);

        // Child's global position should be (10, 10) * 2 = (20, 20)
        let child_global = world.get::<GlobalTransform2D>(child).unwrap();
        assert!((child_global.position.x - 20.0).abs() < 0.001);
        assert!((child_global.position.y - 20.0).abs() < 0.001);
        // Child's global scale should be 2x
        assert!((child_global.scale.x - 2.0).abs() < 0.001);
        assert!((child_global.scale.y - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_disabled_system_does_nothing() {
        let mut world = World::new();

        let entity = world.create_entity();
        world
            .add_component(&entity, Transform2D::new(Vec2::new(100.0, 50.0)))
            .unwrap();
        world
            .add_component(&entity, GlobalTransform2D::default())
            .unwrap();

        // Disable the system
        let mut system = TransformHierarchySystem::new();
        system.set_enabled(false);
        system.update(&mut world, 0.016);

        // GlobalTransform should remain at default (0, 0) since system didn't run
        let global = world.get::<GlobalTransform2D>(entity).unwrap();
        assert_eq!(global.position, Vec2::ZERO);
    }
}
