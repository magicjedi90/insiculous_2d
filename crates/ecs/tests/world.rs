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
