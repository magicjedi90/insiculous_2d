use ecs::prelude::*;

#[test]
fn test_world_creation() {
    // Test creating a new world
    let world = World::new();

    assert_eq!(world.entity_count(), 0);
    assert_eq!(world.system_count(), 0);
}

#[test]
fn test_entity_creation_and_removal() {
    // Test creating and removing entities
    let mut world = World::new();

    // Create an entity
    let entity_id = world.create_entity();

    assert_eq!(world.entity_count(), 1);

    // Remove the entity
    let result = world.remove_entity(&entity_id);

    assert!(result.is_ok());
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn test_component_management() {
    // Test adding, getting, and removing components
    let mut world = World::new();
    let entity_id = world.create_entity();

    // Define a simple test component
    #[derive(Debug)]
    struct TestComponent {
        #[allow(dead_code)]
        value: i32,
    }

    // Add the component to the entity
    let result = world.add_component(&entity_id, TestComponent { value: 42 });

    assert!(result.is_ok());

    // Check if the entity has the component
    let has_component = world.has_component::<TestComponent>(&entity_id);

    assert!(has_component.is_ok());
    assert!(has_component.unwrap());

    // Remove the component
    let result = world.remove_component::<TestComponent>(&entity_id);

    assert!(result.is_ok());

    // Check if the entity still has the component
    let has_component = world.has_component::<TestComponent>(&entity_id);

    assert!(has_component.is_ok());
    assert!(!has_component.unwrap());
}

#[test]
fn test_system_management() {
    // Test adding and updating systems
    let mut world = World::new();

    // Create a simple system
    struct TestSystem {
        update_count: usize,
    }

    impl System for TestSystem {
        fn update(&mut self, _world: &mut World, _delta_time: f32) {
            self.update_count += 1;
        }

        fn name(&self) -> &str {
            "TestSystem"
        }
    }

    // Add the system to the world
    world.add_system(TestSystem { update_count: 0 });

    assert_eq!(world.system_count(), 1);

    // Update the systems
    let _ = world.update(0.016);

    // Note: We can't directly check the update_count since we don't have access to the system after it's added
}

#[test]
fn test_world_initialization() {
    // Test initializing the world through the init function
    let result = init();

    assert!(result.is_ok());
    let world = result.unwrap();
    assert_eq!(world.entity_count(), 0);
    assert_eq!(world.system_count(), 0);
}

#[test]
fn test_hierarchy_cycle_detection() {
    // Test that setting a parent that would create a cycle is rejected
    let mut world = World::new();

    // Create a chain: grandparent -> parent -> child
    let grandparent = world.create_entity();
    let parent = world.create_entity();
    let child = world.create_entity();

    // Set up the hierarchy
    assert!(world.set_parent(parent, grandparent).is_ok());
    assert!(world.set_parent(child, parent).is_ok());

    // Attempt to create a cycle: grandparent -> child (child is an ancestor of grandparent via parent)
    // This would create: child -> parent -> grandparent -> child (cycle!)
    let result = world.set_parent(grandparent, child);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn test_hierarchy_self_parent_rejected() {
    // Test that an entity cannot be its own parent
    let mut world = World::new();

    let entity = world.create_entity();

    // Attempt to set entity as its own parent
    let result = world.set_parent(entity, entity);

    // This should be rejected - self-parenting creates a trivial cycle
    assert!(result.is_err());
}

#[test]
fn test_query_entities() {
    use ecs::{Single, Pair};
    use ecs::sprite_components::{Transform2D, Sprite};

    let mut world = World::new();

    // Create entities with different component combinations
    let entity_with_transform = world.create_entity();
    world.add_component(&entity_with_transform, Transform2D::default()).unwrap();

    let entity_with_sprite = world.create_entity();
    world.add_component(&entity_with_sprite, Sprite::new(0)).unwrap();

    let entity_with_both = world.create_entity();
    world.add_component(&entity_with_both, Transform2D::default()).unwrap();
    world.add_component(&entity_with_both, Sprite::new(0)).unwrap();

    let entity_with_nothing = world.create_entity();

    // Query for entities with Transform2D
    let with_transform = world.query_entities::<Single<Transform2D>>();
    assert_eq!(with_transform.len(), 2);
    assert!(with_transform.contains(&entity_with_transform));
    assert!(with_transform.contains(&entity_with_both));
    assert!(!with_transform.contains(&entity_with_sprite));
    assert!(!with_transform.contains(&entity_with_nothing));

    // Query for entities with Sprite
    let with_sprite = world.query_entities::<Single<Sprite>>();
    assert_eq!(with_sprite.len(), 2);
    assert!(with_sprite.contains(&entity_with_sprite));
    assert!(with_sprite.contains(&entity_with_both));

    // Query for entities with both Transform2D and Sprite
    let with_both = world.query_entities::<Pair<Transform2D, Sprite>>();
    assert_eq!(with_both.len(), 1);
    assert!(with_both.contains(&entity_with_both));
}

#[test]
fn test_world_clear_removes_entities_and_components() {
    use ecs::sprite_components::Transform2D;

    let mut world = World::new();
    let e1 = world.create_entity();
    let e2 = world.create_entity();
    world.add_component(&e1, Transform2D::default()).unwrap();
    world.add_component(&e2, Transform2D::default()).unwrap();

    assert_eq!(world.entity_count(), 2);

    world.clear();

    assert_eq!(world.entity_count(), 0);
    assert!(world.get::<Transform2D>(e1).is_none());
    assert!(world.get::<Transform2D>(e2).is_none());
}

#[test]
fn test_create_entity_with_id_preserves_id() {
    let mut world = World::new();
    let id = EntityId::with_generation(42, 3);
    let created = world.create_entity_with_id(id);

    assert_eq!(created, id);
    assert_eq!(created.value(), 42);
    assert_eq!(created.generation(), 3);
    assert_eq!(world.entity_count(), 1);
    assert!(world.get_entity(&id).is_ok());
}

#[test]
fn test_clear_then_create_entity_with_id() {
    use ecs::sprite_components::Transform2D;

    let mut world = World::new();
    let original = world.create_entity();
    world.add_component(&original, Transform2D::new(glam::Vec2::new(10.0, 20.0))).unwrap();

    world.clear();

    // Recreate with same ID
    let restored = world.create_entity_with_id(original);
    world.add_component(&restored, Transform2D::new(glam::Vec2::new(30.0, 40.0))).unwrap();

    assert_eq!(world.entity_count(), 1);
    let t = world.get::<Transform2D>(restored).unwrap();
    assert_eq!(t.position, glam::Vec2::new(30.0, 40.0));
}
