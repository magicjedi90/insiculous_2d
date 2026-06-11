//! Entity CRUD operations for the editor.
//!
//! Pure functions operating on `&mut World` + `&mut Selection` with no UI
//! dependency — fully testable headlessly.

use ecs::sprite_components::{Name, Sprite};
use ecs::hierarchy::GlobalTransform2D;
use ecs::{EntityId, World, WorldHierarchyExt};
use editor::{capture_all_components, restore_components, Selection};
use glam::Vec2;
use physics::components::{Collider, RigidBody, RigidBodyType};

use crate::constants::DUPLICATE_OFFSET;

// Component add/remove and the add-component popup are driven by
// `editor::ComponentKind` — the registry in editor/src/stored_component.rs
// is the single source of truth for editor-visible component types.

/// Create a base entity with Transform2D, GlobalTransform2D, and Name, then select it.
fn create_base_entity(
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    label: &str,
    counter: &mut u32,
) -> EntityId {
    *counter += 1;
    let entity = world.create_entity();
    world.add_component(&entity, common::Transform2D::new(position)).ok();
    world.add_component(&entity, GlobalTransform2D::default()).ok();
    world.add_component(&entity, Name::new(format!("{} {}", label, counter))).ok();
    selection.select(entity);
    entity
}

/// Create an empty entity with Transform2D, GlobalTransform2D, and Name.
pub fn create_empty_entity(
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    counter: &mut u32,
) -> EntityId {
    create_base_entity(world, selection, position, "Entity", counter)
}

/// Create a sprite entity (empty + Sprite).
pub fn create_sprite_entity(
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    counter: &mut u32,
) -> EntityId {
    let entity = create_base_entity(world, selection, position, "Sprite", counter);
    world.add_component(&entity, Sprite::new(0)).ok();
    entity
}

/// Create a camera entity (empty + Camera).
pub fn create_camera_entity(
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    counter: &mut u32,
) -> EntityId {
    let entity = create_base_entity(world, selection, position, "Camera", counter);
    world.add_component(&entity, common::Camera::default()).ok();
    entity
}

/// Create a physics body entity (empty + Sprite + RigidBody + Collider).
pub fn create_physics_body(
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    body_type: RigidBodyType,
    counter: &mut u32,
) -> EntityId {
    let type_label = match body_type {
        RigidBodyType::Static => "StaticBody",
        RigidBodyType::Dynamic => "DynamicBody",
        RigidBodyType::Kinematic => "KinematicBody",
    };
    let entity = create_base_entity(world, selection, position, type_label, counter);
    world.add_component(&entity, Sprite::new(0)).ok();
    world.add_component(&entity, RigidBody::default().with_body_type(body_type)).ok();
    world.add_component(&entity, Collider::default()).ok();
    entity
}

/// Dispatch a menu action string to the appropriate create function.
///
/// Returns `Some(entity_id)` if an entity was created, `None` if the action
/// is not recognized as a create action.
pub fn handle_create_action(
    action: &str,
    world: &mut World,
    selection: &mut Selection,
    position: Vec2,
    counter: &mut u32,
) -> Option<EntityId> {
    match action {
        "Create Empty" => Some(create_empty_entity(world, selection, position, counter)),
        "Create Sprite" => Some(create_sprite_entity(world, selection, position, counter)),
        "Create Camera" => Some(create_camera_entity(world, selection, position, counter)),
        "Create Static Body" => Some(create_physics_body(world, selection, position, RigidBodyType::Static, counter)),
        "Create Dynamic Body" => Some(create_physics_body(world, selection, position, RigidBodyType::Dynamic, counter)),
        "Create Kinematic Body" => Some(create_physics_body(world, selection, position, RigidBodyType::Kinematic, counter)),
        _ => None,
    }
}

/// Delete all selected entities, reparenting their children.
///
/// For each deleted entity:
/// - Children are reparented to the deleted entity's parent (or made roots).
/// - The entity and all its components are removed.
/// - Selection is cleared afterward.
///
/// Used in tests; production code uses command system (`DeleteEntityCommand`).
#[cfg(test)]
pub fn delete_selected_entities(world: &mut World, selection: &mut Selection) {
    let selected: Vec<EntityId> = selection.selected().collect();
    if selected.is_empty() {
        return;
    }

    for &entity in &selected {
        // Get this entity's parent (before removing)
        let parent_id = world.get_parent(entity);

        // Reparent children to grandparent (or make roots)
        if let Some(children) = world.get_children(entity) {
            let child_ids: Vec<EntityId> = children.to_vec();
            for child in child_ids {
                if let Some(new_parent) = parent_id {
                    world.set_parent(child, new_parent).ok();
                } else {
                    world.remove_parent(child).ok();
                }
            }
        }

        // Remove hierarchy links then entity
        world.remove_parent(entity).ok();
        world.remove_entity(&entity).ok();
    }

    selection.clear();
}

