//! Tests for entity generation tracking and validation.

use ecs::prelude::*;
use ecs::generation::{EntityGeneration, EntityReference, EntityIdGenerator};

#[test]
fn test_entity_generation_creation() {
    let generation = EntityGeneration::new();
    assert_eq!(generation.generation(), 1);
    assert!(generation.is_alive());
    assert!(generation.is_valid(1));
    assert!(!generation.is_valid(2));
}

#[test]
fn test_entity_generation_lifecycle() {
    let mut generation = EntityGeneration::new();
    
    // Mark as dead
    generation.mark_dead();
    assert!(!generation.is_alive());
    assert!(!generation.is_valid(1));
    
    // Increment generation (entity reuse)
    generation.increment();
    assert!(generation.is_alive());
    assert_eq!(generation.generation(), 2);
    assert!(generation.is_valid(2));
    assert!(!generation.is_valid(1));
}

#[test]
fn test_entity_id_generator() {
    let generator = EntityIdGenerator::new();
    
    let id1 = generator.generate_id();
    let id2 = generator.generate_id();
    let gen1 = generator.generate_generation();
    let gen2 = generator.generate_generation();
    
    assert_ne!(id1, id2);
    assert_ne!(gen1, gen2);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(gen1, 1);
    assert_eq!(gen2, 2);
}

#[test]
fn test_entity_reference() {
    let reference = EntityReference::new(42, 5);
    
    assert_eq!(reference.id(), 42);
    assert_eq!(reference.generation(), 5);
    assert!(reference.is_valid(5));
    assert!(!reference.is_valid(4));
    assert!(!reference.is_valid(6));
}

#[test]
fn test_world_entity_generation_tracking() {
    let mut world = World::new();
    
    // Create an entity
    let entity_id = world.create_entity();
    let generation = world.get_entity_generation(&entity_id).unwrap();
    
    assert_eq!(generation.generation(), entity_id.generation());
    assert!(generation.is_alive());
    
    // Validate the entity
    world.validate_entity(&entity_id).unwrap();
}

#[test]
fn test_world_entity_validation_after_removal() {
    let mut world = World::new();
    
    // Create and remove an entity
    let entity_id = world.create_entity();
    world.remove_entity(&entity_id).unwrap();
    
    // Validation should fail after removal
    let result = world.validate_entity(&entity_id);
    assert!(result.is_err());
}

#[test]
fn test_world_entity_reference_validation() {
    let mut world = World::new();
    
    // Create an entity
    let entity_id = world.create_entity();
    let reference = entity_id.to_reference();
    
    // Reference should be valid
    assert!(world.is_entity_reference_valid(&reference));
    
    // Remove the entity
    world.remove_entity(&entity_id).unwrap();
    
    // Reference should no longer be valid
    assert!(!world.is_entity_reference_valid(&reference));
}

#[test]
fn test_world_entity_generation_after_reuse() {
    let mut world = World::new();
    
    // Create and remove an entity
    let entity_id1 = world.create_entity();
    let generation1 = entity_id1.generation();
    world.remove_entity(&entity_id1).unwrap();
    
    // Create a new entity (will get a new ID, not reuse the old one)
    let entity_id2 = world.create_entity();
    let generation2 = entity_id2.generation();
    
    // The IDs should be different for different entities
    assert_ne!(entity_id1.value(), entity_id2.value());
    
    // Both should have generation 1 (since we don't reuse entity IDs yet)
    assert_eq!(generation1, 1);
    assert_eq!(generation2, 1);
    
    // The references should be different
    assert_ne!(entity_id1.to_reference(), entity_id2.to_reference());
}

#[test]
fn test_entity_generation_error_handling() {
    let entity_id = EntityId::with_generation(42, 1);
    let generation = EntityGeneration::with_generation(2); // Different generation
    
    let result = entity_id.validate(&generation);
    assert!(result.is_err());
    
    // Test with dead entity
    let mut dead_generation = EntityGeneration::new();
    dead_generation.mark_dead();
    
    let result = entity_id.validate(&dead_generation);
    assert!(result.is_err());
}

#[test]
fn test_world_entity_operations_with_generation() {
    let mut world = World::new();
    world.initialize().unwrap();
    world.start().unwrap();
    
    // Create entity with component
    let entity_id = world.create_entity();
    world.add_component(&entity_id, TestComponent { value: 42 }).unwrap();
    
    // Verify entity is valid
    world.validate_entity(&entity_id).unwrap();
    assert!(world.has_component::<TestComponent>(&entity_id).unwrap());
    
    // Remove entity
    world.remove_entity(&entity_id).unwrap();
    
    // Entity should no longer be valid
    let result = world.validate_entity(&entity_id);
    assert!(result.is_err());
    
    // Component should also be removed
    let result = world.has_component::<TestComponent>(&entity_id);
    assert!(result.is_err()); // Entity not found
}

#[test]
fn test_entity_reuse_detection() {
    let mut world = World::new();
    
    // Create and remove an entity
    let entity_id1 = world.create_entity();
    let reference1 = entity_id1.to_reference();
    world.remove_entity(&entity_id1).unwrap();
    
    // Create a new entity (might reuse the ID)
    let entity_id2 = world.create_entity();
    let reference2 = entity_id2.to_reference();
    
    // The references should be different even if IDs are the same
    if entity_id1.value() == entity_id2.value() {
        assert_ne!(reference1.generation(), reference2.generation());
        assert!(!world.is_entity_reference_valid(&reference1));
        assert!(world.is_entity_reference_valid(&reference2));
    }
}

// Test component for entity operations
#[derive(Debug)]
struct TestComponent {
    value: i32,
}