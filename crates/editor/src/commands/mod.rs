//! Undo/redo command system for the editor.
//!
//! Implements the Command pattern: each user action is represented as an
//! `EditorCommand` that can be executed, undone, and redone. The `CommandHistory`
//! manages undo/redo stacks with optional command merging for continuous edits
//! (e.g., dragging a gizmo or scrubbing a slider).

use std::any::Any;

use ecs::World;

mod component_commands;
mod entity_commands;
mod set_commands;

pub use component_commands::{AddComponentCommand, RemoveComponentCommand};
pub use entity_commands::{CreateEntityCommand, DeleteEntityCommand, MacroCommand};
pub use set_commands::{
    SetAudioSourceCommand, SetBehaviorCommand, SetColliderCommand, SetRigidBodyCommand,
    SetSpriteCommand, SetTransformCommand, TransformGizmoCommand,
};

// The registry-generated ComponentKind is re-exported here so existing
// `editor::commands::ComponentKind` paths keep working.
pub use crate::stored_component::ComponentKind;

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


#[cfg(test)]
mod tests;
