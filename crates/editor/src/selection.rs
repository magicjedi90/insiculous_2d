//! Entity selection management for the editor.
//!
//! The Selection struct tracks which entities are currently selected in the
//! editor and provides methods for manipulating the selection.

use std::collections::HashSet;
use ecs::EntityId;

/// Manages the current entity selection in the editor.
///
/// Supports single and multi-selection of entities, with methods for
/// common selection operations like toggle, add, remove, and clear.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    /// Currently selected entities
    selected: HashSet<EntityId>,
    /// Primary selection (the "focus" entity for property editing)
    primary: Option<EntityId>,
}

impl Selection {
    /// Create a new empty selection.
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            primary: None,
        }
    }

    /// Check if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Get the number of selected entities.
    pub fn len(&self) -> usize {
        self.selected.len()
    }

    /// Check if an entity is selected.
    pub fn contains(&self, entity: EntityId) -> bool {
        self.selected.contains(&entity)
    }

    /// Get the primary selected entity (for property editing).
    pub fn primary(&self) -> Option<EntityId> {
        self.primary
    }

    /// Get all selected entities.
    pub fn selected(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.selected.iter().copied()
    }

    /// Select a single entity, clearing any previous selection.
    pub fn select(&mut self, entity: EntityId) {
        self.selected.clear();
        self.selected.insert(entity);
        self.primary = Some(entity);
    }

    /// Add an entity to the current selection (multi-select).
    pub fn add(&mut self, entity: EntityId) {
        self.selected.insert(entity);
        if self.primary.is_none() {
            self.primary = Some(entity);
        }
    }

    /// Remove an entity from the selection.
    pub fn remove(&mut self, entity: EntityId) {
        self.selected.remove(&entity);
        if self.primary == Some(entity) {
            self.primary = self.selected.iter().next().copied();
        }
    }

    /// Toggle an entity's selection state.
    pub fn toggle(&mut self, entity: EntityId) {
        if self.selected.contains(&entity) {
            self.remove(entity);
        } else {
            self.add(entity);
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.primary = None;
    }

    /// Select multiple entities, clearing any previous selection.
    pub fn select_multiple(&mut self, entities: impl IntoIterator<Item = EntityId>) {
        self.selected.clear();
        self.selected.extend(entities);
        self.primary = self.selected.iter().next().copied();
    }

    /// Set the primary selection (must be in the current selection).
    pub fn set_primary(&mut self, entity: EntityId) {
        if self.selected.contains(&entity) {
            self.primary = Some(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u64) -> EntityId {
        EntityId::with_generation(id, 1)
    }

    #[test]
    fn test_selection_new() {
        let selection = Selection::new();
        assert!(selection.is_empty());
        assert_eq!(selection.len(), 0);
        assert!(selection.primary().is_none());
    }

    #[test]
    fn test_selection_select() {
        let mut selection = Selection::new();
        let e1 = entity(1);

        selection.select(e1);

        assert!(!selection.is_empty());
        assert_eq!(selection.len(), 1);
        assert!(selection.contains(e1));
        assert_eq!(selection.primary(), Some(e1));
    }

    #[test]
    fn test_selection_select_clears_previous() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.select(e1);
        selection.select(e2);

        assert_eq!(selection.len(), 1);
        assert!(!selection.contains(e1));
        assert!(selection.contains(e2));
        assert_eq!(selection.primary(), Some(e2));
    }

    #[test]
    fn test_selection_add() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);

        assert_eq!(selection.len(), 2);
        assert!(selection.contains(e1));
        assert!(selection.contains(e2));
        // Primary should be the first added
        assert_eq!(selection.primary(), Some(e1));
    }

    #[test]
    fn test_selection_remove() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);
        selection.remove(e1);

        assert_eq!(selection.len(), 1);
        assert!(!selection.contains(e1));
        assert!(selection.contains(e2));
    }

    #[test]
    fn test_selection_remove_primary_updates() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);
        assert_eq!(selection.primary(), Some(e1));

        selection.remove(e1);
        // Primary should update to remaining entity
        assert_eq!(selection.primary(), Some(e2));
    }

    #[test]
    fn test_selection_toggle() {
        let mut selection = Selection::new();
        let e1 = entity(1);

        selection.toggle(e1);
        assert!(selection.contains(e1));

        selection.toggle(e1);
        assert!(!selection.contains(e1));
    }

    #[test]
    fn test_selection_clear() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);
        selection.clear();

        assert!(selection.is_empty());
        assert!(selection.primary().is_none());
    }

    #[test]
    fn test_selection_select_multiple() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);
        let e3 = entity(3);

        selection.select(e3); // Previous selection
        selection.select_multiple([e1, e2]);

        assert_eq!(selection.len(), 2);
        assert!(selection.contains(e1));
        assert!(selection.contains(e2));
        assert!(!selection.contains(e3));
    }

    #[test]
    fn test_selection_set_primary() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);
        selection.set_primary(e2);

        assert_eq!(selection.primary(), Some(e2));
    }

    #[test]
    fn test_selection_set_primary_must_be_selected() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.select(e1);
        selection.set_primary(e2); // e2 is not selected

        // Primary should remain e1
        assert_eq!(selection.primary(), Some(e1));
    }

    #[test]
    fn test_selection_iterator() {
        let mut selection = Selection::new();
        let e1 = entity(1);
        let e2 = entity(2);

        selection.add(e1);
        selection.add(e2);

        let selected: Vec<_> = selection.selected().collect();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&e1));
        assert!(selected.contains(&e2));
    }
}
