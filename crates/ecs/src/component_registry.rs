//! Component registry for unified component definitions
//!
//! This module provides macros and traits for defining components that work
//! seamlessly with the ECS, scene serialization, and editor inspection.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::OnceLock;

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
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Register a component type
    pub fn register<T: ComponentMeta + 'static>(&mut self) {
        self.types.insert(T::type_name(), TypeId::of::<T>());
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
    fn test_global_registry_has_builtin_components() {
        let registry = global_registry();

        assert!(registry.is_registered("Transform2D"));
        assert!(registry.is_registered("Sprite"));
        assert!(registry.is_registered("SpriteAnimation"));
        assert!(registry.is_registered("Camera"));
    }
}
