use ecs::prelude::*;

#[test]
fn test_component_trait() {
    // Define a simple test component
    #[derive(Debug)]
    struct TestComponent {
        #[allow(dead_code)]
        value: i32,
    }

    // Create an instance of the component
    let component = TestComponent { value: 42 };

    assert_eq!(
        component.type_name(),
        std::any::type_name::<TestComponent>()
    );
}

#[test]
fn test_component_in_world() {
    // Test adding and retrieving components in a world
    let mut world = World::new();
    let entity_id = world.create_entity();

    // Define a simple test component
    #[derive(Debug)]
    struct TestComponent {
        #[allow(dead_code)]
        value: i32,
    }

    // Add the component to the entity
    world
        .add_component(&entity_id, TestComponent { value: 42 })
        .unwrap();

    // Check if the entity has the component
    let has_component = world.has_component::<TestComponent>(&entity_id).unwrap();

    assert!(has_component);

    // Get the component (this is a bit tricky since we need to downcast)
    let _component = world.get_component::<TestComponent>(&entity_id).unwrap();

    // Note: We can't directly access the value since we get a &dyn Component
    // In a real test, we would need to implement methods on TestComponent to access its value
}

#[test]
fn test_multiple_components() {
    // Test adding multiple different components to an entity
    let mut world = World::new();
    let entity_id = world.create_entity();

    // Define two different test components
    #[derive(Debug)]
    struct PositionComponent {
        #[allow(dead_code)]
        x: f32,
        #[allow(dead_code)]
        y: f32,
    }

    #[derive(Debug)]
    struct VelocityComponent {
        #[allow(dead_code)]
        vx: f32,
        #[allow(dead_code)]
        vy: f32,
    }

    // Add both components to the entity
    world
        .add_component(&entity_id, PositionComponent { x: 10.0, y: 20.0 })
        .unwrap();
    world
        .add_component(&entity_id, VelocityComponent { vx: 1.0, vy: 2.0 })
        .unwrap();

    // Check if the entity has both components
    let has_position = world
        .has_component::<PositionComponent>(&entity_id)
        .unwrap();
    let has_velocity = world
        .has_component::<VelocityComponent>(&entity_id)
        .unwrap();

    assert!(has_position);
    assert!(has_velocity);
}
