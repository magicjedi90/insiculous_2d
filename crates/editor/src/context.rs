//! Editor context extending the game context.
//!
//! The EditorContext wraps a GameContext and adds editor-specific state
//! like selection, gizmos, and editor camera.

use std::path::{Path, PathBuf};

use engine_core::scene_data::EditorSettings;
use glam::Vec2;

use crate::{
    editor_input::EditorInputMapping,
    grid::GridRenderer,
    hierarchy::HierarchyPanel,
    picking::EntityPicker,
    viewport::SceneViewport,
    viewport_input::ViewportInputHandler,
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
    /// Scene viewport for rendering and navigation
    pub viewport: SceneViewport,
    /// Grid renderer for scene view
    pub grid: GridRenderer,
    /// Entity picker for selection
    pub picker: EntityPicker,
    /// Viewport input handler
    pub viewport_input: ViewportInputHandler,
    /// Editor input mapping
    pub input_mapping: EditorInputMapping,
    /// Hierarchy panel for entity tree view
    pub hierarchy: HierarchyPanel,
    /// Snap to grid enabled
    snap_to_grid: bool,
    /// Whether the editor is in play mode (running the game)
    play_mode: bool,
    /// Path to currently loaded scene (None = unsaved new scene)
    current_scene_path: Option<PathBuf>,
    /// Scene name for display
    scene_name: String,
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
            toolbar: Toolbar::new().with_position(Vec2::new(220.0, 54.0)), // Inside scene view, below panel header
            menu_bar: MenuBar::editor_default(),
            dock_area,
            viewport: SceneViewport::new(),
            grid: GridRenderer::new(),
            picker: EntityPicker::new(),
            viewport_input: ViewportInputHandler::new(),
            input_mapping: EditorInputMapping::new(),
            hierarchy: HierarchyPanel::new(),
            snap_to_grid: false,
            play_mode: false,
            current_scene_path: None,
            scene_name: "Untitled".to_string(),
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
    // These delegate to the SceneViewport for camera control

    /// Get the camera offset (position).
    pub fn camera_offset(&self) -> Vec2 {
        self.viewport.camera_position()
    }

    /// Set the camera offset (position).
    pub fn set_camera_offset(&mut self, offset: Vec2) {
        self.viewport.set_camera_position(offset);
    }

    /// Pan the camera by a delta.
    pub fn pan_camera(&mut self, delta: Vec2) {
        self.viewport.pan_immediate(delta);
    }

    /// Get the camera zoom level.
    pub fn camera_zoom(&self) -> f32 {
        self.viewport.camera_zoom()
    }

    /// Set the camera zoom level.
    pub fn set_camera_zoom(&mut self, zoom: f32) {
        self.viewport.set_camera_zoom(zoom);
    }

    /// Zoom the camera by a factor.
    pub fn zoom_camera(&mut self, factor: f32) {
        let new_zoom = self.viewport.camera_zoom() * factor;
        self.viewport.set_camera_zoom(new_zoom);
    }

    /// Reset camera to default view.
    pub fn reset_camera(&mut self) {
        self.viewport.reset_camera_immediate();
    }

    /// Convert screen position to world position.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.viewport.screen_to_world(screen_pos)
    }

    /// Convert world position to screen position.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        self.viewport.world_to_screen(world_pos)
    }

    // ================== Grid Methods ==================
    // These delegate to the GridRenderer

    /// Check if the grid is visible.
    pub fn is_grid_visible(&self) -> bool {
        self.grid.is_visible()
    }

    /// Set grid visibility.
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.grid.set_visible(visible);
    }

    /// Toggle grid visibility.
    pub fn toggle_grid(&mut self) {
        self.grid.toggle_visible();
    }

    /// Get the grid size.
    pub fn grid_size(&self) -> f32 {
        self.grid.grid_size()
    }

    /// Set the grid size.
    pub fn set_grid_size(&mut self, size: f32) {
        self.grid.set_grid_size(size);
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
            let grid_size = self.grid.grid_size();
            Vec2::new(
                (pos.x / grid_size).round() * grid_size,
                (pos.y / grid_size).round() * grid_size,
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

    // ================== Scene Methods ==================

    /// Get the path to the currently loaded scene.
    pub fn current_scene_path(&self) -> Option<&Path> {
        self.current_scene_path.as_deref()
    }

    /// Set the current scene path and name.
    pub fn set_current_scene(&mut self, path: Option<PathBuf>, name: String) {
        self.current_scene_path = path;
        self.scene_name = name;
    }

    /// Get the current scene name.
    pub fn scene_name(&self) -> &str {
        &self.scene_name
    }

    /// Create EditorSettings from current editor state.
    pub fn to_editor_settings(&self) -> EditorSettings {
        EditorSettings {
            camera_position: (self.camera_offset().x, self.camera_offset().y),
            camera_zoom: self.camera_zoom(),
        }
    }

    /// Apply EditorSettings to restore camera state.
    pub fn apply_editor_settings(&mut self, settings: &EditorSettings) {
        self.set_camera_offset(Vec2::new(settings.camera_position.0, settings.camera_position.1));
        self.set_camera_zoom(settings.camera_zoom);
    }

    // ================== Layout Methods ==================

    /// Update layout based on window size.
    ///
    /// This should be called each frame before rendering to ensure
    /// panels are properly sized and viewport bounds are updated.
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

        // Update viewport bounds from scene view panel
        if let Some(scene_bounds) = self.scene_view_bounds() {
            self.viewport.set_viewport_bounds(scene_bounds);
        }

        // Note: The gizmo position should be set by the caller based on entity transform
        // after calling update_layout, using self.selection.primary() to get the selected entity
    }

    /// Update viewport interpolation. Call each frame.
    pub fn update_viewport(&mut self, delta_time: f32) {
        self.viewport.update(delta_time);
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

    // ================== Gizmo Integration ==================

    /// Update gizmo position based on selected entity positions.
    ///
    /// Call this after selection changes to position the gizmo at the
    /// selection center.
    pub fn update_gizmo_from_selection(&mut self, entity_positions: &[(ecs::EntityId, Vec2)]) {
        // Find positions of selected entities
        let selected_positions: Vec<Vec2> = entity_positions
            .iter()
            .filter(|(id, _)| self.selection.contains(*id))
            .map(|(_, pos)| *pos)
            .collect();

        if selected_positions.is_empty() {
            // No selection - hide gizmo
            self.gizmo.set_mode(crate::GizmoMode::None);
            return;
        }

        // Calculate center of selection
        let center = if selected_positions.len() == 1 {
            selected_positions[0]
        } else {
            let sum: Vec2 = selected_positions.iter().copied().sum();
            sum / selected_positions.len() as f32
        };

        // Update gizmo position (world coords)
        self.gizmo.set_position(center);
    }

    /// Get the screen position for the gizmo based on its world position.
    ///
    /// Use this to pass to gizmo.render().
    pub fn gizmo_screen_position(&self) -> Vec2 {
        self.viewport.world_to_screen(self.gizmo.position())
    }

    /// Convert a gizmo delta from screen space to world space.
    ///
    /// The gizmo returns deltas in screen pixels. This converts them
    /// to world units accounting for camera zoom and Y-axis inversion
    /// (screen Y increases downward, world Y increases upward).
    pub fn gizmo_delta_to_world(&self, screen_delta: Vec2) -> Vec2 {
        Vec2::new(
            screen_delta.x / self.viewport.camera_zoom(),
            -screen_delta.y / self.viewport.camera_zoom(), // Negate Y for world coords
        )
    }

    /// Check if the gizmo should take priority over picking.
    ///
    /// Returns true if the gizmo is currently being interacted with,
    /// meaning picking should be skipped.
    pub fn gizmo_has_priority(&self) -> bool {
        self.gizmo.is_active()
    }

    /// Focus the viewport camera on the current selection.
    pub fn focus_on_selection(&mut self, entity_positions: &[(ecs::EntityId, Vec2)]) {
        let selected_positions: Vec<Vec2> = entity_positions
            .iter()
            .filter(|(id, _)| self.selection.contains(*id))
            .map(|(_, pos)| *pos)
            .collect();

        if !selected_positions.is_empty() {
            self.viewport.focus_on_bounds(&selected_positions);
        }
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
        // Set up viewport bounds first
        ctx.update_layout(Vec2::new(800.0, 600.0));

        // Get viewport center
        let viewport_center = ctx.viewport.viewport_center();

        // No offset, no zoom: viewport center = world origin
        let world = ctx.screen_to_world(viewport_center);
        assert!((world.x).abs() < 0.01);
        assert!((world.y).abs() < 0.01);

        let screen = ctx.world_to_screen(Vec2::ZERO);
        assert!((screen.x - viewport_center.x).abs() < 0.01);
        assert!((screen.y - viewport_center.y).abs() < 0.01);

        // With offset
        ctx.set_camera_offset(Vec2::new(100.0, 50.0));
        let world = ctx.screen_to_world(viewport_center);
        assert!((world.x - 100.0).abs() < 0.01);
        assert!((world.y - 50.0).abs() < 0.01);
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

    #[test]
    fn test_gizmo_delta_to_world() {
        let mut ctx = EditorContext::new();

        // At zoom 1.0, X unchanged but Y inverted (screen down = world up)
        let delta = Vec2::new(100.0, 50.0);
        let world_delta = ctx.gizmo_delta_to_world(delta);
        assert_eq!(world_delta, Vec2::new(100.0, -50.0));

        // At zoom 2.0, delta is halved and Y inverted
        ctx.set_camera_zoom(2.0);
        let world_delta = ctx.gizmo_delta_to_world(delta);
        assert_eq!(world_delta, Vec2::new(50.0, -25.0));
    }

    #[test]
    fn test_gizmo_has_priority() {
        let ctx = EditorContext::new();

        // Gizmo not active by default
        assert!(!ctx.gizmo_has_priority());
    }

    #[test]
    fn test_update_gizmo_from_selection_empty() {
        let mut ctx = EditorContext::new();
        ctx.set_tool(EditorTool::Move); // Set to Move so gizmo shows

        // Empty selection should hide gizmo
        let entities: Vec<(ecs::EntityId, Vec2)> = vec![];
        ctx.update_gizmo_from_selection(&entities);

        assert_eq!(ctx.gizmo.mode(), GizmoMode::None);
    }

    #[test]
    fn test_update_gizmo_from_selection_single() {
        let mut ctx = EditorContext::new();
        ctx.set_tool(EditorTool::Move);

        let entity_id = ecs::EntityId::with_generation(1, 1);
        ctx.selection.select(entity_id);

        let entities = vec![(entity_id, Vec2::new(100.0, 200.0))];
        ctx.update_gizmo_from_selection(&entities);

        assert_eq!(ctx.gizmo.position(), Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_update_gizmo_from_selection_multiple() {
        let mut ctx = EditorContext::new();
        ctx.set_tool(EditorTool::Move);

        let id1 = ecs::EntityId::with_generation(1, 1);
        let id2 = ecs::EntityId::with_generation(2, 1);
        ctx.selection.select(id1);
        ctx.selection.add(id2);

        let entities = vec![
            (id1, Vec2::new(0.0, 0.0)),
            (id2, Vec2::new(100.0, 100.0)),
        ];
        ctx.update_gizmo_from_selection(&entities);

        // Gizmo should be at center of selection
        assert_eq!(ctx.gizmo.position(), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_gizmo_screen_position() {
        let mut ctx = EditorContext::new();
        ctx.update_layout(Vec2::new(800.0, 600.0));

        // Set gizmo at world origin
        ctx.gizmo.set_position(Vec2::ZERO);

        // Should be at viewport center
        let screen_pos = ctx.gizmo_screen_position();
        let viewport_center = ctx.viewport.viewport_center();
        assert!((screen_pos.x - viewport_center.x).abs() < 0.01);
        assert!((screen_pos.y - viewport_center.y).abs() < 0.01);
    }

    #[test]
    fn test_editor_context_scene_tracking() {
        let mut ctx = EditorContext::new();

        // Initially no scene
        assert!(ctx.current_scene_path().is_none());
        assert_eq!(ctx.scene_name(), "Untitled");

        // Set a scene
        ctx.set_current_scene(
            Some(std::path::PathBuf::from("/test/scene.ron")),
            "My Scene".to_string(),
        );

        assert_eq!(
            ctx.current_scene_path(),
            Some(std::path::Path::new("/test/scene.ron"))
        );
        assert_eq!(ctx.scene_name(), "My Scene");
    }

    #[test]
    fn test_editor_settings_conversion() {
        let mut ctx = EditorContext::new();
        ctx.set_camera_offset(Vec2::new(150.0, -200.0));
        ctx.set_camera_zoom(1.5);

        let settings = ctx.to_editor_settings();

        assert_eq!(settings.camera_position, (150.0, -200.0));
        assert_eq!(settings.camera_zoom, 1.5);
    }

    #[test]
    fn test_apply_editor_settings() {
        let mut ctx = EditorContext::new();

        let settings = engine_core::scene_data::EditorSettings {
            camera_position: (100.0, 50.0),
            camera_zoom: 2.0,
        };

        ctx.apply_editor_settings(&settings);

        assert_eq!(ctx.camera_offset(), Vec2::new(100.0, 50.0));
        assert_eq!(ctx.camera_zoom(), 2.0);
    }
}
