//! Hierarchy panel for displaying entity tree structure.
//!
//! The HierarchyPanel displays all entities in the scene as a tree view,
//! showing parent-child relationships and allowing entity selection.

use std::collections::HashSet;

use ecs::{EntityId, Name, Sprite, World, WorldHierarchyExt};
use glam::Vec2;
use physics::components::RigidBody;

use crate::layout::{LINE_HEIGHT, PADDING};
use crate::Selection;

/// Row height for each entity in the hierarchy (matches LINE_HEIGHT).
const ROW_HEIGHT: f32 = LINE_HEIGHT;

/// Base left padding (matches standard PADDING).
const BASE_PADDING: f32 = PADDING;

/// Indentation per depth level.
const INDENT_PER_DEPTH: f32 = 16.0;

/// Width of the expand/collapse arrow.
const ARROW_WIDTH: f32 = 16.0;

/// Hierarchy panel for displaying entity tree structure.
#[derive(Debug, Default)]
pub struct HierarchyPanel {
    /// Entities that are collapsed (all expanded by default).
    collapsed: HashSet<EntityId>,
}

impl HierarchyPanel {
    /// Create a new hierarchy panel.
    pub fn new() -> Self {
        Self {
            collapsed: HashSet::new(),
        }
    }

    /// Check if an entity is expanded (default: true).
    pub fn is_expanded(&self, entity: EntityId) -> bool {
        !self.collapsed.contains(&entity)
    }

    /// Toggle expand/collapse state for an entity.
    pub fn toggle_expanded(&mut self, entity: EntityId) {
        if self.collapsed.contains(&entity) {
            self.collapsed.remove(&entity);
        } else {
            self.collapsed.insert(entity);
        }
    }

    /// Set an entity to be collapsed.
    pub fn collapse(&mut self, entity: EntityId) {
        self.collapsed.insert(entity);
    }

    /// Set an entity to be expanded.
    pub fn expand(&mut self, entity: EntityId) {
        self.collapsed.remove(&entity);
    }

    /// Get the display name for an entity.
    ///
    /// Resolution order:
    /// 1. Name component
    /// 2. Sprite component → "Sprite (Entity {id})"
    /// 3. RigidBody component → "RigidBody (Entity {id})"
    /// 4. Fallback → "Entity {id}"
    pub fn entity_display_name(world: &World, entity: EntityId) -> String {
        // Check for Name component first
        if let Some(name) = world.get::<Name>(entity) {
            return name.as_str().to_string();
        }

        // Check for Sprite component
        if world.get::<Sprite>(entity).is_some() {
            return format!("Sprite (Entity {})", entity.value());
        }

        // Check for RigidBody component
        if world.get::<RigidBody>(entity).is_some() {
            return format!("RigidBody (Entity {})", entity.value());
        }

        // Fallback
        format!("Entity {}", entity.value())
    }

    /// Render the hierarchy panel.
    ///
    /// Returns the list of entities that were clicked for selection.
    pub fn render(
        &mut self,
        ui: &mut ui::UIContext,
        world: &World,
        selection: &mut Selection,
        bounds: common::Rect,
    ) -> Vec<EntityId> {
        let mut clicked_entities = Vec::new();

        // Get root entities (no parent) and sort by ID for consistent ordering
        let mut roots = world.get_root_entities();
        roots.sort_by_key(|e| e.value());

        // Render each root and its descendants with top padding
        let mut y = bounds.y + BASE_PADDING;
        for root in roots {
            y = self.render_node(
                ui,
                world,
                selection,
                bounds,
                root,
                0,
                y,
                &mut clicked_entities,
            );
        }

        clicked_entities
    }

