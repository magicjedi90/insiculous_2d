use ecs::prelude::*;

#[test]
fn test_entity_creation() {
    // Test creating a new entity
    let entity = Entity::new();

    // TODO: Assert that the entity is properly initialized
    assert!(entity.is_active());
}

#[test]
fn test_entity_id_uniqueness() {
    // Test that entity IDs are unique
    let entity1 = Entity::new();
    let entity2 = Entity::new();

    // TODO: Assert that the entity IDs are unique
    assert_ne!(entity1.id(), entity2.id());
}

#[test]
fn test_entity_active_state() {
    // Test setting the active state of an entity
    let mut entity = Entity::new();

    // Entity should be active by default
    assert!(entity.is_active());

    // Set the entity to inactive
    entity.set_active(false);

    // TODO: Assert that the entity is now inactive
    assert!(!entity.is_active());

    // Set the entity back to active
    entity.set_active(true);

    // TODO: Assert that the entity is active again
    assert!(entity.is_active());
}

#[test]
fn test_entity_id_value() {
    // Test getting the raw value of an entity ID
    let entity = Entity::new();
    let id = entity.id();

    // TODO: Assert that the ID value is non-zero
    assert!(id.value() > 0);
}

#[test]
fn test_entity_id_display() {
    // Test the Display implementation for EntityId
    let entity = Entity::new();
    let id = entity.id();

    // TODO: Assert that the string representation is as expected
    let id_string = format!("{}", id);
    assert!(id_string.starts_with("Entity("));
    assert!(id_string.ends_with(")"));
}
