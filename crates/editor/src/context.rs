//! Editor context extending the game context.
//!
//! The EditorContext wraps a GameContext and adds editor-specific state
//! like selection, gizmos, and editor camera.

use glam::Vec2;

use crate::{
    DockArea, DockPanel, DockPosition, EditorTool, Gizmo, GizmoMode, MenuBar, PanelId, Selection,
    Toolbar,
};

/// Editor-specific state that extends the game context.
#[derive(Debug)]
pub struct EditorContext {
    /// Current entity selection
    pub selection: Selection,
    /// Transform gizmo
    pub gizmo: Gizmo,
    /// Editor toolbar
    pub toolbar: Toolbar,
    /// Menu bar
    pub menu_bar: MenuBar,
    /// Dock area for panels
    pub dock_area: DockArea,
    /// Editor camera offset (for panning)
    camera_offset: Vec2,
    /// Editor camera zoom level
    camera_zoom: f32,
    /// Grid visibility
    grid_visible: bool,
    /// Grid size (in world units)
    grid_size: f32,
    /// Snap to grid enabled
    snap_to_grid: bool,
    /// Whether the editor is in play mode (running the game)
    play_mode: bool,
}

impl Default for EditorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorContext {
    /// Create a new editor context with default settings.
    pub fn new() -> Self {
        let mut dock_area = DockArea::new();

        // Add default panels
        dock_area.add_panel(
            DockPanel::new(PanelId::HIERARCHY, "Hierarchy", DockPosition::Left)
                .with_size(200.0)
                .with_min_size(150.0),
        );
        dock_area.add_panel(
            DockPanel::new(PanelId::INSPECTOR, "Inspector", DockPosition::Right)
                .with_size(280.0)
                .with_min_size(200.0),
        );
        dock_area.add_panel(DockPanel::new(
            PanelId::SCENE_VIEW,
            "Scene",
            DockPosition::Center,
        ));
        dock_area.add_panel(
            DockPanel::new(PanelId::ASSET_BROWSER, "Assets", DockPosition::Bottom)
                .with_size(180.0)
                .with_min_size(100.0),
        );

        Self {
            selection: Selection::new(),
            gizmo: Gizmo::new(),
            toolbar: Toolbar::new().with_position(Vec2::new(220.0, 34.0)), // Below menu, after hierarchy
            menu_bar: MenuBar::editor_default(),
            dock_area,
            camera_offset: Vec2::ZERO,
            camera_zoom: 1.0,
            grid_visible: true,
            grid_size: 32.0,
            snap_to_grid: false,
            play_mode: false,
        }
    }

    // ================== Tool Methods ==================

    /// Get the currently selected editor tool.
    pub fn current_tool(&self) -> EditorTool {
        self.toolbar.current_tool()
    }

    /// Set the current editor tool.
    pub fn set_tool(&mut self, tool: EditorTool) {
        self.toolbar.set_tool(tool);

        // Update gizmo mode to match tool
        let gizmo_mode = match tool {
            EditorTool::Select => GizmoMode::None,
            EditorTool::Move => GizmoMode::Translate,
            EditorTool::Rotate => GizmoMode::Rotate,
            EditorTool::Scale => GizmoMode::Scale,
        };
        self.gizmo.set_mode(gizmo_mode);
    }

    // ================== Camera Methods ==================

    /// Get the camera offset.
    pub fn camera_offset(&self) -> Vec2 {
        self.camera_offset
    }

    /// Set the camera offset.
    pub fn set_camera_offset(&mut self, offset: Vec2) {
        self.camera_offset = offset;
    }

    /// Pan the camera by a delta.
    pub fn pan_camera(&mut self, delta: Vec2) {
        self.camera_offset += delta;
    }

    /// Get the camera zoom level.
    pub fn camera_zoom(&self) -> f32 {
        self.camera_zoom
    }

    /// Set the camera zoom level.
    pub fn set_camera_zoom(&mut self, zoom: f32) {
        self.camera_zoom = zoom.clamp(0.1, 10.0);
    }

    /// Zoom the camera by a factor.
    pub fn zoom_camera(&mut self, factor: f32) {
        self.camera_zoom = (self.camera_zoom * factor).clamp(0.1, 10.0);
    }

    /// Reset camera to default view.
    pub fn reset_camera(&mut self) {
        self.camera_offset = Vec2::ZERO;
        self.camera_zoom = 1.0;
    }

    /// Convert screen position to world position.
    pub fn screen_to_world(&self, screen_pos: Vec2, window_center: Vec2) -> Vec2 {
        (screen_pos - window_center) / self.camera_zoom + self.camera_offset
    }

    /// Convert world position to screen position.
    pub fn world_to_screen(&self, world_pos: Vec2, window_center: Vec2) -> Vec2 {
        (world_pos - self.camera_offset) * self.camera_zoom + window_center
    }

