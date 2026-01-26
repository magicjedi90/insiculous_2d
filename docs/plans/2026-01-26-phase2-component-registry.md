# Phase 2: Component Registry Foundation

> **Status:** ✅ COMPLETE (2026-01-26)

**Goal:** Create a unified component registry that reduces the 4-file-touch pattern to 1-file-touch when adding new components

**Architecture:** Declarative macro (`macro_rules!`) generates component structs with derives, defaults, and metadata. Scene loader uses registry for type lookup instead of hardcoded enum matches.

**Tech Stack:** Rust macro_rules!, serde, RON

---

## Overview

Currently adding a component requires changes in:
1. `ecs/src/sprite_components.rs` - struct definition
2. `engine_core/src/scene_data.rs` - ComponentData enum variant
3. `engine_core/src/scene_loader.rs` - component_type_name() match
4. `engine_core/src/scene_loader.rs` - add_component_to_entity() match

After this refactor:
1. Single file with `define_component!` macro call

We'll do this incrementally, keeping backward compatibility throughout.

---

## Task 1: Create Component Registry Module

**Files:**
- Create: `crates/ecs/src/component_registry.rs`
- Modify: `crates/ecs/src/lib.rs`

**Step 1: Create the registry module with basic macro**

Create `crates/ecs/src/component_registry.rs`:

```rust
//! Component registry for unified component definitions
//!
//! This module provides macros and traits for defining components that work
//! seamlessly with the ECS, scene serialization, and editor inspection.

use std::any::TypeId;
use std::collections::HashMap;

/// Metadata about a component type for editor inspection and serialization
pub trait ComponentMeta: Send + Sync + 'static {
    /// The component's display name (e.g., "Transform2D")
    fn type_name() -> &'static str where Self: Sized;

    /// Field names for editor inspection
    fn field_names() -> &'static [&'static str] where Self: Sized;
}

/// Simple macro for defining components with standard derives
///
/// # Example
/// ```ignore
/// define_component! {
///     /// A 2D transform component
///     pub struct Transform2D {
///         pub position: Vec2 = Vec2::ZERO,
///         pub rotation: f32 = 0.0,
///         pub scale: Vec2 = Vec2::ONE,
///     }
/// }
/// ```
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

pub use define_component;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

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
```

**Step 2: Add module to lib.rs**

Modify `crates/ecs/src/lib.rs` - add near other module declarations:

```rust
pub mod component_registry;
pub use component_registry::{define_component, ComponentMeta};
```

**Step 3: Run tests to verify macro works**

Run: `cargo test -p ecs component_registry`

Expected: All 5 tests pass

**Step 4: Commit**

```bash
git add crates/ecs/src/component_registry.rs crates/ecs/src/lib.rs
git commit -m "$(cat <<'EOF'
feat: add component registry module with define_component! macro

Provides unified way to define components with:
- Standard derives (Debug, Clone, Serialize, Deserialize)
- Default impl from field defaults
- ComponentMeta trait for type name and field introspection

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Migrate SpriteAnimation as Proof of Concept

**Files:**
- Modify: `crates/ecs/src/sprite_components.rs`
- Test: `crates/ecs/tests/sprite_components.rs`

**Step 1: Verify existing SpriteAnimation tests pass**

Run: `cargo test -p ecs sprite_animation`

Expected: All animation tests pass (baseline)

**Step 2: Refactor SpriteAnimation to use define_component!**

The current SpriteAnimation has methods (`update`, `play`, `pause`, etc.) which can't be generated by the macro. We need to keep the struct definition separate from methods.

Instead of full migration, let's add ComponentMeta impl to existing struct.

Modify `crates/ecs/src/sprite_components.rs` - add after SpriteAnimation impl block (around line 193):

```rust
impl crate::component_registry::ComponentMeta for SpriteAnimation {
    fn type_name() -> &'static str {
        "SpriteAnimation"
    }

    fn field_names() -> &'static [&'static str] {
        &["current_frame", "fps", "playing", "loop_animation", "time_accumulator", "frames"]
    }
}
```

**Step 3: Add test for SpriteAnimation metadata**

Add to `crates/ecs/tests/sprite_components.rs`:

```rust
#[test]
fn test_sprite_animation_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(SpriteAnimation::type_name(), "SpriteAnimation");

    let fields = SpriteAnimation::field_names();
    assert!(fields.contains(&"fps"));
    assert!(fields.contains(&"frames"));
    assert!(fields.contains(&"playing"));
}
```

**Step 4: Run tests**

Run: `cargo test -p ecs sprite_animation`

Expected: All tests pass including new metadata test

**Step 5: Commit**

```bash
git add crates/ecs/src/sprite_components.rs crates/ecs/tests/sprite_components.rs
git commit -m "$(cat <<'EOF'
feat: add ComponentMeta impl for SpriteAnimation

First component to use the new metadata system. Enables editor
inspection to discover field names without hardcoding.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add ComponentMeta to Remaining ECS Components

**Files:**
- Modify: `crates/ecs/src/sprite_components.rs`
- Modify: `crates/common/src/transform.rs` (or wherever Transform2D is)
- Test: `crates/ecs/tests/sprite_components.rs`

**Step 1: Find Transform2D and Camera locations**

These are re-exported from `common` crate. Check `crates/common/src/`.

**Step 2: Add ComponentMeta to Sprite**

Add to `crates/ecs/src/sprite_components.rs` after Sprite impl block:

```rust
impl crate::component_registry::ComponentMeta for Sprite {
    fn type_name() -> &'static str {
        "Sprite"
    }

    fn field_names() -> &'static [&'static str] {
        &["offset", "rotation", "scale", "tex_region", "color", "depth", "texture_handle"]
    }
}
```

**Step 3: Add ComponentMeta to Transform2D**

First, check where Transform2D is defined. If in `common` crate, we need to either:
- Add ecs as a dependency to common (creates cycle - bad)
- Define ComponentMeta in common crate (cleaner)
- Or implement it in ecs using extension trait

For simplicity, add to `crates/ecs/src/sprite_components.rs` since Transform2D is re-exported there:

```rust
impl crate::component_registry::ComponentMeta for Transform2D {
    fn type_name() -> &'static str {
        "Transform2D"
    }

    fn field_names() -> &'static [&'static str] {
        &["position", "rotation", "scale"]
    }
}

