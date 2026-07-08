//! The editor's component registry — the single source of truth for every
//! component type the editor can capture, restore, add, remove, and inspect.
//!
//! All per-component dispatch (undo/redo capture, add-component popup,
//! read-only inspection) is generated from ONE `editor_component_registry!`
//! invocation below. **To make a new component editor-visible, add one line
//! to that invocation** — no match statements elsewhere need to change.

use ecs::audio_components::{AudioListener, AudioSource};
use ecs::behavior::{Behavior, BehaviorState, EntityTag};
use ecs::hierarchy::GlobalTransform2D;
use ecs::sprite_components::{Name, Sprite, SpriteAnimation};
use ecs::{EntityId, World};
use physics::components::{Collider, RigidBody};
use ui::UIContext;

use crate::inspector::{inspect_component, InspectorStyle};

/// Category grouping for the add-component popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentCategory {
    Core,
    Rendering,
    Physics,
    Audio,
    Gameplay,
}

impl ComponentCategory {
    /// All categories in display order.
    pub const ALL: [ComponentCategory; 5] = [
        ComponentCategory::Core,
        ComponentCategory::Rendering,
        ComponentCategory::Physics,
        ComponentCategory::Audio,
        ComponentCategory::Gameplay,
    ];

    /// Display name for the category header.
    pub fn label(self) -> &'static str {
        match self {
            ComponentCategory::Core => "Core",
            ComponentCategory::Rendering => "Rendering",
            ComponentCategory::Physics => "Physics",
            ComponentCategory::Audio => "Audio",
            ComponentCategory::Gameplay => "Gameplay",
        }
    }
}

/// Generates the editor's component dispatch from a single component list.
///
/// Sections:
/// - `hidden`: captured for undo/redo only (always present on entities,
///   never inspected or removable) — e.g. `GlobalTransform2D`, `Name`.
/// - `builtin`: captured AND inspected, but never addable/removable —
///   e.g. `Transform2D`.
/// - `removable`: full lifecycle (capture, inspect, add, remove), each
///   tagged with a `ComponentCategory` for the add-component popup.
macro_rules! editor_component_registry {
    (
        hidden:    [ $( $h:ident => $h_ty:ty ),+ $(,)? ],
        builtin:   [ $( $b:ident => $b_ty:ty ),+ $(,)? ],
        removable: [ $( $r:ident => $r_ty:ty : $cat:ident ),+ $(,)? ] $(,)?
    ) => {
        /// A captured component value for undo/redo storage.
        ///
        /// Each variant stores a cloned concrete component type, avoiding the
        /// need for trait objects and enabling type-safe restore operations.
        #[derive(Debug, Clone)]
        pub enum StoredComponent {
            $( $h($h_ty), )+
            $( $b($b_ty), )+
            $( $r($r_ty), )+
        }

        impl StoredComponent {
            /// Add this stored component to an entity in the world.
            pub fn apply_to(&self, world: &mut World, entity: EntityId) {
                match self {
                    $( Self::$h(c) => { world.add_component(&entity, Clone::clone(c)).ok(); } )+
                    $( Self::$b(c) => { world.add_component(&entity, Clone::clone(c)).ok(); } )+
                    $( Self::$r(c) => { world.add_component(&entity, Clone::clone(c)).ok(); } )+
                }
            }
        }

        /// Capture all known component types from an entity into a `Vec<StoredComponent>`.
        ///
        /// This reads every registered component type and stores any that are present.
        /// Hierarchy components (Parent, Children) are deliberately excluded —
        /// hierarchy is managed separately by the command implementations.
        pub fn capture_all_components(world: &World, entity: EntityId) -> Vec<StoredComponent> {
            let mut components = Vec::new();
            $( if let Some(c) = world.get::<$h_ty>(entity) {
                components.push(StoredComponent::$h(Clone::clone(c)));
            } )+
            $( if let Some(c) = world.get::<$b_ty>(entity) {
                components.push(StoredComponent::$b(Clone::clone(c)));
            } )+
            $( if let Some(c) = world.get::<$r_ty>(entity) {
                components.push(StoredComponent::$r(Clone::clone(c)));
            } )+
            components
        }

        /// The component kinds that can be added to / removed from entities.
        ///
        /// This is THE editor-wide `ComponentKind` — commands, the inspector,
        /// and the add-component popup all share it.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ComponentKind {
            $( $r, )+
        }

        impl ComponentKind {
            /// All removable component kinds, in registry order.
            pub const ALL: &'static [ComponentKind] = &[ $( ComponentKind::$r, )+ ];

            /// Human-readable display name (matches the type name).
            pub fn display_name(self) -> &'static str {
                match self { $( Self::$r => stringify!($r), )+ }
            }

            /// Category for the add-component popup.
            pub fn category(self) -> ComponentCategory {
                match self { $( Self::$r => ComponentCategory::$cat, )+ }
            }

            /// Add a default instance of this component to an entity.
            pub fn add_default(self, world: &mut World, entity: EntityId) {
                match self {
                    $( Self::$r => { world.add_component(&entity, <$r_ty>::default()).ok(); } )+
                }
            }

            /// Capture the current value of this component, if present.
            pub fn capture(self, world: &World, entity: EntityId) -> Option<StoredComponent> {
                match self {
                    $( Self::$r => world.get::<$r_ty>(entity)
                        .map(|c| StoredComponent::$r(Clone::clone(c))), )+
                }
            }

            /// Remove this component from an entity (no-op if absent).
            pub fn remove(self, world: &mut World, entity: EntityId) {
                match self {
                    $( Self::$r => { world.remove_component::<$r_ty>(&entity).ok(); } )+
                }
            }

            /// Whether the entity currently has this component.
            pub fn is_present(self, world: &World, entity: EntityId) -> bool {
                match self {
                    $( Self::$r => world.get::<$r_ty>(entity).is_some(), )+
                }
            }
        }

        /// Render a read-only inspection of every present inspectable component
        /// (builtin + removable), in registry order. Returns the next Y position.
        pub fn inspect_all_components(
            ui: &mut UIContext,
            world: &World,
            entity: EntityId,
            x: f32,
            mut y: f32,
            style: &InspectorStyle,
            section_gap: f32,
        ) -> f32 {
            $( if let Some(c) = world.get::<$b_ty>(entity) {
                y += section_gap;
                y = inspect_component(ui, stringify!($b), c, x, y, style);
            } )+
            $( if let Some(c) = world.get::<$r_ty>(entity) {
                y += section_gap;
                y = inspect_component(ui, stringify!($r), c, x, y, style);
            } )+
            y
        }
    };
}

