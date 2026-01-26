//! Component registry for unified component definitions
//!
//! This module provides macros and traits for defining components that work
//! seamlessly with the ECS, scene serialization, and editor inspection.

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
}
