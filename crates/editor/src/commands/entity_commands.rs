//! Commands for entity lifecycle: create, delete, and grouped (macro) actions.

use std::any::Any;

use ecs::{EntityId, World, WorldHierarchyExt};

use crate::stored_component::{capture_all_components, restore_components, StoredComponent};

use super::EditorCommand;

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