    // ================== Grid Methods ==================

    /// Check if the grid is visible.
    pub fn is_grid_visible(&self) -> bool {
        self.grid_visible
    }

    /// Set grid visibility.
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.grid_visible = visible;
    }

    /// Toggle grid visibility.
    pub fn toggle_grid(&mut self) {
        self.grid_visible = !self.grid_visible;
    }

    /// Get the grid size.
    pub fn grid_size(&self) -> f32 {
        self.grid_size
    }

    /// Set the grid size.
    pub fn set_grid_size(&mut self, size: f32) {
        self.grid_size = size.max(1.0);
    }

    /// Check if snap to grid is enabled.
    pub fn is_snap_to_grid(&self) -> bool {
        self.snap_to_grid
    }

    /// Set snap to grid.
    pub fn set_snap_to_grid(&mut self, snap: bool) {
        self.snap_to_grid = snap;
    }

    /// Toggle snap to grid.
    pub fn toggle_snap_to_grid(&mut self) {
        self.snap_to_grid = !self.snap_to_grid;
    }

    /// Snap a position to the grid.
    pub fn snap_position(&self, pos: Vec2) -> Vec2 {
        if self.snap_to_grid {
            Vec2::new(
                (pos.x / self.grid_size).round() * self.grid_size,
                (pos.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            pos
        }
    }

    // ================== Play Mode Methods ==================

    /// Check if the editor is in play mode.
    pub fn is_play_mode(&self) -> bool {
        self.play_mode
    }

    /// Enter play mode.
    pub fn enter_play_mode(&mut self) {
        self.play_mode = true;
    }

    /// Exit play mode.
    pub fn exit_play_mode(&mut self) {
        self.play_mode = false;
    }

    /// Toggle play mode.
    pub fn toggle_play_mode(&mut self) {
        self.play_mode = !self.play_mode;
    }

    // ================== Layout Methods ==================

    /// Update layout based on window size.
    ///
    /// This should be called each frame before rendering to ensure
    /// panels are properly sized.
    pub fn update_layout(&mut self, window_size: Vec2) {
        // Reserve space for menu bar
        let menu_height = self.menu_bar.height();
        let content_bounds = common::Rect::new(
            0.0,
            menu_height,
            window_size.x,
            window_size.y - menu_height,
        );

        self.dock_area.set_bounds(content_bounds);
        self.dock_area.layout();

        // Note: The gizmo position should be set by the caller based on entity transform
        // after calling update_layout, using self.selection.primary() to get the selected entity
    }

    /// Get the scene view content bounds (where the game world is rendered).
    pub fn scene_view_bounds(&self) -> Option<common::Rect> {
        self.dock_area
            .get_panel(PanelId::SCENE_VIEW)
            .map(|p| p.content_bounds())
    }

    /// Get the inspector panel bounds.
    pub fn inspector_bounds(&self) -> Option<common::Rect> {
        self.dock_area
            .get_panel(PanelId::INSPECTOR)
            .map(|p| p.content_bounds())
    }

    /// Get the hierarchy panel bounds.
    pub fn hierarchy_bounds(&self) -> Option<common::Rect> {
        self.dock_area
            .get_panel(PanelId::HIERARCHY)
            .map(|p| p.content_bounds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_context_new() {
        let ctx = EditorContext::new();
        assert!(ctx.selection.is_empty());
        assert_eq!(ctx.current_tool(), EditorTool::Select);
        assert_eq!(ctx.camera_offset(), Vec2::ZERO);
        assert_eq!(ctx.camera_zoom(), 1.0);
        assert!(ctx.is_grid_visible());
        assert!(!ctx.is_snap_to_grid());
        assert!(!ctx.is_play_mode());
    }

    #[test]
    fn test_editor_context_set_tool() {
        let mut ctx = EditorContext::new();

        ctx.set_tool(EditorTool::Move);
        assert_eq!(ctx.current_tool(), EditorTool::Move);
        assert_eq!(ctx.gizmo.mode(), GizmoMode::Translate);

        ctx.set_tool(EditorTool::Rotate);
        assert_eq!(ctx.current_tool(), EditorTool::Rotate);
        assert_eq!(ctx.gizmo.mode(), GizmoMode::Rotate);

        ctx.set_tool(EditorTool::Scale);
        assert_eq!(ctx.current_tool(), EditorTool::Scale);
        assert_eq!(ctx.gizmo.mode(), GizmoMode::Scale);

        ctx.set_tool(EditorTool::Select);
        assert_eq!(ctx.current_tool(), EditorTool::Select);
        assert_eq!(ctx.gizmo.mode(), GizmoMode::None);
    }

    #[test]
    fn test_editor_context_camera() {
        let mut ctx = EditorContext::new();

        ctx.set_camera_offset(Vec2::new(100.0, 200.0));
        assert_eq!(ctx.camera_offset(), Vec2::new(100.0, 200.0));

        ctx.pan_camera(Vec2::new(50.0, -25.0));
        assert_eq!(ctx.camera_offset(), Vec2::new(150.0, 175.0));

        ctx.set_camera_zoom(2.0);
        assert_eq!(ctx.camera_zoom(), 2.0);

        ctx.zoom_camera(0.5);
        assert_eq!(ctx.camera_zoom(), 1.0);

        ctx.reset_camera();
        assert_eq!(ctx.camera_offset(), Vec2::ZERO);
        assert_eq!(ctx.camera_zoom(), 1.0);
    }

    #[test]
    fn test_editor_context_camera_zoom_clamp() {
        let mut ctx = EditorContext::new();

        ctx.set_camera_zoom(0.01);
        assert_eq!(ctx.camera_zoom(), 0.1); // Clamped to min

        ctx.set_camera_zoom(100.0);
        assert_eq!(ctx.camera_zoom(), 10.0); // Clamped to max
    }

    #[test]
    fn test_editor_context_coordinate_conversion() {
        let mut ctx = EditorContext::new();
        let window_center = Vec2::new(400.0, 300.0);

        // No offset, no zoom: screen center = world origin
        let world = ctx.screen_to_world(window_center, window_center);
        assert_eq!(world, Vec2::ZERO);

        let screen = ctx.world_to_screen(Vec2::ZERO, window_center);
        assert_eq!(screen, window_center);

        // With offset
        ctx.set_camera_offset(Vec2::new(100.0, 50.0));
        let world = ctx.screen_to_world(window_center, window_center);
        assert_eq!(world, Vec2::new(100.0, 50.0));

        // With zoom
        ctx.reset_camera();
        ctx.set_camera_zoom(2.0);
        let world = ctx.screen_to_world(Vec2::new(500.0, 300.0), window_center);
        assert_eq!(world, Vec2::new(50.0, 0.0)); // (500-400)/2 = 50
    }

    #[test]
    fn test_editor_context_grid() {
        let mut ctx = EditorContext::new();

        assert!(ctx.is_grid_visible());
        ctx.toggle_grid();
        assert!(!ctx.is_grid_visible());
        ctx.set_grid_visible(true);
        assert!(ctx.is_grid_visible());

        ctx.set_grid_size(64.0);
        assert_eq!(ctx.grid_size(), 64.0);

        // Grid size minimum
        ctx.set_grid_size(0.5);
        assert_eq!(ctx.grid_size(), 1.0);
    }

    #[test]
    fn test_editor_context_snap() {
        let mut ctx = EditorContext::new();
        ctx.set_grid_size(32.0);

        // Snap disabled
        let pos = Vec2::new(45.0, 78.0);
        assert_eq!(ctx.snap_position(pos), pos);

        // Snap enabled
        ctx.set_snap_to_grid(true);
        let snapped = ctx.snap_position(pos);
        // 45/32 = 1.4 rounds to 1 -> 32
        // 78/32 = 2.4 rounds to 2 -> 64
        assert_eq!(snapped, Vec2::new(32.0, 64.0));
    }

    #[test]
    fn test_editor_context_play_mode() {
        let mut ctx = EditorContext::new();

        assert!(!ctx.is_play_mode());
        ctx.enter_play_mode();
        assert!(ctx.is_play_mode());
        ctx.exit_play_mode();
        assert!(!ctx.is_play_mode());
        ctx.toggle_play_mode();
        assert!(ctx.is_play_mode());
    }

    #[test]
    fn test_editor_context_default_panels() {
        let ctx = EditorContext::new();

        // Should have 4 default panels
        assert_eq!(ctx.dock_area.panels().len(), 4);

        // Check panel positions
        assert!(ctx.dock_area.get_panel(PanelId::HIERARCHY).is_some());
        assert!(ctx.dock_area.get_panel(PanelId::INSPECTOR).is_some());
        assert!(ctx.dock_area.get_panel(PanelId::SCENE_VIEW).is_some());
        assert!(ctx.dock_area.get_panel(PanelId::ASSET_BROWSER).is_some());
    }

    #[test]
    fn test_editor_context_update_layout() {
        let mut ctx = EditorContext::new();
        ctx.update_layout(Vec2::new(1280.0, 720.0));

        // Scene view should have bounds set
        let scene_bounds = ctx.scene_view_bounds();
        assert!(scene_bounds.is_some());

        let bounds = scene_bounds.unwrap();
        assert!(bounds.width > 0.0);
        assert!(bounds.height > 0.0);
    }
}
