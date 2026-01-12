
#[test]
fn test_init() {
    // Test initializing the ECS
    let result = ecs::init();

    assert!(result.is_ok());

    // Get the world from the result
    let world = result.unwrap();

    assert_eq!(world.entity_count(), 0);
    assert_eq!(world.system_count(), 0);
}

#[test]
fn test_init_and_use() {
    // Test initializing the ECS and using the world
    let result = ecs::init();
    assert!(result.is_ok());

    // Get the world from the result
    let mut world = result.unwrap();

    // Create an entity
    let entity_id = world.create_entity();

    assert_eq!(world.entity_count(), 1);

    // Define a simple test component
    #[derive(Debug)]
    struct TestComponent {
        #[allow(dead_code)]
        value: i32,
    }

    // Add a component to the entity
    let result = world.add_component(&entity_id, TestComponent { value: 42 });

    assert!(result.is_ok());

    // Check if the entity has the component
    let has_component = world.has_component::<TestComponent>(&entity_id);

    assert!(has_component.is_ok());
    assert!(has_component.unwrap());
}