/// Duplicate the primary selected entity (and its descendants).
///
/// The duplicate is offset by `(20, -20)` and gets " (Copy)" appended to its name.
/// Children are recursively duplicated with hierarchy preserved.
/// The new top-level entity is selected afterward.
pub fn duplicate_selected_entities(
    world: &mut World,
    selection: &mut Selection,
    counter: &mut u32,
) {
    let primary = match selection.primary() {
        Some(id) => id,
        None => return,
    };

    let parent_id = world.get_parent(primary);
    let new_entity = duplicate_entity_recursive(
        world,
        primary,
        parent_id,
        DUPLICATE_OFFSET,
        counter,
        true,
    );

    if let Some(entity) = new_entity {
        selection.select(entity);
    }
}

/// Recursively duplicate an entity and its children.
///
/// `offset` is applied only to the top-level entity (is_root=true).
fn duplicate_entity_recursive(
    world: &mut World,
    source: EntityId,
    new_parent: Option<EntityId>,
    offset: Vec2,
    counter: &mut u32,
    is_root: bool,
) -> Option<EntityId> {
    *counter += 1;
    let new_entity = world.create_entity();

    // Clone all registry-known component types (hierarchy links excluded —
    // hierarchy is rebuilt explicitly below).
    let stored = capture_all_components(world, source);
    restore_components(world, new_entity, &stored);

    // Offset top-level duplicate's position
    if is_root {
        if let Some(t) = world.get_mut::<common::Transform2D>(new_entity) {
            t.position += offset;
        }
    }

    // Append " (Copy)" to name
    if let Some(name) = world.get::<Name>(new_entity) {
        let new_name = format!("{} (Copy)", name.as_str());
        world.add_component(&new_entity, Name::new(new_name)).ok();
    }

    // Set parent for this duplicate
    if let Some(parent) = new_parent {
        world.set_parent(new_entity, parent).ok();
    }

    // Recurse for children
    if let Some(children) = world.get_children(source) {
        let child_ids: Vec<EntityId> = children.to_vec();
        for child in child_ids {
            duplicate_entity_recursive(
                world,
                child,
                Some(new_entity),
                Vec2::ZERO,
                counter,
                false,
            );
        }
    }

    Some(new_entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_world_and_selection() -> (World, Selection) {
        (World::new(), Selection::new())
    }

    // ==================== Create Tests ====================

    #[test]
    fn test_create_empty_has_transform_and_name() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        assert!(world.get::<common::Transform2D>(entity).is_some());
        assert!(world.get::<GlobalTransform2D>(entity).is_some());
        assert!(world.get::<Name>(entity).is_some());
    }

    #[test]
    fn test_create_empty_at_correct_position() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let pos = Vec2::new(100.0, -50.0);
        let entity = create_empty_entity(&mut world, &mut sel, pos, &mut counter);

        let t = world.get::<common::Transform2D>(entity).unwrap();
        assert_eq!(t.position, pos);
    }

    #[test]
    fn test_create_empty_auto_selects() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        assert_eq!(sel.primary(), Some(entity));
    }

    #[test]
    fn test_create_sprite_has_sprite_component() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_sprite_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        assert!(world.get::<Sprite>(entity).is_some());
        assert!(world.get::<common::Transform2D>(entity).is_some());
    }

    #[test]
    fn test_create_camera_has_camera_component() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_camera_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        assert!(world.get::<common::Camera>(entity).is_some());
        assert!(world.get::<common::Transform2D>(entity).is_some());
    }

    #[test]
    fn test_create_static_body_has_physics() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_physics_body(
            &mut world, &mut sel, Vec2::ZERO, RigidBodyType::Static, &mut counter,
        );

        let rb = world.get::<RigidBody>(entity).unwrap();
        assert_eq!(rb.body_type, RigidBodyType::Static);
        assert!(world.get::<Collider>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
    }

    #[test]
    fn test_create_dynamic_body_has_physics() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_physics_body(
            &mut world, &mut sel, Vec2::ZERO, RigidBodyType::Dynamic, &mut counter,
        );

        let rb = world.get::<RigidBody>(entity).unwrap();
        assert_eq!(rb.body_type, RigidBodyType::Dynamic);
    }

    #[test]
    fn test_create_increments_counter() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        assert_eq!(counter, 1);
        create_sprite_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        assert_eq!(counter, 2);
        create_camera_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        assert_eq!(counter, 3);
    }

    #[test]
    fn test_create_names_are_unique() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let e1 = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let e2 = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        let n1 = world.get::<Name>(e1).unwrap().as_str().to_string();
        let n2 = world.get::<Name>(e2).unwrap().as_str().to_string();
        assert_ne!(n1, n2);
    }

    // ==================== Delete Tests ====================

    #[test]
    fn test_delete_removes_entity() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let entity = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        delete_selected_entities(&mut world, &mut sel);

        assert!(world.get::<common::Transform2D>(entity).is_none());
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_delete_clears_selection() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        delete_selected_entities(&mut world, &mut sel);

        assert!(sel.is_empty());
    }

    #[test]
    fn test_delete_reparents_children_to_grandparent() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let grandparent = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let parent = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let child = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        world.set_parent(parent, grandparent).unwrap();
        world.set_parent(child, parent).unwrap();

        // Select only the parent for deletion
        sel.select(parent);
        delete_selected_entities(&mut world, &mut sel);

        // Child should now be parented to grandparent
        let child_parent = world.get_parent(child);
        assert_eq!(child_parent, Some(grandparent));
    }

    #[test]
    fn test_delete_orphans_children_when_root() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let parent = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let child = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        world.set_parent(child, parent).unwrap();

        // Select only the parent for deletion
        sel.select(parent);
        delete_selected_entities(&mut world, &mut sel);

        // Child should be a root entity now
        assert!(world.get_parent(child).is_none());
        assert!(world.get::<common::Transform2D>(child).is_some());
    }

    #[test]
    fn test_delete_empty_selection_is_noop() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        sel.clear();

        let count_before = world.entity_count();
        delete_selected_entities(&mut world, &mut sel);
        assert_eq!(world.entity_count(), count_before);
    }

    #[test]
    fn test_delete_multiple_selected() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let e1 = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let e2 = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let _e3 = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        sel.select(e1);
        sel.add(e2);
        delete_selected_entities(&mut world, &mut sel);

        // Only e3 should remain
        assert_eq!(world.entity_count(), 1);
    }

    // ==================== Duplicate Tests ====================

    #[test]
    fn test_duplicate_copies_components() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let original = create_sprite_entity(&mut world, &mut sel, Vec2::new(10.0, 20.0), &mut counter);

        sel.select(original);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        let dup = sel.primary().unwrap();
        assert_ne!(dup, original);
        assert!(world.get::<Sprite>(dup).is_some());
        assert!(world.get::<common::Transform2D>(dup).is_some());
        assert!(world.get::<Name>(dup).is_some());
    }

    #[test]
    fn test_duplicate_offsets_position() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let original = create_empty_entity(&mut world, &mut sel, Vec2::new(100.0, 200.0), &mut counter);

        sel.select(original);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        let dup = sel.primary().unwrap();
        let t = world.get::<common::Transform2D>(dup).unwrap();
        assert_eq!(t.position, Vec2::new(120.0, 180.0));
    }

    #[test]
    fn test_duplicate_selects_new_entity() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let original = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        sel.select(original);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        let dup = sel.primary().unwrap();
        assert_ne!(dup, original);
    }

    #[test]
    fn test_duplicate_preserves_original() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let original = create_empty_entity(&mut world, &mut sel, Vec2::new(50.0, 50.0), &mut counter);

        sel.select(original);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        // Original should be untouched
        let t = world.get::<common::Transform2D>(original).unwrap();
        assert_eq!(t.position, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_duplicate_recursive_copies_children() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let parent = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let child = create_empty_entity(&mut world, &mut sel, Vec2::new(10.0, 10.0), &mut counter);
        world.set_parent(child, parent).unwrap();

        sel.select(parent);
        let count_before = world.entity_count();
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        // Should have 2 new entities (parent dup + child dup)
        assert_eq!(world.entity_count(), count_before + 2);
    }

    #[test]
    fn test_duplicate_children_have_correct_parent() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let parent = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        let _child = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        world.set_parent(_child, parent).unwrap();

        sel.select(parent);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        let dup_parent = sel.primary().unwrap();
        let dup_children = world.get_children(dup_parent);
        assert!(dup_children.is_some());
        assert_eq!(dup_children.unwrap().len(), 1);

        // The duplicated child's parent should be the duplicated parent
        let dup_child = dup_children.unwrap()[0];
        assert_eq!(world.get_parent(dup_child), Some(dup_parent));
    }

    #[test]
    fn test_duplicate_name_appends_copy() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        let original = create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);

        sel.select(original);
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);

        let dup = sel.primary().unwrap();
        let name = world.get::<Name>(dup).unwrap();
        assert!(name.as_str().ends_with("(Copy)"), "Name was: {}", name.as_str());
    }

    #[test]
    fn test_duplicate_empty_selection_is_noop() {
        let (mut world, mut sel) = new_world_and_selection();
        let mut counter = 0;
        create_empty_entity(&mut world, &mut sel, Vec2::ZERO, &mut counter);
        sel.clear();

        let count_before = world.entity_count();
        duplicate_selected_entities(&mut world, &mut sel, &mut counter);
        assert_eq!(world.entity_count(), count_before);
    }

    // Component add/remove dispatch tests live with the registry in
    // editor/src/stored_component.rs (and the RigidBody → Collider cascade
    // in editor/src/commands.rs).
}