editor_component_registry! {
    hidden: [
        GlobalTransform2D => GlobalTransform2D,
        Name              => Name,
        BehaviorState     => BehaviorState,
    ],
    builtin: [
        Transform2D => common::Transform2D,
    ],
    removable: [
        Camera          => common::Camera : Core,
        Sprite          => Sprite : Rendering,
        SpriteAnimation => SpriteAnimation : Rendering,
        RigidBody       => RigidBody : Physics,
        Collider        => Collider : Physics,
        AudioSource     => AudioSource : Audio,
        AudioListener   => AudioListener : Audio,
        Behavior        => Behavior : Gameplay,
        EntityTag       => EntityTag : Gameplay,
    ],
}

/// Restore a set of stored components onto an entity.
pub fn restore_components(world: &mut World, entity: EntityId, components: &[StoredComponent]) {
    for component in components {
        component.apply_to(world, entity);
    }
}

/// Returns the component kinds that are NOT present on the entity
/// (the candidates for the add-component popup).
pub fn available_components(world: &World, entity: EntityId) -> Vec<ComponentKind> {
    ComponentKind::ALL
        .iter()
        .copied()
        .filter(|kind| !kind.is_present(world, entity))
        .collect()
}

/// Returns all component kinds grouped by category, in display order.
/// Categories with no components are omitted.
pub fn categorized_components() -> Vec<(ComponentCategory, Vec<ComponentKind>)> {
    ComponentCategory::ALL
        .iter()
        .map(|&category| {
            let kinds: Vec<ComponentKind> = ComponentKind::ALL
                .iter()
                .copied()
                .filter(|kind| kind.category() == category)
                .collect();
            (category, kinds)
        })
        .filter(|(_, kinds)| !kinds.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_capture_empty_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        let captured = capture_all_components(&world, entity);
        assert!(captured.is_empty());
    }

    #[test]
    fn test_capture_and_restore_round_trip() {
        let mut world = World::new();
        let entity = world.create_entity();
        let pos = Vec2::new(42.0, 99.0);
        world.add_component(&entity, common::Transform2D::new(pos)).ok();
        world.add_component(&entity, GlobalTransform2D::default()).ok();
        world.add_component(&entity, Name::new("TestEntity")).ok();
        world.add_component(&entity, Sprite::new(5)).ok();
        world.add_component(&entity, RigidBody::default()).ok();

        let captured = capture_all_components(&world, entity);
        assert_eq!(captured.len(), 5);

        // Create a fresh entity and restore onto it
        let new_entity = world.create_entity();
        restore_components(&mut world, new_entity, &captured);

        let t = world.get::<common::Transform2D>(new_entity).unwrap();
        assert_eq!(t.position, pos);
        assert!(world.get::<Name>(new_entity).is_some());
        assert!(world.get::<Sprite>(new_entity).is_some());
        assert!(world.get::<RigidBody>(new_entity).is_some());
        assert!(world.get::<GlobalTransform2D>(new_entity).is_some());
    }

    #[test]
    fn test_capture_includes_all_component_types() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::default()).ok();
        world.add_component(&entity, GlobalTransform2D::default()).ok();
        world.add_component(&entity, Name::new("All")).ok();
        world.add_component(&entity, common::Camera::default()).ok();
        world.add_component(&entity, Sprite::default()).ok();
        world.add_component(&entity, SpriteAnimation::default()).ok();
        world.add_component(&entity, RigidBody::default()).ok();
        world.add_component(&entity, Collider::default()).ok();
        world.add_component(&entity, AudioSource::default()).ok();
        world.add_component(&entity, AudioListener::default()).ok();
        world.add_component(&entity, Behavior::default()).ok();
        world.add_component(&entity, BehaviorState::default()).ok();
        world.add_component(&entity, EntityTag::default()).ok();

        let captured = capture_all_components(&world, entity);
        assert_eq!(captured.len(), 13);
    }

    #[test]
    fn test_gameplay_components_registered_under_gameplay_category() {
        assert_eq!(ComponentKind::Behavior.category(), ComponentCategory::Gameplay);
        assert_eq!(ComponentKind::EntityTag.category(), ComponentCategory::Gameplay);

        let categories = categorized_components();
        let (_, gameplay_kinds) = categories
            .iter()
            .find(|(c, _)| *c == ComponentCategory::Gameplay)
            .expect("Gameplay category present");
        assert!(gameplay_kinds.contains(&ComponentKind::Behavior));
        assert!(gameplay_kinds.contains(&ComponentKind::EntityTag));
    }

    // ==================== ComponentKind dispatch ====================

    #[test]
    fn test_add_default_creates_each_component_kind() {
        let mut world = World::new();
        let entity = world.create_entity();

        for &kind in ComponentKind::ALL {
            kind.add_default(&mut world, entity);
            assert!(
                kind.is_present(&world, entity),
                "add_default did not add {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_remove_deletes_each_component_kind() {
        let mut world = World::new();
        let entity = world.create_entity();

        for &kind in ComponentKind::ALL {
            kind.add_default(&mut world, entity);
            kind.remove(&mut world, entity);
            assert!(
                !kind.is_present(&world, entity),
                "remove did not delete {:?}",
                kind
            );
        }
    }

    #[test]
    fn test_remove_absent_component_is_safe() {
        let mut world = World::new();
        let entity = world.create_entity();
        // Should not panic
        ComponentKind::Sprite.remove(&mut world, entity);
        assert!(!ComponentKind::Sprite.is_present(&world, entity));
    }

    #[test]
    fn test_capture_returns_value_when_present() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, Sprite::new(7)).ok();

        let stored = ComponentKind::Sprite.capture(&world, entity);
        assert!(matches!(stored, Some(StoredComponent::Sprite(s)) if s.texture_handle == 7));
        assert!(ComponentKind::Camera.capture(&world, entity).is_none());
    }

    #[test]
    fn test_display_names_match_variant_names() {
        assert_eq!(ComponentKind::Camera.display_name(), "Camera");
        assert_eq!(ComponentKind::SpriteAnimation.display_name(), "SpriteAnimation");
        for &kind in ComponentKind::ALL {
            assert!(!kind.display_name().is_empty());
        }
    }

    #[test]
    fn test_available_components_filters_present() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, Sprite::default()).ok();
        world.add_component(&entity, RigidBody::default()).ok();

        let available = available_components(&world, entity);
        assert!(!available.contains(&ComponentKind::Sprite));
        assert!(!available.contains(&ComponentKind::RigidBody));
        assert!(available.contains(&ComponentKind::Camera));
        assert!(available.contains(&ComponentKind::Collider));
        assert!(available.contains(&ComponentKind::AudioSource));
    }

    #[test]
    fn test_categorized_components_covers_all_kinds() {
        let categories = categorized_components();
        let all: Vec<ComponentKind> = categories
            .iter()
            .flat_map(|(_, kinds)| kinds.iter().copied())
            .collect();
        assert_eq!(all.len(), ComponentKind::ALL.len());
        for &kind in ComponentKind::ALL {
            assert!(all.contains(&kind), "{:?} missing from categories", kind);
        }
    }

    #[test]
    fn test_every_kind_has_consistent_category() {
        for &kind in ComponentKind::ALL {
            let category = kind.category();
            let categories = categorized_components();
            let (_, kinds) = categories
                .iter()
                .find(|(c, _)| *c == category)
                .expect("category present");
            assert!(kinds.contains(&kind));
        }
    }
}
