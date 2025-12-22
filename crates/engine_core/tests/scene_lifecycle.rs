//! Tests for scene lifecycle management.

use engine_core::{Scene, LifecycleState};
use ecs::prelude::*;
use ecs::system::SystemRegistry;

#[test]
fn test_scene_lifecycle_states() {
    let mut scene = Scene::new("TestScene");
    
    // Initial state
    assert_eq!(scene.lifecycle_state(), LifecycleState::Created);
    assert!(!scene.is_operational());
    assert!(!scene.is_initialized());
    assert!(!scene.is_running());
    
    // Initialize
    scene.initialize().unwrap();
    
    // Debug the state
    let state = scene.lifecycle_state();
    let operational = scene.is_operational();
    let initialized = scene.is_initialized();
    let running = scene.is_running();
    
    println!("State: {:?}, Operational: {}, Initialized: {}, Running: {}", 
             state, operational, initialized, running);
    
    // Test each assertion individually
    assert_eq!(state, LifecycleState::Initialized);
    assert!(operational);
    
    // Use a different assertion approach
    if initialized != true {
        panic!("Expected initialized to be true, got: {:?}", initialized);
    }
    
    assert!(!running);
    
    // Start
    scene.start().unwrap();
    assert_eq!(scene.lifecycle_state(), LifecycleState::Running);
    assert!(scene.is_operational());
    assert!(scene.is_initialized());
    assert!(scene.is_running());
    
    // Stop
    scene.stop().unwrap();
    assert_eq!(scene.lifecycle_state(), LifecycleState::Initialized);
    assert!(scene.is_operational());
    assert!(scene.is_initialized());
    assert!(!scene.is_running());
    
    // Shutdown
    scene.shutdown().unwrap();
    assert_eq!(scene.lifecycle_state(), LifecycleState::ShutDown);
    assert!(!scene.is_operational());
    assert!(!scene.is_initialized());
    assert!(!scene.is_running());
}

#[test]
fn test_scene_update_operational() {
    let mut scene = Scene::new("TestScene");
    
    // Update should fail when not operational
    let result = scene.update(0.016);
    assert!(result.is_err());
    
    // Initialize and start
    scene.initialize().unwrap();
    scene.start().unwrap();
    
    // Update should work now
    scene.update(0.016).unwrap();
    
    // Stop
    scene.stop().unwrap();
    
    // Update should fail again
    let result = scene.update(0.016);
    assert!(result.is_err());
}

#[test]
fn test_scene_with_schedule() {
    let mut scene = Scene::new("TestScene");
    let mut schedule = SystemRegistry::new();
    
    // Add a simple system to the schedule
    let update_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let update_count_clone = update_count.clone();
    
    schedule.add_simple("TestSystem", move |_world, _dt| {
        *update_count_clone.lock().unwrap() += 1;
    });
    
    // Initialize scene and schedule
    scene.initialize().unwrap();
    scene.start().unwrap();
    schedule.initialize().unwrap();
    schedule.start().unwrap();
    
    // Update with schedule
    scene.update_with_schedule(&mut schedule, 0.016).unwrap();
    scene.update_with_schedule(&mut schedule, 0.016).unwrap();
    
    assert_eq!(*update_count.lock().unwrap(), 2);
    
    // Cleanup
    schedule.stop().unwrap();
    schedule.shutdown().unwrap();
    scene.stop().unwrap();
    scene.shutdown().unwrap();
}

#[test]
fn test_scene_lifecycle_errors() {
    let mut scene = Scene::new("TestScene");
    
    // Try to start without initialization
    let result = scene.start();
    assert!(result.is_err());
    
    // Initialize
    scene.initialize().unwrap();
    
    // Try to initialize twice
    let result = scene.initialize();
    assert!(result.is_err());
    
    // Start
    scene.start().unwrap();
    
    // Try to start twice
    let result = scene.start();
    assert!(result.is_err());
    
    // Stop
    scene.stop().unwrap();
    
    // Try to stop twice
    let result = scene.stop();
    assert!(result.is_err());
}

#[test]
fn test_scene_entity_management() {
    let mut scene = Scene::new("EntityTestScene");
    
    // Initialize scene
    scene.initialize().unwrap();
    scene.start().unwrap();
    
    // Create entities
    let entity1 = scene.world.create_entity();
    let entity2 = scene.world.create_entity();
    
    assert_ne!(entity1, entity2);
    assert!(scene.world.validate_entity(&entity1).is_ok());
    assert!(scene.world.validate_entity(&entity2).is_ok());
    
    // Add components
    scene.world.add_component(&entity1, Transform { x: 10.0, y: 20.0 }).unwrap();
    scene.world.add_component(&entity2, Transform { x: 30.0, y: 40.0 }).unwrap();
    
    // Verify components
    assert!(scene.world.has_component::<Transform>(&entity1).unwrap());
    assert!(scene.world.has_component::<Transform>(&entity2).unwrap());
    
    // Remove entity
    scene.world.remove_entity(&entity1).unwrap();
    
    // Entity should no longer be valid
    let result = scene.world.validate_entity(&entity1);
    assert!(result.is_err());
    
    // Component should be removed
    let result = scene.world.has_component::<Transform>(&entity1);
    assert!(result.is_err());
    
    // Other entity should still be valid
    assert!(scene.world.validate_entity(&entity2).is_ok());
    
    // Cleanup
    scene.stop().unwrap();
    scene.shutdown().unwrap();
}

#[test]
fn test_scene_world_lifecycle_integration() {
    let mut scene = Scene::new("IntegrationTest");
    
    // The scene should initialize its world when initialized
    scene.initialize().unwrap();
    assert!(scene.world.is_initialized());
    
    // Start should start the world
    scene.start().unwrap();
    assert!(scene.world.is_running());
    
    // Stop should stop the world
    scene.stop().unwrap();
    assert!(!scene.world.is_running());
    
    // Shutdown should shutdown the world
    scene.shutdown().unwrap();
    assert!(!scene.world.is_initialized());
}

#[test]
fn test_multiple_scenes() {
    let mut scene1 = Scene::new("Scene1");
    let mut scene2 = Scene::new("Scene2");
    
    // Initialize both scenes
    scene1.initialize().unwrap();
    scene2.initialize().unwrap();
    
    // Both should be operational
    assert!(scene1.is_operational());
    assert!(scene2.is_operational());
    
    // Start both
    scene1.start().unwrap();
    scene2.start().unwrap();
    
    // Both should be running
    assert!(scene1.is_running());
    assert!(scene2.is_running());
    
    // Update both
    scene1.update(0.016).unwrap();
    scene2.update(0.016).unwrap();
    
    // Shutdown both
    scene1.shutdown().unwrap();
    scene2.shutdown().unwrap();
}

#[test]
fn test_scene_error_recovery() {
    let mut scene = Scene::new("ErrorTestScene");
    
    // Try an invalid transition (start without initializing)
    let result = scene.start();
    assert!(result.is_err());
    
    // Scene should still be in Created state (error doesn't change state)
    assert_eq!(scene.lifecycle_state(), LifecycleState::Created);
    assert!(!scene.is_operational());
    
    // Should be able to initialize normally after the error
    scene.initialize().unwrap();
    assert!(scene.is_operational());
    assert_eq!(scene.lifecycle_state(), LifecycleState::Initialized);
}

// Test components
#[derive(Debug)]
struct Transform {
    x: f32,
    y: f32,
}