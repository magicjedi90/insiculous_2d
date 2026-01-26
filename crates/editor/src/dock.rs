//! Dockable panel system for the editor.
//!
//! Provides a flexible layout system with dockable panels that can be
//! positioned at different edges of the window or floated.

use ui::{Color, Rect, TextAlign, UIContext, WidgetId};

use crate::layout::{DEFAULT_PANEL_WIDTH, HEADER_HEIGHT, MIN_PANEL_SIZE, RESIZE_HANDLE_SIZE};

/// Unique identifier for a dock panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelId(pub u32);

impl PanelId {
    /// Scene view panel (main viewport)
    pub const SCENE_VIEW: PanelId = PanelId(0);
    /// Entity inspector panel
    pub const INSPECTOR: PanelId = PanelId(1);
    /// Scene hierarchy panel
    pub const HIERARCHY: PanelId = PanelId(2);
    /// Asset browser panel
    pub const ASSET_BROWSER: PanelId = PanelId(3);
    /// Console/output panel
    pub const CONSOLE: PanelId = PanelId(4);
}

impl From<PanelId> for WidgetId {
    fn from(id: PanelId) -> Self {
        WidgetId::new(id.0 as u64 + 10000) // Offset to avoid collision with other widgets
    }
}

/// Position where a panel can be docked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockPosition {
    /// Panel is docked to the left edge
    Left,
    /// Panel is docked to the right edge
    Right,
    /// Panel is docked to the top edge
    Top,
    /// Panel is docked to the bottom edge
    Bottom,
    /// Panel fills the center (main content area)
    #[default]
    Center,
    /// Panel is floating (not docked)
    Floating,
}

/// A dockable panel in the editor.
#[derive(Debug, Clone)]
pub struct DockPanel {
    /// Panel identifier
    pub id: PanelId,
    /// Panel title displayed in the header
    pub title: String,
    /// Where the panel is docked
    pub position: DockPosition,
    /// Panel bounds (updated during layout)
    pub bounds: Rect,
    /// Panel size (width for Left/Right, height for Top/Bottom)
    pub size: f32,
    /// Minimum size
    pub min_size: f32,
    /// Whether the panel is visible
    pub visible: bool,
    /// Whether the panel can be resized
    pub resizable: bool,
}

impl DockPanel {
    /// Create a new dock panel.
    pub fn new(id: PanelId, title: impl Into<String>, position: DockPosition) -> Self {
        Self {
            id,
            title: title.into(),
            position,
            bounds: Rect::default(),
            size: DEFAULT_PANEL_WIDTH,
            min_size: MIN_PANEL_SIZE,
            visible: true,
            resizable: true,
        }
    }

    /// Set the panel size.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the minimum size.
    pub fn with_min_size(mut self, min_size: f32) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set whether the panel is resizable.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Get the content bounds (excluding header).
    pub fn content_bounds(&self) -> Rect {
        Rect::new(
            self.bounds.x,
            self.bounds.y + HEADER_HEIGHT,
            self.bounds.width,
            (self.bounds.height - HEADER_HEIGHT).max(0.0),
        )
    }
}

/// Manages the layout and rendering of docked panels.
#[derive(Debug, Clone)]
pub struct DockArea {
    /// All panels in the dock area
    panels: Vec<DockPanel>,
    /// Available area for docking
    bounds: Rect,
    /// Header height for panels
    header_height: f32,
    /// Resize handle size
    resize_handle_size: f32,
}

impl Default for DockArea {
    fn default() -> Self {
        Self::new()
    }
}

