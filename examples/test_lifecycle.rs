//! Test example to verify core system initialization and lifecycle management.
//! This example doesn't rely on the renderer, so it can test our core systems.

use engine_core::Scene;
use ecs::prelude::*;

#[derive(Debug)]
struct TestComponent {
    value: i32,
}

/// Main function that tests core functionality without rendering
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    log::info!("=== Testing Core System Lifecycle ===");

    // Test 1: Scene lifecycle
    log::info!("Test 1: Scene Lifecycle");
    let mut scene = Scene::new("Test Scene");
    
    // Verify initial state
    assert_eq!(scene.lifecycle_state(), engine_core::LifecycleState::Created);
    assert!(!scene.is_operational());
    
    // Initialize scene
    scene.initialize()?;
    assert_eq!(scene.lifecycle_state(), engine_core::LifecycleState::Initialized);
    assert!(scene.is_operational());
    
    // Start scene
    scene.start()?;
    assert_eq!(scene.lifecycle_state(), engine_core::LifecycleState::Running);
    assert!(scene.is_operational());
    
    // Stop scene
    scene.stop()?;
    assert_eq!(scene.lifecycle_state(), engine_core::LifecycleState::Initialized);
    assert!(scene.is_operational());
    
    // Shutdown scene
    scene.shutdown()?;
    assert_eq!(scene.lifecycle_state(), engine_core::LifecycleState::ShutDown);
    assert!(!scene.is_operational());
    
    log::info!("✅ Scene lifecycle test passed");

    // Test 2: Entity generation tracking
    log::info!("Test 2: Entity Generation Tracking");
    let mut world = init()?;
    world.initialize()?;
    
    // Create entities
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    assert_ne!(entity1, entity2);
    
    // Add components
    world.add_component(&entity1, TestComponent { value: 42 })?;
    world.add_component(&entity2, TestComponent { value: 100 })?;
    
    // Verify components
    assert!(world.has_component::<TestComponent>(&entity1)?);
    assert!(world.has_component::<TestComponent>(&entity2)?);
    
    // Remove entity and verify generation tracking
    world.remove_entity(&entity1)?;
    
    // Create new entity (should get different ID/generation)
    let entity3 = world.create_entity();
    assert_ne!(entity1, entity3);
    
    // Test entity reference validation
    let reference = entity2.to_reference();
    assert!(world.is_entity_reference_valid(&reference));
    
    log::info!("✅ Entity generation tracking test passed");

    // Test 3: System lifecycle
    log::info!("Test 3: System Lifecycle");
    let mut world2 = init()?;
    
    // Create a simple counter system
    struct CounterSystem {
        count: std::sync::Arc<std::sync::Mutex<i32>>,
    }
    
    impl System for CounterSystem {
        fn update(&mut self, _world: &mut World, _delta_time: f32) {
            *self.count.lock().unwrap() += 1;
        }
        
        fn name(&self) -> &str {
            "CounterSystem"
        }
    }
    
    let update_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    world2.add_system(CounterSystem { count: update_count.clone() });
    
    // Initialize and start world
    world2.initialize()?;
    world2.start()?;
    
    // Run a few updates
    for i in 0..5 {
        world2.update(0.016)?;
    }
    
    // Verify system was updated
    assert_eq!(*update_count.lock().unwrap(), 5);
    
    // Stop and shutdown
    world2.stop()?;
    world2.shutdown()?;
    
    log::info!("✅ System lifecycle test passed");

    // Test 4: Error handling
    log::info!("Test 4: Error Handling");
    let mut scene2 = Scene::new("Error Test Scene");
    
    // Try invalid transition (should fail but not crash)
    let result = scene2.start();
    assert!(result.is_err());
    assert_eq!(scene2.lifecycle_state(), engine_core::LifecycleState::Created);
    
    // Should be able to initialize after error
    scene2.initialize()?;
    assert_eq!(scene2.lifecycle_state(), engine_core::LifecycleState::Initialized);
    
    log::info!("✅ Error handling test passed");

    log::info!("=== All Core System Tests Passed! ===");
    log::info!("The engine foundation is working correctly.");
    log::info!("Ready to move to Phase 2: Core Features!");

    Ok(())
}