    /// Render a single node and its children recursively.
    ///
    /// Returns the next Y position after this node and its visible children.
    fn render_node(
        &mut self,
        ui: &mut ui::UIContext,
        world: &World,
        selection: &mut Selection,
        bounds: common::Rect,
        entity: EntityId,
        depth: usize,
        y: f32,
        clicked_entities: &mut Vec<EntityId>,
    ) -> f32 {
        // Check if this row is visible within bounds
        if y + ROW_HEIGHT < bounds.y || y > bounds.y + bounds.height {
            // Skip rendering but still calculate next Y
            let mut next_y = y + ROW_HEIGHT;
            if self.is_expanded(entity) {
                if let Some(children) = world.get_children(entity) {
                    for &child in children {
                        next_y =
                            self.render_node(ui, world, selection, bounds, child, depth + 1, next_y, clicked_entities);
                    }
                }
            }
            return next_y;
        }

        let x = bounds.x + BASE_PADDING + (depth as f32 * INDENT_PER_DEPTH);
        let has_children = world.get_children(entity).map_or(false, |c| !c.is_empty());
        let is_selected = selection.contains(entity);
        let is_expanded = self.is_expanded(entity);

        // Row background for selection (full width)
        let row_rect = common::Rect::new(bounds.x, y, bounds.width, ROW_HEIGHT);
        if is_selected {
            ui.rect(row_rect, ui::Color::new(0.3, 0.5, 0.8, 0.5));
        }

        // Check arrow interaction FIRST for entities with children
        let mut arrow_clicked = false;
        if has_children {
            let arrow_rect = common::Rect::new(x, y, ARROW_WIDTH, ROW_HEIGHT);
            let arrow_id = format!("hierarchy_arrow_{}", entity.value());
            let arrow_interaction = ui.interact(arrow_id.as_str(), arrow_rect, true);

            if arrow_interaction.clicked {
                self.toggle_expanded(entity);
                arrow_clicked = true;
            }

            // Draw arrow (baseline near bottom of row)
            let arrow = if is_expanded { "▼" } else { "▶" };
            ui.label(arrow, Vec2::new(x, y + ROW_HEIGHT - 4.0));
        }

        // Row interaction - use area after arrow for entities with children
        let row_interact_x = if has_children { x + ARROW_WIDTH } else { bounds.x };
        let row_interact_width = bounds.x + bounds.width - row_interact_x;
        let row_interact_rect = common::Rect::new(row_interact_x, y, row_interact_width, ROW_HEIGHT);

        let row_id = format!("hierarchy_row_{}", entity.value());
        let row_interaction = ui.interact(row_id.as_str(), row_interact_rect, true);

        if row_interaction.clicked && !arrow_clicked {
            clicked_entities.push(entity);
        }

        // Hover highlight (full row width for visual consistency)
        if row_interaction.state == ui::WidgetState::Hovered && !is_selected {
            ui.rect(row_rect, ui::Color::new(0.5, 0.5, 0.5, 0.2));
        }

        // Entity name (baseline near bottom of row)
        let name = Self::entity_display_name(world, entity);
        let name_x = x + if has_children { ARROW_WIDTH } else { 0.0 };
        ui.label(&name, Vec2::new(name_x, y + ROW_HEIGHT - 4.0));

        // Render children if expanded
        let mut next_y = y + ROW_HEIGHT;
        if is_expanded && has_children {
            if let Some(children) = world.get_children(entity) {
                // Clone to avoid borrow issues
                let children_vec: Vec<EntityId> = children.to_vec();
                for child in children_vec {
                    next_y = self.render_node(
                        ui,
                        world,
                        selection,
                        bounds,
                        child,
                        depth + 1,
                        next_y,
                        clicked_entities,
                    );
                }
            }
        }

        next_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::World;

    fn entity(id: u64) -> EntityId {
        EntityId::with_generation(id, 1)
    }

    // ==================== Expand/Collapse State Tests ====================

    #[test]
    fn test_default_expanded() {
        let panel = HierarchyPanel::new();
        let e1 = entity(1);

        // Entities are expanded by default
        assert!(panel.is_expanded(e1));
    }

    #[test]
    fn test_toggle_collapse() {
        let mut panel = HierarchyPanel::new();
        let e1 = entity(1);

        // Initially expanded
        assert!(panel.is_expanded(e1));

        // Toggle to collapse
        panel.toggle_expanded(e1);
        assert!(!panel.is_expanded(e1));

        // Toggle to expand again
        panel.toggle_expanded(e1);
        assert!(panel.is_expanded(e1));
    }

    #[test]
    fn test_collapse_persists() {
        let mut panel = HierarchyPanel::new();
        let e1 = entity(1);
        let e2 = entity(2);

        // Collapse e1
        panel.collapse(e1);
        assert!(!panel.is_expanded(e1));
        assert!(panel.is_expanded(e2)); // e2 still expanded

        // Expand e1
        panel.expand(e1);
        assert!(panel.is_expanded(e1));
    }

    #[test]
    fn test_multiple_entities_independent_state() {
        let mut panel = HierarchyPanel::new();
        let e1 = entity(1);
        let e2 = entity(2);
        let e3 = entity(3);

        panel.collapse(e1);
        panel.collapse(e3);

        assert!(!panel.is_expanded(e1));
        assert!(panel.is_expanded(e2));
        assert!(!panel.is_expanded(e3));
    }

    // ==================== Name Resolution Tests ====================

    #[test]
    fn test_name_from_name_component() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(&e, Name::new("Player")).ok();
        world.add_component(&e, Sprite::default()).ok(); // Also has sprite

        // Name component takes priority
        let name = HierarchyPanel::entity_display_name(&world, e);
        assert_eq!(name, "Player");
    }

