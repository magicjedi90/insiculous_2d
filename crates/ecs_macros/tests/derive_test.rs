//! Integration tests for ComponentMeta derive macro.

use ecs_macros::ComponentMeta;

// Import the trait to call methods
// Note: We need to define the trait here since ecs crate depends on ecs_macros
// This is a simplified version for testing
pub trait ComponentMeta {
    fn type_name() -> &'static str
    where
        Self: Sized;
    fn field_names() -> &'static [&'static str]
    where
        Self: Sized;
}

// Test struct using the derive macro
#[derive(Debug, Clone, ComponentMeta)]
pub struct TestComponent {
    pub health: f32,
    pub name: String,
    pub active: bool,
}

#[test]
fn test_type_name_generated() {
    assert_eq!(TestComponent::type_name(), "TestComponent");
}

#[test]
fn test_field_names_generated() {
    let fields = TestComponent::field_names();
    assert_eq!(fields.len(), 3);
    assert!(fields.contains(&"health"));
    assert!(fields.contains(&"name"));
    assert!(fields.contains(&"active"));
}

#[test]
fn test_field_names_order_preserved() {
    let fields = TestComponent::field_names();
    assert_eq!(fields, &["health", "name", "active"]);
}