impl DockArea {
    /// Create a new dock area.
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            bounds: Rect::default(),
            header_height: HEADER_HEIGHT,
            resize_handle_size: RESIZE_HANDLE_SIZE,
        }
    }

    /// Add a panel to the dock area.
    pub fn add_panel(&mut self, panel: DockPanel) {
        self.panels.push(panel);
    }

    /// Get a panel by ID.
    pub fn get_panel(&self, id: PanelId) -> Option<&DockPanel> {
        self.panels.iter().find(|p| p.id == id)
    }

    /// Get a panel by ID (mutable).
    pub fn get_panel_mut(&mut self, id: PanelId) -> Option<&mut DockPanel> {
        self.panels.iter_mut().find(|p| p.id == id)
    }

    /// Get all panels.
    pub fn panels(&self) -> &[DockPanel] {
        &self.panels
    }

    /// Set the available bounds for the dock area.
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    /// Update panel layouts based on current dock positions.
    pub fn layout(&mut self) {
        let mut remaining = self.bounds;

        // First pass: allocate space for edge-docked panels
        for panel in &mut self.panels {
            if !panel.visible {
                continue;
            }

            match panel.position {
                DockPosition::Left => {
                    let width = panel.size.min(remaining.width);
                    panel.bounds = Rect::new(remaining.x, remaining.y, width, remaining.height);
                    remaining.x += width;
                    remaining.width -= width;
                }
                DockPosition::Right => {
                    let width = panel.size.min(remaining.width);
                    panel.bounds = Rect::new(
                        remaining.x + remaining.width - width,
                        remaining.y,
                        width,
                        remaining.height,
                    );
                    remaining.width -= width;
                }
                DockPosition::Top => {
                    let height = panel.size.min(remaining.height);
                    panel.bounds = Rect::new(remaining.x, remaining.y, remaining.width, height);
                    remaining.y += height;
                    remaining.height -= height;
                }
                DockPosition::Bottom => {
                    let height = panel.size.min(remaining.height);
                    panel.bounds = Rect::new(
                        remaining.x,
                        remaining.y + remaining.height - height,
                        remaining.width,
                        height,
                    );
                    remaining.height -= height;
                }
                DockPosition::Center | DockPosition::Floating => {
                    // Handled in second pass
                }
            }
        }

        // Second pass: center panels get remaining space
        for panel in &mut self.panels {
            if !panel.visible {
                continue;
            }

            if panel.position == DockPosition::Center {
                panel.bounds = remaining;
            }
        }
    }

    /// Render all panels.
    ///
    /// Returns the content bounds for each visible panel. The caller should:
    /// 1. Render content within each bounds
    /// 2. Call `end_panel_content(ui)` after rendering each panel's content
    pub fn render(&mut self, ui: &mut UIContext) -> Vec<(PanelId, Rect)> {
        let mut content_areas = Vec::new();

        for panel in &self.panels {
            if !panel.visible {
                continue;
            }

            // Draw panel background
            ui.panel(panel.bounds);

            // Draw panel header
            let header_bounds = Rect::new(
                panel.bounds.x,
                panel.bounds.y,
                panel.bounds.width,
                self.header_height,
            );
            ui.rect_rounded(header_bounds, Color::new(0.15, 0.15, 0.15, 1.0), 0.0);

            // Draw panel title - properly centered
            ui.label_in_bounds(&panel.title, header_bounds, TextAlign::Left);

            // Get content bounds and push clip rect
            let content = panel.content_bounds();
            ui.push_clip_rect(content);

            // Track content area (caller will render content, then pop clip)
            content_areas.push((panel.id, content));
        }

        content_areas
    }

    /// Call after rendering content for each panel to pop the clip rect.
    pub fn end_panel_content(&self, ui: &mut UIContext, panel_count: usize) {
        for _ in 0..panel_count {
            ui.pop_clip_rect();
        }
    }

    /// Handle resize dragging for panels.
    pub fn handle_resize(&mut self, ui: &mut UIContext) {
        for i in 0..self.panels.len() {
            if !self.panels[i].visible || !self.panels[i].resizable {
                continue;
            }

            let panel = &self.panels[i];
            let resize_bounds = self.resize_handle_bounds(panel);

            // Create unique ID for resize handle
            let id = format!("resize_handle_{}", panel.id.0);
            let result = ui.interact(id.as_str(), resize_bounds, true);

            if result.dragging {
                let mouse_pos = ui.mouse_pos();
                let panel = &mut self.panels[i];

                match panel.position {
                    DockPosition::Left => {
                        panel.size = (mouse_pos.x - panel.bounds.x).max(panel.min_size);
                    }
                    DockPosition::Right => {
                        let right_edge = panel.bounds.x + panel.bounds.width;
                        panel.size = (right_edge - mouse_pos.x).max(panel.min_size);
                    }
                    DockPosition::Top => {
                        panel.size = (mouse_pos.y - panel.bounds.y).max(panel.min_size);
                    }
                    DockPosition::Bottom => {
                        let bottom_edge = panel.bounds.y + panel.bounds.height;
                        panel.size = (bottom_edge - mouse_pos.y).max(panel.min_size);
                    }
                    _ => {}
                }

                // Re-layout after resize
                self.layout();
            }
        }
    }

    /// Get the resize handle bounds for a panel.
    fn resize_handle_bounds(&self, panel: &DockPanel) -> Rect {
        match panel.position {
            DockPosition::Left => Rect::new(
                panel.bounds.x + panel.bounds.width - self.resize_handle_size,
                panel.bounds.y,
                self.resize_handle_size * 2.0,
                panel.bounds.height,
            ),
            DockPosition::Right => Rect::new(
                panel.bounds.x - self.resize_handle_size,
                panel.bounds.y,
                self.resize_handle_size * 2.0,
                panel.bounds.height,
            ),
            DockPosition::Top => Rect::new(
                panel.bounds.x,
                panel.bounds.y + panel.bounds.height - self.resize_handle_size,
                panel.bounds.width,
                self.resize_handle_size * 2.0,
            ),
            DockPosition::Bottom => Rect::new(
                panel.bounds.x,
                panel.bounds.y - self.resize_handle_size,
                panel.bounds.width,
                self.resize_handle_size * 2.0,
            ),
            _ => Rect::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_id_constants() {
        assert_eq!(PanelId::SCENE_VIEW.0, 0);
        assert_eq!(PanelId::INSPECTOR.0, 1);
        assert_eq!(PanelId::HIERARCHY.0, 2);
        assert_eq!(PanelId::ASSET_BROWSER.0, 3);
        assert_eq!(PanelId::CONSOLE.0, 4);
    }

    #[test]
    fn test_dock_position_default() {
        assert_eq!(DockPosition::default(), DockPosition::Center);
    }

    #[test]
    fn test_dock_panel_new() {
        let panel = DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right);
        assert_eq!(panel.id, PanelId::INSPECTOR);
        assert_eq!(panel.title, "Inspector");
        assert_eq!(panel.position, DockPosition::Right);
        assert!(panel.visible);
        assert!(panel.resizable);
    }

    #[test]
    fn test_dock_panel_builder() {
        let panel = DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left)
            .with_size(300.0)
            .with_min_size(150.0)
            .with_resizable(false);

        assert_eq!(panel.size, 300.0);
        assert_eq!(panel.min_size, 150.0);
        assert!(!panel.resizable);
    }

    #[test]
    fn test_dock_panel_content_bounds() {
        let mut panel = DockPanel::new(PanelId::INSPECTOR, "Test", DockPosition::Right);
        panel.bounds = Rect::new(100.0, 50.0, 200.0, 400.0);

        let content = panel.content_bounds();
        assert_eq!(content.x, 100.0);
        assert_eq!(content.y, 74.0); // 50 + 24 header
        assert_eq!(content.width, 200.0);
        assert_eq!(content.height, 376.0); // 400 - 24 header
    }

    #[test]
    fn test_dock_area_new() {
        let area = DockArea::new();
        assert!(area.panels().is_empty());
    }

    #[test]
    fn test_dock_area_add_panel() {
        let mut area = DockArea::new();
        area.add_panel(DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right));
        area.add_panel(DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left));

        assert_eq!(area.panels().len(), 2);
    }

    #[test]
    fn test_dock_area_get_panel() {
        let mut area = DockArea::new();
        area.add_panel(DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right));

        let panel = area.get_panel(PanelId::INSPECTOR);
        assert!(panel.is_some());
        assert_eq!(panel.unwrap().title, "Inspector");

        let missing = area.get_panel(PanelId::HIERARCHY);
        assert!(missing.is_none());
    }

    #[test]
    fn test_dock_area_layout_left() {
        let mut area = DockArea::new();
        area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
        area.add_panel(
            DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left)
                .with_size(200.0),
        );
        area.layout();

        let panel = area.get_panel(PanelId::HIERARCHY).unwrap();
        assert_eq!(panel.bounds.x, 0.0);
        assert_eq!(panel.bounds.y, 0.0);
        assert_eq!(panel.bounds.width, 200.0);
        assert_eq!(panel.bounds.height, 800.0);
    }

    #[test]
    fn test_dock_area_layout_right() {
        let mut area = DockArea::new();
        area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
        area.add_panel(
            DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right)
                .with_size(250.0),
        );
        area.layout();

        let panel = area.get_panel(PanelId::INSPECTOR).unwrap();
        assert_eq!(panel.bounds.x, 750.0); // 1000 - 250
        assert_eq!(panel.bounds.y, 0.0);
        assert_eq!(panel.bounds.width, 250.0);
        assert_eq!(panel.bounds.height, 800.0);
    }

    #[test]
    fn test_dock_area_layout_center_gets_remaining() {
        let mut area = DockArea::new();
        area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
        area.add_panel(
            DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left)
                .with_size(200.0),
        );
        area.add_panel(
            DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right)
                .with_size(250.0),
        );
        area.add_panel(DockPanel::new(
            PanelId::SCENE_VIEW,
            "Scene",
            DockPosition::Center,
        ));
        area.layout();

        let center = area.get_panel(PanelId::SCENE_VIEW).unwrap();
        assert_eq!(center.bounds.x, 200.0);
        assert_eq!(center.bounds.y, 0.0);
        assert_eq!(center.bounds.width, 550.0); // 1000 - 200 - 250
        assert_eq!(center.bounds.height, 800.0);
    }

    #[test]
    fn test_dock_area_layout_hidden_panel() {
        let mut area = DockArea::new();
        area.set_bounds(Rect::new(0.0, 0.0, 1000.0, 800.0));
        area.add_panel({
            let mut panel = DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left)
                .with_size(200.0);
            panel.visible = false;
            panel
        });
        area.add_panel(DockPanel::new(
            PanelId::SCENE_VIEW,
            "Scene",
            DockPosition::Center,
        ));
        area.layout();

        // Center should get full width since left panel is hidden
        let center = area.get_panel(PanelId::SCENE_VIEW).unwrap();
        assert_eq!(center.bounds.x, 0.0);
        assert_eq!(center.bounds.width, 1000.0);
    }
}
