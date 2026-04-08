//! Undo/redo command system for the editor.
//!
//! Implements the Command pattern: each user action is represented as an
//! `EditorCommand` that can be executed, undone, and redone. The `CommandHistory`
//! manages undo/redo stacks with optional command merging for continuous edits
//! (e.g., dragging a gizmo or scrubbing a slider).

use std::any::Any;

use ecs::audio_components::{AudioListener, AudioSource};
use ecs::sprite_components::{Sprite, SpriteAnimation};
use ecs::{EntityId, World, WorldHierarchyExt};
use physics::components::{Collider, RigidBody};

use crate::stored_component::{capture_all_components, restore_components, StoredComponent};

// ---------------------------------------------------------------------------
// EditorCommand trait
// ---------------------------------------------------------------------------

/// A reversible editor action.
pub trait EditorCommand: Send {
    /// Apply the action to the world.
    fn execute(&mut self, world: &mut World);

    /// Reverse the action.
    fn undo(&mut self, world: &mut World);

    /// Human-readable name shown in Edit menu (e.g., "Move Entity").
    fn display_name(&self) -> &str;

    /// Attempt to merge `other` into `self`. Returns `true` if merged.
    ///
    /// When merged, `self` is updated in-place and `other` is discarded.
    /// Default implementation returns `false` (no merging).
    fn try_merge(&mut self, _other: &dyn EditorCommand) -> bool {
        false
    }

    /// Downcast to `&dyn Any` for type-based merging.
    fn as_any(&self) -> &dyn Any;

    /// Downcast to `&mut dyn Any` for type-based merging.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// ---------------------------------------------------------------------------
// CommandHistory
// ---------------------------------------------------------------------------

/// Manages undo/redo stacks for editor commands.
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
    max_history: usize,
}

impl CommandHistory {
    /// Create a new command history with default max history (100).
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    /// Execute a command and push it onto the undo stack. Clears the redo stack.
    pub fn execute(&mut self, mut cmd: Box<dyn EditorCommand>, world: &mut World) {
        cmd.execute(world);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.enforce_limit();
    }

