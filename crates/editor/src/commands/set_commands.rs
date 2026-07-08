//! Commands for property edits: gizmo drags and inspector field writes.

use std::any::Any;

use ecs::audio_components::AudioSource;
use ecs::behavior::Behavior;
use ecs::sprite_components::Sprite;
use ecs::{EntityId, World};
use physics::components::{Collider, RigidBody};

use super::EditorCommand;

// ---------------------------------------------------------------------------
// TransformGizmoCommand
// ---------------------------------------------------------------------------

/// Command for a transform gizmo drag operation.
///
/// Supports merging: consecutive gizmo drags on the same entity collapse
/// into a single undo entry.
pub struct TransformGizmoCommand {
    entity: EntityId,
    initial: common::Transform2D,
    final_val: common::Transform2D,
}

impl TransformGizmoCommand {
    pub fn new(entity: EntityId, initial: common::Transform2D, final_val: common::Transform2D) -> Self {
        Self { entity, initial, final_val }
    }
}

impl EditorCommand for TransformGizmoCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(t) = world.get_mut::<common::Transform2D>(self.entity) {
            *t = self.final_val;
        }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(t) = world.get_mut::<common::Transform2D>(self.entity) {
            *t = self.initial;
        }
    }

    fn display_name(&self) -> &str {
        "Transform Gizmo"
    }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<TransformGizmoCommand>() {
            if self.entity == other.entity {
                self.final_val = other.final_val;
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// Set*Commands (inspector property edits)
// ---------------------------------------------------------------------------

/// Generates a `Set*Command` for an inspector property edit on one component
/// type. All five commands share the same shape: store old/new values plus a
/// `field_hint`, write the value on execute/undo, and merge consecutive edits
/// to the same field on the same entity into one undo entry.
macro_rules! impl_set_component_command {
    ($(#[$attr:meta])* $name:ident, $ty:ty, $display:expr) => {
        $(#[$attr])*
        pub struct $name {
            entity: EntityId,
            old: $ty,
            new: $ty,
            field_hint: &'static str,
        }

        impl $name {
            pub fn new(entity: EntityId, old: $ty, new: $ty, field_hint: &'static str) -> Self {
                Self { entity, old, new, field_hint }
            }
        }

        impl EditorCommand for $name {
            fn execute(&mut self, world: &mut World) {
                if let Some(c) = world.get_mut::<$ty>(self.entity) {
                    *c = Clone::clone(&self.new);
                }
            }

            fn undo(&mut self, world: &mut World) {
                if let Some(c) = world.get_mut::<$ty>(self.entity) {
                    *c = Clone::clone(&self.old);
                }
            }

            fn display_name(&self) -> &str { $display }

            fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
                if let Some(other) = other.as_any().downcast_ref::<$name>() {
                    if self.entity == other.entity && self.field_hint == other.field_hint {
                        self.new = Clone::clone(&other.new);
                        return true;
                    }
                }
                false
            }

            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
        }
    };
}

impl_set_component_command!(
    /// Command for an inspector property edit on a Transform2D.
    SetTransformCommand, common::Transform2D, "Set Transform");
impl_set_component_command!(
    /// Command for an inspector property edit on a Sprite.
    SetSpriteCommand, Sprite, "Set Sprite");
impl_set_component_command!(
    /// Command for an inspector property edit on a RigidBody.
    SetRigidBodyCommand, RigidBody, "Set RigidBody");
impl_set_component_command!(
    /// Command for an inspector property edit on a Collider.
    SetColliderCommand, Collider, "Set Collider");
impl_set_component_command!(
    /// Command for an inspector property edit on an AudioSource.
    SetAudioSourceCommand, AudioSource, "Set AudioSource");
impl_set_component_command!(
    /// Command for an inspector property edit on a Behavior.
    SetBehaviorCommand, Behavior, "Set Behavior");