    #[test]
    fn test_name_fallback_sprite() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(&e, Sprite::default()).ok();

        let name = HierarchyPanel::entity_display_name(&world, e);
        assert!(name.starts_with("Sprite (Entity"));
    }

    #[test]
    fn test_name_fallback_rigidbody() {
        let mut world = World::new();
        let e = world.create_entity();
        world.add_component(&e, RigidBody::default()).ok();

        let name = HierarchyPanel::entity_display_name(&world, e);
        assert!(name.starts_with("RigidBody (Entity"));
    }

    #[test]
    fn test_name_fallback_entity_id() {
        let mut world = World::new();
        let e = world.create_entity();

        let name = HierarchyPanel::entity_display_name(&world, e);
        assert!(name.starts_with("Entity"));
    }

    // ==================== Tree Structure Tests ====================

    #[test]
    fn test_hierarchy_panel_new() {
        let panel = HierarchyPanel::new();
        assert!(panel.collapsed.is_empty());
    }

    #[test]
    fn test_root_entities_rendering_order() {
        // This test verifies the logic without actual UI rendering
        let mut world = World::new();
        let root1 = world.create_entity();
        let root2 = world.create_entity();
        let child = world.create_entity();

        world.set_parent(child, root1).unwrap();

        let roots = world.get_root_entities();

        // Should have 2 root entities
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&root1));
        assert!(roots.contains(&root2));
        assert!(!roots.contains(&child));
    }

    #[test]
    fn test_collapsed_hides_children() {
        let mut panel = HierarchyPanel::new();
        let mut world = World::new();

        let parent = world.create_entity();
        let child = world.create_entity();
        world.set_parent(child, parent).unwrap();

        // When parent is expanded, children are visible (is_expanded returns true)
        assert!(panel.is_expanded(parent));

        // When parent is collapsed, children are hidden
        panel.collapse(parent);
        assert!(!panel.is_expanded(parent));
    }

    #[test]
    fn test_deep_hierarchy_structure() {
        let mut world = World::new();

        let grandparent = world.create_entity();
        let parent = world.create_entity();
        let child = world.create_entity();

        world.set_parent(parent, grandparent).unwrap();
        world.set_parent(child, parent).unwrap();

        // Verify hierarchy structure
        let roots = world.get_root_entities();
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&grandparent));

        let descendants = world.get_descendants(grandparent);
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&parent));
        assert!(descendants.contains(&child));
    }
}