    /// Undo the most recent command.
    pub fn undo(&mut self, world: &mut World) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(world);
            self.redo_stack.push(cmd);
        }
    }

    /// Redo the most recently undone command.
    pub fn redo(&mut self, world: &mut World) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(world);
            self.undo_stack.push(cmd);
        }
    }

    /// Whether there is a command to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there is a command to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Display name of the command that would be undone, if any.
    pub fn undo_name(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.display_name())
    }

    /// Display name of the command that would be redone, if any.
    pub fn redo_name(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.display_name())
    }

    /// Clear both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Push a pre-executed command onto the undo stack without calling execute().
    /// Use when the action was already performed and you just need to record it for undo.
    pub fn push_already_executed(&mut self, cmd: Box<dyn EditorCommand>) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        self.enforce_limit();
    }

    /// Try to merge `cmd` with the last undo command. If merging fails, execute normally.
    ///
    /// Used for continuous edits like gizmo drags or slider scrubs to avoid
    /// flooding the undo history with one entry per frame.
    pub fn try_merge_or_execute(&mut self, cmd: Box<dyn EditorCommand>, world: &mut World) {
        if let Some(last) = self.undo_stack.last_mut() {
            if last.try_merge(cmd.as_ref()) {
                // Merged into existing command — no new push needed.
                return;
            }
        }
        self.execute(cmd, world);
    }

    /// Try to merge `cmd` with the last undo command, or push without executing if merge fails.
    ///
    /// Use when the change was already applied to the world manually (e.g., inspector
    /// writeback for immediate visual feedback). The command is recorded for undo/redo
    /// but `execute()` is not called.
    pub fn try_merge_or_push(&mut self, cmd: Box<dyn EditorCommand>) {
        if let Some(last) = self.undo_stack.last_mut() {
            if last.try_merge(cmd.as_ref()) {
                return;
            }
        }
        self.push_already_executed(cmd);
    }

    fn enforce_limit(&mut self) {
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// CreateEntityCommand
// ---------------------------------------------------------------------------

/// Command for creating a new entity.
///
/// On first execute the entity is already created by the caller — the command
/// captures its ID and all component data. On undo the entity is removed.
/// On redo a new entity is created with the stored components (the stored
/// entity ID is updated since IDs cannot be reused).
pub struct CreateEntityCommand {
    entity: EntityId,
    components: Vec<StoredComponent>,
    captured: bool,
}

impl CreateEntityCommand {
    /// Create from an entity that was already added to the world.
    pub fn already_created(world: &World, entity: EntityId) -> Self {
        Self {
            entity,
            components: capture_all_components(world, entity),
            captured: true,
        }
    }
}

impl EditorCommand for CreateEntityCommand {
    fn execute(&mut self, world: &mut World) {
        if self.captured {
            // First execute — entity already exists. Nothing to do.
            self.captured = false;
        } else {
            // Redo — create a new entity and restore components.
            let new_entity = world.create_entity();
            restore_components(world, new_entity, &self.components);
            self.entity = new_entity;
        }
    }

    fn undo(&mut self, world: &mut World) {
        // Capture latest component state before removing.
        self.components = capture_all_components(world, self.entity);
        world.remove_entity(&self.entity).ok();
    }

    fn display_name(&self) -> &str {
        "Create Entity"
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// DeleteEntityCommand
// ---------------------------------------------------------------------------

/// Command for deleting an entity.
pub struct DeleteEntityCommand {
    entity: EntityId,
    components: Vec<StoredComponent>,
    parent: Option<EntityId>,
    children: Vec<EntityId>,
}

impl DeleteEntityCommand {
    /// Create a delete command. Components are captured when executed.
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            components: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}

impl EditorCommand for DeleteEntityCommand {
    fn execute(&mut self, world: &mut World) {
        // Capture component data and hierarchy before removal.
        self.components = capture_all_components(world, self.entity);
        self.parent = world.get_parent(self.entity);
        self.children = world
            .get_children(self.entity)
            .map(|c| c.to_vec())
            .unwrap_or_default();

        // Reparent children to grandparent (or make roots).
        for &child in &self.children {
            if let Some(parent) = self.parent {
                world.set_parent(child, parent).ok();
            } else {
                world.remove_parent(child).ok();
            }
        }

        world.remove_parent(self.entity).ok();
        world.remove_entity(&self.entity).ok();
    }

    fn undo(&mut self, world: &mut World) {
        // Recreate entity and restore components.
        let new_entity = world.create_entity();
        restore_components(world, new_entity, &self.components);

        // Restore hierarchy.
        if let Some(parent) = self.parent {
            world.set_parent(new_entity, parent).ok();
        }
        for &child in &self.children {
            world.set_parent(child, new_entity).ok();
        }

        self.entity = new_entity;
    }

    fn display_name(&self) -> &str {
        "Delete Entity"
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// AddComponentCommand
// ---------------------------------------------------------------------------

/// The set of component kinds that can be added/removed (mirrors editor_integration's ComponentKind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Camera,
    Sprite,
    SpriteAnimation,
    RigidBody,
    Collider,
    AudioSource,
    AudioListener,
}

/// Command for adding a default component to an entity.
pub struct AddComponentCommand {
    entity: EntityId,
    kind: ComponentKind,
    /// Captured on undo so that redo can restore modifications made between add and undo.
    captured: Option<StoredComponent>,
}

impl AddComponentCommand {
    pub fn new(entity: EntityId, kind: ComponentKind) -> Self {
        Self {
            entity,
            kind,
            captured: None,
        }
    }
}

impl EditorCommand for AddComponentCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(ref stored) = self.captured {
            // Redo — restore the captured value.
            stored.apply_to(world, self.entity);
        } else {
            // First execute — add default.
            add_default_component(world, self.entity, self.kind);
        }
    }

    fn undo(&mut self, world: &mut World) {
        // Capture the component before removing it.
        self.captured = capture_component_by_kind(world, self.entity, self.kind);
        remove_component_by_kind(world, self.entity, self.kind);
    }

    fn display_name(&self) -> &str {
        "Add Component"
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// RemoveComponentCommand
// ---------------------------------------------------------------------------

/// Command for removing a component from an entity.
pub struct RemoveComponentCommand {
    entity: EntityId,
    /// Primary stored component.
    stored: Option<StoredComponent>,
    /// Extra component captured during RigidBody cascade (the Collider).
    cascade_stored: Option<StoredComponent>,
    kind: ComponentKind,
}

impl RemoveComponentCommand {
    pub fn new(entity: EntityId, kind: ComponentKind) -> Self {
        Self {
            entity,
            stored: None,
            cascade_stored: None,
            kind,
        }
    }
}

impl EditorCommand for RemoveComponentCommand {
    fn execute(&mut self, world: &mut World) {
        // Capture before removal.
        self.stored = capture_component_by_kind(world, self.entity, self.kind);

        // Handle RigidBody → Collider cascade.
        if self.kind == ComponentKind::RigidBody {
            self.cascade_stored = capture_component_by_kind(world, self.entity, ComponentKind::Collider);
            remove_component_by_kind(world, self.entity, ComponentKind::Collider);
        }

        remove_component_by_kind(world, self.entity, self.kind);
    }

    fn undo(&mut self, world: &mut World) {
        // Restore primary component.
        if let Some(ref stored) = self.stored {
            stored.apply_to(world, self.entity);
        }
        // Restore cascaded component.
        if let Some(ref stored) = self.cascade_stored {
            stored.apply_to(world, self.entity);
        }
    }

    fn display_name(&self) -> &str {
        "Remove Component"
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

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
// SetTransformCommand
// ---------------------------------------------------------------------------

/// Command for an inspector property edit on a Transform2D.
pub struct SetTransformCommand {
    entity: EntityId,
    old: common::Transform2D,
    new: common::Transform2D,
    field_hint: &'static str,
}

impl SetTransformCommand {
    pub fn new(
        entity: EntityId,
        old: common::Transform2D,
        new: common::Transform2D,
        field_hint: &'static str,
    ) -> Self {
        Self { entity, old, new, field_hint }
    }
}

impl EditorCommand for SetTransformCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(t) = world.get_mut::<common::Transform2D>(self.entity) {
            *t = self.new;
        }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(t) = world.get_mut::<common::Transform2D>(self.entity) {
            *t = self.old;
        }
    }

    fn display_name(&self) -> &str {
        "Set Transform"
    }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<SetTransformCommand>() {
            if self.entity == other.entity && self.field_hint == other.field_hint {
                self.new = other.new;
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// SetSpriteCommand
// ---------------------------------------------------------------------------

/// Command for an inspector property edit on a Sprite.
pub struct SetSpriteCommand {
    entity: EntityId,
    old: Sprite,
    new: Sprite,
    field_hint: &'static str,
}

impl SetSpriteCommand {
    pub fn new(entity: EntityId, old: Sprite, new: Sprite, field_hint: &'static str) -> Self {
        Self { entity, old, new, field_hint }
    }
}

impl EditorCommand for SetSpriteCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(s) = world.get_mut::<Sprite>(self.entity) { *s = self.new.clone(); }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(s) = world.get_mut::<Sprite>(self.entity) { *s = self.old.clone(); }
    }

    fn display_name(&self) -> &str { "Set Sprite" }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<SetSpriteCommand>() {
            if self.entity == other.entity && self.field_hint == other.field_hint {
                self.new = other.new.clone();
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// SetRigidBodyCommand
// ---------------------------------------------------------------------------

/// Command for an inspector property edit on a RigidBody.
pub struct SetRigidBodyCommand {
    entity: EntityId,
    old: RigidBody,
    new: RigidBody,
    field_hint: &'static str,
}

impl SetRigidBodyCommand {
    pub fn new(entity: EntityId, old: RigidBody, new: RigidBody, field_hint: &'static str) -> Self {
        Self { entity, old, new, field_hint }
    }
}

impl EditorCommand for SetRigidBodyCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<RigidBody>(self.entity) { *c = self.new.clone(); }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<RigidBody>(self.entity) { *c = self.old.clone(); }
    }

    fn display_name(&self) -> &str { "Set RigidBody" }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<SetRigidBodyCommand>() {
            if self.entity == other.entity && self.field_hint == other.field_hint {
                self.new = other.new.clone();
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// SetColliderCommand
// ---------------------------------------------------------------------------

/// Command for an inspector property edit on a Collider.
pub struct SetColliderCommand {
    entity: EntityId,
    old: Collider,
    new: Collider,
    field_hint: &'static str,
}

impl SetColliderCommand {
    pub fn new(entity: EntityId, old: Collider, new: Collider, field_hint: &'static str) -> Self {
        Self { entity, old, new, field_hint }
    }
}

impl EditorCommand for SetColliderCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<Collider>(self.entity) { *c = self.new.clone(); }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<Collider>(self.entity) { *c = self.old.clone(); }
    }

    fn display_name(&self) -> &str { "Set Collider" }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<SetColliderCommand>() {
            if self.entity == other.entity && self.field_hint == other.field_hint {
                self.new = other.new.clone();
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// SetAudioSourceCommand
// ---------------------------------------------------------------------------

/// Command for an inspector property edit on an AudioSource.
pub struct SetAudioSourceCommand {
    entity: EntityId,
    old: AudioSource,
    new: AudioSource,
    field_hint: &'static str,
}

impl SetAudioSourceCommand {
    pub fn new(entity: EntityId, old: AudioSource, new: AudioSource, field_hint: &'static str) -> Self {
        Self { entity, old, new, field_hint }
    }
}

impl EditorCommand for SetAudioSourceCommand {
    fn execute(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<AudioSource>(self.entity) { *c = self.new.clone(); }
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(c) = world.get_mut::<AudioSource>(self.entity) { *c = self.old.clone(); }
    }

    fn display_name(&self) -> &str { "Set AudioSource" }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<SetAudioSourceCommand>() {
            if self.entity == other.entity && self.field_hint == other.field_hint {
                self.new = other.new.clone();
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// MacroCommand
// ---------------------------------------------------------------------------

/// Groups multiple commands into a single undoable action.
pub struct MacroCommand {
    name: String,
    commands: Vec<Box<dyn EditorCommand>>,
}

impl MacroCommand {
    pub fn new(name: impl Into<String>, commands: Vec<Box<dyn EditorCommand>>) -> Self {
        Self {
            name: name.into(),
            commands,
        }
    }
}

impl EditorCommand for MacroCommand {
    fn execute(&mut self, world: &mut World) {
        for cmd in &mut self.commands {
            cmd.execute(world);
        }
    }

    fn undo(&mut self, world: &mut World) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(world);
        }
    }

    fn display_name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn add_default_component(world: &mut World, entity: EntityId, kind: ComponentKind) {
    match kind {
        ComponentKind::Camera => { world.add_component(&entity, common::Camera::default()).ok(); }
        ComponentKind::Sprite => { world.add_component(&entity, Sprite::default()).ok(); }
        ComponentKind::SpriteAnimation => { world.add_component(&entity, SpriteAnimation::default()).ok(); }
        ComponentKind::RigidBody => { world.add_component(&entity, RigidBody::default()).ok(); }
        ComponentKind::Collider => { world.add_component(&entity, Collider::default()).ok(); }
        ComponentKind::AudioSource => { world.add_component(&entity, AudioSource::default()).ok(); }
        ComponentKind::AudioListener => { world.add_component(&entity, AudioListener::default()).ok(); }
    }
}

fn capture_component_by_kind(
    world: &World,
    entity: EntityId,
    kind: ComponentKind,
) -> Option<StoredComponent> {
    match kind {
        ComponentKind::Camera => world.get::<common::Camera>(entity).cloned().map(StoredComponent::Camera),
        ComponentKind::Sprite => world.get::<Sprite>(entity).cloned().map(StoredComponent::Sprite),
        ComponentKind::SpriteAnimation => world.get::<SpriteAnimation>(entity).cloned().map(StoredComponent::SpriteAnimation),
        ComponentKind::RigidBody => world.get::<RigidBody>(entity).cloned().map(StoredComponent::RigidBody),
        ComponentKind::Collider => world.get::<Collider>(entity).cloned().map(StoredComponent::Collider),
        ComponentKind::AudioSource => world.get::<AudioSource>(entity).cloned().map(StoredComponent::AudioSource),
        ComponentKind::AudioListener => world.get::<AudioListener>(entity).cloned().map(StoredComponent::AudioListener),
    }
}

fn remove_component_by_kind(world: &mut World, entity: EntityId, kind: ComponentKind) {
    match kind {
        ComponentKind::Camera => { world.remove_component::<common::Camera>(&entity).ok(); }
        ComponentKind::Sprite => { world.remove_component::<Sprite>(&entity).ok(); }
        ComponentKind::SpriteAnimation => { world.remove_component::<SpriteAnimation>(&entity).ok(); }
        ComponentKind::RigidBody => { world.remove_component::<RigidBody>(&entity).ok(); }
        ComponentKind::Collider => { world.remove_component::<Collider>(&entity).ok(); }
        ComponentKind::AudioSource => { world.remove_component::<AudioSource>(&entity).ok(); }
        ComponentKind::AudioListener => { world.remove_component::<AudioListener>(&entity).ok(); }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::hierarchy::GlobalTransform2D;
    use ecs::sprite_components::Name;
    use glam::Vec2;

    fn setup_entity(world: &mut World) -> EntityId {
        let entity = world.create_entity();
        world.add_component(&entity, common::Transform2D::new(Vec2::ZERO)).ok();
        world.add_component(&entity, GlobalTransform2D::default()).ok();
        world.add_component(&entity, Name::new("Test")).ok();
        entity
    }

    // -- CommandHistory basics --

    #[test]
    fn test_command_history_execute_and_undo() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::new(10.0, 20.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );

        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(10.0, 20.0));

        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
    }

    #[test]
    fn test_command_history_redo() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::new(5.0, 5.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
        history.undo(&mut world);
        history.redo(&mut world);

        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_redo_cleared_on_new_command() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let t0 = *world.get::<common::Transform2D>(entity).unwrap();
        let t1 = common::Transform2D::new(Vec2::new(1.0, 0.0));
        let t2 = common::Transform2D::new(Vec2::new(2.0, 0.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(SetTransformCommand::new(entity, t0, t1, "position")),
            &mut world,
        );
        history.undo(&mut world);
        assert!(history.can_redo());

        // New command should clear redo.
        history.execute(
            Box::new(SetTransformCommand::new(entity, t0, t2, "position")),
            &mut world,
        );
        assert!(!history.can_redo());
    }

    #[test]
    fn test_can_undo_and_redo() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::ONE);

        let mut history = CommandHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut world);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_undo_name_and_redo_name() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::ONE);

        let mut history = CommandHistory::new();
        assert!(history.undo_name().is_none());
        assert!(history.redo_name().is_none());

        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
        assert_eq!(history.undo_name(), Some("Set Transform"));
        assert!(history.redo_name().is_none());

        history.undo(&mut world);
        assert!(history.undo_name().is_none());
        assert_eq!(history.redo_name(), Some("Set Transform"));
    }

    // -- CreateEntityCommand --

    #[test]
    fn test_create_entity_undo_removes() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        assert_eq!(world.entity_count(), 1);

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(CreateEntityCommand::already_created(&world, entity)),
            &mut world,
        );
        assert_eq!(world.entity_count(), 1);

        history.undo(&mut world);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_create_entity_redo_recreates() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let pos = Vec2::new(42.0, 99.0);
        world.get_mut::<common::Transform2D>(entity).unwrap().position = pos;

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(CreateEntityCommand::already_created(&world, entity)),
            &mut world,
        );
        history.undo(&mut world);
        assert_eq!(world.entity_count(), 0);

        history.redo(&mut world);
        assert_eq!(world.entity_count(), 1);

        // The recreated entity should have the same transform data.
        let entities: Vec<EntityId> = world.entities();
        let t = world.get::<common::Transform2D>(entities[0]).unwrap();
        assert_eq!(t.position, pos);
    }

    // -- DeleteEntityCommand --

    #[test]
    fn test_delete_entity_undo_restores() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        world.add_component(&entity, Sprite::new(7)).ok();
        assert_eq!(world.entity_count(), 1);

        let mut history = CommandHistory::new();
        history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
        assert_eq!(world.entity_count(), 0);

        history.undo(&mut world);
        assert_eq!(world.entity_count(), 1);

        let entities: Vec<EntityId> = world.entities();
        let restored = entities[0];
        assert!(world.get::<common::Transform2D>(restored).is_some());
        assert!(world.get::<Sprite>(restored).is_some());
        assert_eq!(world.get::<Sprite>(restored).unwrap().texture_handle, 7);
    }

    #[test]
    fn test_delete_entity_redo_removes_again() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);

        let mut history = CommandHistory::new();
        history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
        history.undo(&mut world);
        assert_eq!(world.entity_count(), 1);

        history.redo(&mut world);
        assert_eq!(world.entity_count(), 0);
    }

    // -- AddComponentCommand --

    #[test]
    fn test_add_component_undo_removes() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(AddComponentCommand::new(entity, ComponentKind::Sprite)),
            &mut world,
        );
        assert!(world.get::<Sprite>(entity).is_some());

        history.undo(&mut world);
        assert!(world.get::<Sprite>(entity).is_none());
    }

    // -- RemoveComponentCommand --

    #[test]
    fn test_remove_component_undo_restores() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        world.add_component(&entity, Sprite::new(3)).ok();

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(RemoveComponentCommand::new(entity, ComponentKind::Sprite)),
            &mut world,
        );
        assert!(world.get::<Sprite>(entity).is_none());

        history.undo(&mut world);
        assert!(world.get::<Sprite>(entity).is_some());
        assert_eq!(world.get::<Sprite>(entity).unwrap().texture_handle, 3);
    }

    // -- TransformGizmoCommand --

    #[test]
    fn test_transform_gizmo_undo() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let initial = *world.get::<common::Transform2D>(entity).unwrap();
        let final_val = common::Transform2D::new(Vec2::new(100.0, 200.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(TransformGizmoCommand::new(entity, initial, final_val)),
            &mut world,
        );
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::new(100.0, 200.0));

        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
    }

    #[test]
    fn test_transform_gizmo_merge() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let initial = *world.get::<common::Transform2D>(entity).unwrap();
        let mid = common::Transform2D::new(Vec2::new(50.0, 50.0));
        let final_val = common::Transform2D::new(Vec2::new(100.0, 100.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(TransformGizmoCommand::new(entity, initial, mid)),
            &mut world,
        );
        // Second gizmo drag on same entity — should merge.
        history.try_merge_or_execute(
            Box::new(TransformGizmoCommand::new(entity, mid, final_val)),
            &mut world,
        );

        // Should be a single undo entry.
        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
        assert!(!history.can_undo());
    }

    // -- SetTransformCommand --

    #[test]
    fn test_set_transform_undo() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let old = *world.get::<common::Transform2D>(entity).unwrap();
        let new = common::Transform2D::new(Vec2::new(7.0, 8.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(SetTransformCommand::new(entity, old, new, "position")),
            &mut world,
        );
        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
    }

    #[test]
    fn test_set_transform_merge() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        let t0 = *world.get::<common::Transform2D>(entity).unwrap();
        let t1 = common::Transform2D::new(Vec2::new(1.0, 0.0));
        let t2 = common::Transform2D::new(Vec2::new(2.0, 0.0));

        let mut history = CommandHistory::new();
        history.execute(
            Box::new(SetTransformCommand::new(entity, t0, t1, "position")),
            &mut world,
        );
        history.try_merge_or_execute(
            Box::new(SetTransformCommand::new(entity, t1, t2, "position")),
            &mut world,
        );

        // Single undo should go back to original.
        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(entity).unwrap().position, Vec2::ZERO);
        assert!(!history.can_undo());
    }

    // -- MacroCommand --

    #[test]
    fn test_macro_command_undo() {
        let mut world = World::new();
        let e1 = setup_entity(&mut world);
        let e2 = setup_entity(&mut world);
        let t1_old = *world.get::<common::Transform2D>(e1).unwrap();
        let t2_old = *world.get::<common::Transform2D>(e2).unwrap();
        let t1_new = common::Transform2D::new(Vec2::new(10.0, 0.0));
        let t2_new = common::Transform2D::new(Vec2::new(0.0, 10.0));

        let macro_cmd = MacroCommand::new("Move Two", vec![
            Box::new(SetTransformCommand::new(e1, t1_old, t1_new, "position")),
            Box::new(SetTransformCommand::new(e2, t2_old, t2_new, "position")),
        ]);

        let mut history = CommandHistory::new();
        history.execute(Box::new(macro_cmd), &mut world);

        assert_eq!(world.get::<common::Transform2D>(e1).unwrap().position, Vec2::new(10.0, 0.0));
        assert_eq!(world.get::<common::Transform2D>(e2).unwrap().position, Vec2::new(0.0, 10.0));

        history.undo(&mut world);
        assert_eq!(world.get::<common::Transform2D>(e1).unwrap().position, Vec2::ZERO);
        assert_eq!(world.get::<common::Transform2D>(e2).unwrap().position, Vec2::ZERO);
    }

    // -- Max history limit --

    #[test]
    fn test_max_history_limit() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);

        let mut history = CommandHistory::new();
        // Push 105 commands (limit is 100).
        for i in 0..105 {
            let old = *world.get::<common::Transform2D>(entity).unwrap();
            let new = common::Transform2D::new(Vec2::new(i as f32, 0.0));
            history.execute(
                Box::new(SetTransformCommand::new(entity, old, new, "position")),
                &mut world,
            );
        }

        // Should have at most 100 undo entries.
        let mut undo_count = 0;
        while history.can_undo() {
            history.undo(&mut world);
            undo_count += 1;
        }
        assert_eq!(undo_count, 100);
    }

    // -- StoredComponent round-trip (via commands) --

    #[test]
    fn test_stored_component_capture_and_restore() {
        let mut world = World::new();
        let entity = setup_entity(&mut world);
        world.add_component(&entity, Sprite::new(42)).ok();
        world.add_component(&entity, RigidBody::default()).ok();
        world.add_component(&entity, Collider::default()).ok();

        // Delete captures all components.
        let mut history = CommandHistory::new();
        history.execute(Box::new(DeleteEntityCommand::new(entity)), &mut world);
        assert_eq!(world.entity_count(), 0);

        // Undo restores them.
        history.undo(&mut world);
        assert_eq!(world.entity_count(), 1);

        let entities: Vec<EntityId> = world.entities();
        let restored = entities[0];
        assert!(world.get::<common::Transform2D>(restored).is_some());
        assert!(world.get::<Name>(restored).is_some());
        assert!(world.get::<Sprite>(restored).is_some());
        assert!(world.get::<RigidBody>(restored).is_some());
        assert!(world.get::<Collider>(restored).is_some());
    }
}