impl crate::component_registry::ComponentMeta for Camera {
    fn type_name() -> &'static str {
        "Camera"
    }

    fn field_names() -> &'static [&'static str] {
        &["position", "rotation", "zoom", "viewport_size", "is_main_camera", "near", "far"]
    }
}
```

**Step 4: Add tests for all component metadata**

Add to `crates/ecs/tests/sprite_components.rs`:

```rust
#[test]
fn test_sprite_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(Sprite::type_name(), "Sprite");
    let fields = Sprite::field_names();
    assert!(fields.contains(&"texture_handle"));
    assert!(fields.contains(&"color"));
}

#[test]
fn test_transform2d_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(Transform2D::type_name(), "Transform2D");
    let fields = Transform2D::field_names();
    assert!(fields.contains(&"position"));
    assert!(fields.contains(&"rotation"));
    assert!(fields.contains(&"scale"));
}

#[test]
fn test_camera_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(Camera::type_name(), "Camera");
    let fields = Camera::field_names();
    assert!(fields.contains(&"zoom"));
    assert!(fields.contains(&"viewport_size"));
}
```

**Step 5: Run all tests**

Run: `cargo test -p ecs`

Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/ecs/src/sprite_components.rs crates/ecs/tests/sprite_components.rs
git commit -m "$(cat <<'EOF'
feat: add ComponentMeta to Sprite, Transform2D, and Camera

All core ECS components now provide metadata for editor inspection.
This enables generic component display without hardcoded field lists.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Create Component Type Registry

**Files:**
- Modify: `crates/ecs/src/component_registry.rs`

**Step 1: Add runtime type registry**

Add to `crates/ecs/src/component_registry.rs`:

```rust
use std::sync::OnceLock;

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
        // Register built-in components
        // (These will be added as we migrate components)
        registry
    })
}
```

**Step 2: Add tests for registry**

Add to tests in `component_registry.rs`:

```rust
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
```

**Step 3: Run tests**

Run: `cargo test -p ecs component_registry`

Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/ecs/src/component_registry.rs
git commit -m "$(cat <<'EOF'
feat: add ComponentRegistry for runtime type lookup

Enables looking up component types by name string, which is needed
for scene serialization to work without hardcoded enum matches.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Wire Up Component Registration

**Files:**
- Modify: `crates/ecs/src/component_registry.rs`
- Modify: `crates/ecs/src/lib.rs`

**Step 1: Register built-in components in global registry**

Modify `global_registry()` in `crates/ecs/src/component_registry.rs`:

```rust
/// Get or initialize the global component registry
pub fn global_registry() -> &'static ComponentRegistry {
    COMPONENT_REGISTRY.get_or_init(|| {
        let mut registry = ComponentRegistry::new();

        // Register built-in ECS components
        use crate::sprite_components::{Sprite, SpriteAnimation, Transform2D, Camera};
        registry.register::<Transform2D>();
        registry.register::<Sprite>();
        registry.register::<SpriteAnimation>();
        registry.register::<Camera>();

        registry
    })
}
```

**Step 2: Export global_registry from lib.rs**

Add to `crates/ecs/src/lib.rs`:

```rust
pub use component_registry::global_registry;
```

**Step 3: Add integration test**

Add to `crates/ecs/src/component_registry.rs` tests:

```rust
#[test]
fn test_global_registry_has_builtin_components() {
    let registry = global_registry();

    assert!(registry.is_registered("Transform2D"));
    assert!(registry.is_registered("Sprite"));
    assert!(registry.is_registered("SpriteAnimation"));
    assert!(registry.is_registered("Camera"));
}
```

**Step 4: Run tests**

Run: `cargo test -p ecs`

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/ecs/src/component_registry.rs crates/ecs/src/lib.rs
git commit -m "$(cat <<'EOF'
feat: register built-in components in global registry

Transform2D, Sprite, SpriteAnimation, and Camera are now automatically
registered and discoverable by name at runtime.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Verify Full Test Suite

**Step 1: Run all workspace tests**

Run: `cargo test --workspace`

Expected: All tests pass

**Step 2: Run hello_world example**

Run: `cargo run --example hello_world`

Verify: Game runs, scene loads, entities behave correctly

**Step 3: Document in training.md**

If time permits, add a section to `training.md` about the new component registry pattern.

---

## Success Criteria

- [ ] `define_component!` macro generates structs with proper derives
- [ ] `ComponentMeta` trait provides type name and field names
- [ ] All core components (Transform2D, Sprite, SpriteAnimation, Camera) have ComponentMeta impls
- [ ] `ComponentRegistry` can lookup types by name
- [ ] `global_registry()` includes all built-in components
- [ ] All workspace tests pass
- [ ] hello_world example runs correctly

---

## Future Work (Phase 3+)

This phase establishes the foundation. Future phases will:

1. **Phase 3a**: Update scene loader to use registry for component lookup (reduces one match arm)
2. **Phase 3b**: Create generic editor inspector using ComponentMeta
3. **Phase 3c**: Add derive macro for automatic ComponentMeta generation
4. **Phase 3d**: Consider serde-based scene format that doesn't need ComponentData enum
