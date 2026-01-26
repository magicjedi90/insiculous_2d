//! Component registry for unified component definitions
//!
//! This module provides macros and traits for defining components that work
//! seamlessly with the ECS, scene serialization, and editor inspection.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Factory function type for creating components from JSON
pub type ComponentFactoryFn = fn(serde_json::Value) -> Result<Box<dyn Any + Send + Sync>, String>;

/// Metadata about a component type for editor inspection and serialization
pub trait ComponentMeta: Send + Sync + 'static {
    /// The component's display name (e.g., "Transform2D")
    fn type_name() -> &'static str
    where
        Self: Sized;

    /// Field names for editor inspection
    fn field_names() -> &'static [&'static str]
    where
        Self: Sized;
}

/// Global registry of component types
static COMPONENT_REGISTRY: OnceLock<ComponentRegistry> = OnceLock::new();

/// Runtime registry for component type lookup by name
pub struct ComponentRegistry {
    types: HashMap<&'static str, TypeId>,
    factories: HashMap<&'static str, ComponentFactoryFn>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            factories: HashMap::new(),
        }
    }

    /// Register a component type with its factory
    pub fn register<T: ComponentMeta + for<'de> serde::Deserialize<'de> + Send + Sync + 'static>(
        &mut self,
    ) {
        let name = T::type_name();
        self.types.insert(name, TypeId::of::<T>());
        self.factories.insert(name, |json| {
            serde_json::from_value::<T>(json)
                .map(|c| Box::new(c) as Box<dyn Any + Send + Sync>)
                .map_err(|e| e.to_string())
        });
    }

    /// Check if a component type is registered
    pub fn is_registered(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get TypeId for a component name
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.types.get(name).copied()
    }

    /// Get all registered type names
    pub fn type_names(&self) -> impl Iterator<Item = &&'static str> {
        self.types.keys()
    }

    /// Create a component by name from JSON
    pub fn create_component(
        &self,
        name: &str,
        json: serde_json::Value,
    ) -> Result<Box<dyn Any + Send + Sync>, String> {
        self.factories
            .get(name)
            .ok_or_else(|| format!("Unknown component type: {}", name))
            .and_then(|factory| factory(json))
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get or initialize the global component registry
pub fn global_registry() -> &'static ComponentRegistry {
    COMPONENT_REGISTRY.get_or_init(|| {
        let mut registry = ComponentRegistry::new();

        // Register built-in ECS components
        use crate::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
        registry.register::<Transform2D>();
        registry.register::<Sprite>();
        registry.register::<SpriteAnimation>();
        registry.register::<Camera>();

        registry
    })
}

/// Simple macro for defining components with standard derives
#[macro_export]
macro_rules! define_component {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                pub $field:ident : $type:ty = $default:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $type,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                }
            }
        }

        impl $crate::component_registry::ComponentMeta for $name {
            fn type_name() -> &'static str {
                stringify!($name)
            }

            fn field_names() -> &'static [&'static str] {
                &[ $( stringify!($field), )* ]
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    define_component! {
        /// Test component for unit tests
        pub struct TestComponent {
            pub value: f32 = 1.0,
            pub name: String = String::new(),
        }
    }

    #[test]
    fn test_define_component_creates_struct() {
        let component = TestComponent::default();
        assert_eq!(component.value, 1.0);
        assert_eq!(component.name, "");
    }

    #[test]
    fn test_define_component_custom_values() {
        let component = TestComponent {
            value: 42.0,
            name: "test".to_string(),
        };
        assert_eq!(component.value, 42.0);
        assert_eq!(component.name, "test");
    }

    #[test]
    fn test_component_meta_type_name() {
        assert_eq!(TestComponent::type_name(), "TestComponent");
    }

    #[test]
    fn test_component_meta_field_names() {
        let fields = TestComponent::field_names();
        assert_eq!(fields, &["value", "name"]);
    }

    #[test]
    fn test_component_serialization() {
        let component = TestComponent {
            value: 3.14,
            name: "serialized".to_string(),
        };

        let json = serde_json::to_string(&component).expect("serialize");
        let parsed: TestComponent = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.value, 3.14);
        assert_eq!(parsed.name, "serialized");
    }

    #[test]
    fn test_registry_register_and_lookup() {
        let mut registry = ComponentRegistry::new();
        registry.register::<TestComponent>();

        assert!(registry.is_registered("TestComponent"));
        assert!(!registry.is_registered("NonExistent"));
    }

    #[test]
    fn test_registry_type_names() {
        let mut registry = ComponentRegistry::new();
        registry.register::<TestComponent>();

        let names: Vec<_> = registry.type_names().collect();
        assert!(names.contains(&&"TestComponent"));
    }

    #[test]
    fn test_component_factory_creates_from_json() {
        use serde_json::json;

        let mut registry = ComponentRegistry::new();
        registry.register::<TestComponent>();

        let json = json!({
            "value": 42.0,
            "name": "factory_test"
        });

        let result = registry.create_component("TestComponent", json);
        assert!(result.is_ok());

        let component = result.unwrap();
        let test_component = component.downcast_ref::<TestComponent>();
        assert!(test_component.is_some());
        assert_eq!(test_component.unwrap().value, 42.0);
        assert_eq!(test_component.unwrap().name, "factory_test");
    }

    #[test]
    fn test_component_factory_unknown_type() {
        let registry = ComponentRegistry::new();

        let result = registry.create_component("NonExistent", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown component type"));
    }

    #[test]
    fn test_global_registry_has_builtin_components() {
        let registry = global_registry();

        assert!(registry.is_registered("Transform2D"));
        assert!(registry.is_registered("Sprite"));
        assert!(registry.is_registered("SpriteAnimation"));
        assert!(registry.is_registered("Camera"));
    }
}
