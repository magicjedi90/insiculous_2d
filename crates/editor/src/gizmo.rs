//! Transform gizmos for the editor.
//!
//! Gizmos are visual handles that allow manipulating entity transforms
//! (position, rotation, scale) directly in the scene view.

use glam::Vec2;
use ui::{Color, Rect, UIContext};

use crate::theme::EditorTheme;

/// The type of gizmo operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    /// No gizmo visible
    None,
    /// Move/translate gizmo with XY axes
    #[default]
    Translate,
    /// Rotation gizmo with circular handle
    Rotate,
    /// Scale gizmo with corner handles
    Scale,
}

impl GizmoMode {
    /// Get the display name for this mode.
    pub fn name(&self) -> &'static str {
        match self {
            GizmoMode::None => "None",
            GizmoMode::Translate => "Translate",
            GizmoMode::Rotate => "Rotate",
            GizmoMode::Scale => "Scale",
        }
    }
}

/// Which part of the gizmo is being interacted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandle {
    /// X-axis handle (red)
    AxisX,
    /// Y-axis handle (green)
    AxisY,
    /// Both axes (center/free movement)
    Center,
    /// Rotation ring
    Ring,
    /// Scale corner handle
    ScaleCorner(Corner),
}

/// Corner positions for scale handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Result of gizmo interaction.
#[derive(Debug, Clone, Copy)]
pub struct GizmoInteraction {
    /// Which handle is being dragged
    pub handle: Option<GizmoHandle>,
    /// Delta movement since last frame
    pub delta: Vec2,
    /// Rotation delta in radians (for rotation gizmo)
    pub rotation_delta: f32,
    /// Scale delta (for scale gizmo)
    pub scale_delta: Vec2,
}

impl Default for GizmoInteraction {
    fn default() -> Self {
        Self {
            handle: None,
            delta: Vec2::ZERO,
            rotation_delta: 0.0,
            scale_delta: Vec2::ZERO,
        }
    }
}

/// Complete color set for gizmo rendering.
///
/// Defaults match the editor's dark theme; `EditorTheme::gizmo_palette()`
/// produces a themed instance.
#[derive(Debug, Clone)]
pub struct GizmoPalette {
    /// X axis line and handle
    pub x: Color,
    /// Y axis line and handle
    pub y: Color,
    /// Center/free-move handle
    pub center: Color,
    /// X handle while hovered/dragged
    pub x_hover: Color,
    /// Y handle while hovered/dragged
    pub y_hover: Color,
    /// Center handle while hovered/dragged
    pub center_hover: Color,
    /// Rotation ring
    pub ring: Color,
    /// Current-rotation indicator line
    pub rotation_indicator: Color,
    /// Scale box outline
    pub scale_outline: Color,
    /// Scale corner handles
    pub scale_handle: Color,
    /// Scale corner handles while hovered/dragged
    pub scale_handle_hover: Color,
}

impl Default for GizmoPalette {
    fn default() -> Self {
        Self {
            x: Color::new(0.9, 0.2, 0.2, 1.0),      // Red
            y: Color::new(0.2, 0.9, 0.2, 1.0),      // Green
            center: Color::new(0.9, 0.9, 0.2, 1.0), // Yellow
            x_hover: Color::new(1.0, 0.4, 0.4, 1.0),
            y_hover: Color::new(0.4, 1.0, 0.4, 1.0),
            center_hover: Color::new(1.0, 1.0, 0.4, 1.0),
            ring: Color::new(0.3, 0.3, 0.9, 1.0),
            rotation_indicator: Color::new(0.9, 0.9, 0.9, 1.0),
            scale_outline: Color::new(0.6, 0.6, 0.6, 1.0),
            scale_handle: Color::new(0.7, 0.7, 0.7, 1.0),
            scale_handle_hover: Color::new(0.9, 0.9, 0.4, 1.0),
        }
    }
}

/// Transform gizmo for manipulating entity transforms.
#[derive(Debug, Clone)]
pub struct Gizmo {
    /// Current gizmo mode
    mode: GizmoMode,
    /// Position of the gizmo center (world space)
    position: Vec2,
    /// Current rotation of the entity (for rotation gizmo display)
    rotation: f32,
    /// Current scale of the entity (for scale gizmo display)
    scale: Vec2,
    /// Size of the gizmo handles
    handle_size: f32,
    /// Length of the axis lines
    axis_length: f32,
    /// Active handle being dragged
    active_handle: Option<GizmoHandle>,
    /// Last mouse position for delta calculation
    last_mouse_pos: Vec2,
    /// Colors for every gizmo element
    palette: GizmoPalette,
}

impl Default for Gizmo {
    fn default() -> Self {
        Self::new()
    }
}

impl Gizmo {
    /// Create a new gizmo.
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            handle_size: 12.0,
            axis_length: 80.0,
            active_handle: None,
            last_mouse_pos: Vec2::ZERO,
            palette: GizmoPalette::default(),
        }
    }

    /// Set the gizmo mode.
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
    }

    /// Get the current gizmo mode.
    pub fn mode(&self) -> GizmoMode {
        self.mode
    }

    /// Set the gizmo position (world space).
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Set the length of the gizmo axis arms in screen pixels (minimum 10).
    pub fn set_axis_length(&mut self, length: f32) {
        self.axis_length = length.max(10.0);
    }

    /// Get the length of the gizmo axis arms in screen pixels.
    pub fn axis_length(&self) -> f32 {
        self.axis_length
    }

    /// Get the gizmo position.
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Apply colors from the editor theme.
    pub fn apply_theme(&mut self, theme: &EditorTheme) {
        self.palette = theme.gizmo_palette();
    }

    /// Set the entity rotation (for rotation gizmo display).
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Set the entity scale (for scale gizmo display).
    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    /// Check if the gizmo is currently being dragged.
    pub fn is_active(&self) -> bool {
        self.active_handle.is_some()
    }

    /// Get the active handle being dragged.
    pub fn active_handle(&self) -> Option<GizmoHandle> {
        self.active_handle
    }

    /// Create a square rect centered at the given position, sized to handle_size.
    fn centered_handle_rect(&self, center: Vec2) -> Rect {
        Rect::new(
            center.x - self.handle_size / 2.0,
            center.y - self.handle_size / 2.0,
            self.handle_size,
            self.handle_size,
        )
    }

    /// Draw one translate axis (line + arrow handle) and return the handle's
    /// interactive bounds. The handle brightens while hovered or dragged.
    fn render_axis_handle(
        &self,
        ui: &mut UIContext,
        origin: Vec2,
        end: Vec2,
        base: Color,
        hover: Color,
        handle: GizmoHandle,
    ) -> Rect {
        ui.line(origin, end, base, 2.0);

        let bounds = self.centered_handle_rect(end);
        let hovered = bounds.contains(ui.mouse_pos());
        let color = if hovered || self.active_handle == Some(handle) {
            hover
        } else {
            base
        };
        ui.rect(bounds, color);
        bounds
    }

    /// Start drag bookkeeping for `handle` if `dragging` and no drag is active.
    fn begin_drag_if(&mut self, dragging: bool, handle: GizmoHandle, mouse_pos: Vec2) {
        if dragging && self.active_handle.is_none() {
            self.active_handle = Some(handle);
            self.last_mouse_pos = mouse_pos;
        }
    }

    /// Render the gizmo and handle interactions.
    ///
    /// # Arguments
    /// * `ui` - UI context for rendering
    /// * `screen_pos` - Screen position of the gizmo center
    ///
    /// Returns the gizmo interaction result.
    pub fn render(&mut self, ui: &mut UIContext, screen_pos: Vec2) -> GizmoInteraction {
        match self.mode {
            GizmoMode::None => GizmoInteraction::default(),
            GizmoMode::Translate => self.render_translate(ui, screen_pos),
            GizmoMode::Rotate => self.render_rotate(ui, screen_pos),
            GizmoMode::Scale => self.render_scale(ui, screen_pos),
        }
    }

    /// Render and handle translation gizmo.
    fn render_translate(&mut self, ui: &mut UIContext, screen_pos: Vec2) -> GizmoInteraction {
        let mut interaction = GizmoInteraction::default();
        let mouse_pos = ui.mouse_pos();

        // X axis (right), Y axis (up in screen space = negative Y)
        let x_end = screen_pos + Vec2::new(self.axis_length, 0.0);
        let y_end = screen_pos + Vec2::new(0.0, -self.axis_length);
        let x_arrow_bounds = self.render_axis_handle(
            ui, screen_pos, x_end, self.palette.x, self.palette.x_hover, GizmoHandle::AxisX,
        );
        let y_arrow_bounds = self.render_axis_handle(
            ui, screen_pos, y_end, self.palette.y, self.palette.y_hover, GizmoHandle::AxisY,
        );

        // Center handle (free movement, no axis line)
        let center_bounds = self.centered_handle_rect(screen_pos);
        let center_hovered = center_bounds.contains(mouse_pos);
        let center_color = if center_hovered || self.active_handle == Some(GizmoHandle::Center) {
            self.palette.center_hover
        } else {
            self.palette.center
        };
        ui.rect(center_bounds, center_color);

        // Handle interaction
        let result_x = ui.interact("gizmo_x", x_arrow_bounds, true);
        let result_y = ui.interact("gizmo_y", y_arrow_bounds, true);
        let result_center = ui.interact("gizmo_center", center_bounds, true);

        // Start dragging (first dragging handle wins; later calls no-op)
        self.begin_drag_if(result_x.dragging, GizmoHandle::AxisX, mouse_pos);
        self.begin_drag_if(result_y.dragging, GizmoHandle::AxisY, mouse_pos);
        self.begin_drag_if(result_center.dragging, GizmoHandle::Center, mouse_pos);

        // Continue dragging
        if let Some(handle) = self.active_handle {
            let delta = mouse_pos - self.last_mouse_pos;
            self.last_mouse_pos = mouse_pos;

            interaction.handle = Some(handle);
            interaction.delta = match handle {
                GizmoHandle::AxisX => Vec2::new(delta.x, 0.0),
                GizmoHandle::AxisY => Vec2::new(0.0, delta.y),
                GizmoHandle::Center => delta,
                _ => Vec2::ZERO,
            };

            // Stop dragging when mouse released
            if !result_x.dragging && !result_y.dragging && !result_center.dragging {
                self.active_handle = None;
            }
        }

        interaction
    }

    /// Render and handle rotation gizmo.
    fn render_rotate(&mut self, ui: &mut UIContext, screen_pos: Vec2) -> GizmoInteraction {
        let mut interaction = GizmoInteraction::default();
        let mouse_pos = ui.mouse_pos();

        // Draw rotation ring (approximated with line segments)
        let ring_radius = self.axis_length * 0.8;
        let segments = 32;
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = screen_pos + Vec2::new(angle1.cos(), angle1.sin()) * ring_radius;
            let p2 = screen_pos + Vec2::new(angle2.cos(), angle2.sin()) * ring_radius;

            ui.line(p1, p2, self.palette.ring, 2.0);
        }

        // Draw current rotation indicator (negated sin: world rotation is
        // CCW-positive but screen Y grows downward)
        let indicator_end = screen_pos + Vec2::new(
            self.rotation.cos() * ring_radius,
            -self.rotation.sin() * ring_radius,
        );
        ui.line(screen_pos, indicator_end, self.palette.rotation_indicator, 3.0);

        // Ring interaction (simplified - uses a rectangular area for now)
        let ring_bounds = Rect::new(
            screen_pos.x - ring_radius - 10.0,
            screen_pos.y - ring_radius - 10.0,
            ring_radius * 2.0 + 20.0,
            ring_radius * 2.0 + 20.0,
        );

        let result = ui.interact("gizmo_ring", ring_bounds, true);

        if result.dragging {
            self.begin_drag_if(true, GizmoHandle::Ring, mouse_pos);

            interaction.handle = Some(GizmoHandle::Ring);
            interaction.rotation_delta =
                crate::gizmo_math::world_rotation_delta(screen_pos, self.last_mouse_pos, mouse_pos);
            self.last_mouse_pos = mouse_pos;
        } else {
            self.active_handle = None;
        }

        interaction
    }

    /// Render and handle scale gizmo.
    fn render_scale(&mut self, ui: &mut UIContext, screen_pos: Vec2) -> GizmoInteraction {
        let mut interaction = GizmoInteraction::default();
        let mouse_pos = ui.mouse_pos();

        // Draw scale box outline
        let box_size = self.axis_length * 0.6;
        let half_size = box_size / 2.0;
        let box_bounds = Rect::new(
            screen_pos.x - half_size,
            screen_pos.y - half_size,
            box_size,
            box_size,
        );

        // Draw box outline
        ui.line(
            Vec2::new(box_bounds.x, box_bounds.y),
            Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y),
            self.palette.scale_outline,
            1.0,
        );
        ui.line(
            Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y),
            Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y + box_bounds.height),
            self.palette.scale_outline,
            1.0,
        );
        ui.line(
            Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y + box_bounds.height),
            Vec2::new(box_bounds.x, box_bounds.y + box_bounds.height),
            self.palette.scale_outline,
            1.0,
        );
        ui.line(
            Vec2::new(box_bounds.x, box_bounds.y + box_bounds.height),
            Vec2::new(box_bounds.x, box_bounds.y),
            self.palette.scale_outline,
            1.0,
        );

        // Draw corner handles
        let corners = [
            (Corner::TopLeft, Vec2::new(box_bounds.x, box_bounds.y)),
            (Corner::TopRight, Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y)),
            (Corner::BottomLeft, Vec2::new(box_bounds.x, box_bounds.y + box_bounds.height)),
            (Corner::BottomRight, Vec2::new(box_bounds.x + box_bounds.width, box_bounds.y + box_bounds.height)),
        ];

        for (corner, pos) in corners {
            let handle_bounds = self.centered_handle_rect(pos);

            let hovered = handle_bounds.contains(mouse_pos);
            let active = self.active_handle == Some(GizmoHandle::ScaleCorner(corner));
            let color = if hovered || active {
                self.palette.scale_handle_hover
            } else {
                self.palette.scale_handle
            };
            ui.rect(handle_bounds, color);

            let id = format!("gizmo_scale_{:?}", corner);
            let result = ui.interact(id.as_str(), handle_bounds, true);

            self.begin_drag_if(result.dragging, GizmoHandle::ScaleCorner(corner), mouse_pos);
        }

        // Process active scale drag
        if let Some(GizmoHandle::ScaleCorner(corner)) = self.active_handle {
            let delta = mouse_pos - self.last_mouse_pos;
            self.last_mouse_pos = mouse_pos;

            // Scale delta based on corner position
            let scale_delta = match corner {
                Corner::TopLeft => Vec2::new(-delta.x, delta.y),
                Corner::TopRight => Vec2::new(delta.x, delta.y),
                Corner::BottomLeft => Vec2::new(-delta.x, -delta.y),
                Corner::BottomRight => Vec2::new(delta.x, -delta.y),
            } * 0.01; // Scale sensitivity

            interaction.handle = Some(GizmoHandle::ScaleCorner(corner));
            interaction.scale_delta = scale_delta;

            // Check if any handle is still being dragged
            let still_dragging = corners.iter().any(|(c, _)| {
                let id = format!("gizmo_scale_{:?}", c);
                let bounds = self.centered_handle_rect(screen_pos);
                ui.interact(id.as_str(), bounds, true).dragging
            });

            if !still_dragging {
                self.active_handle = None;
            }
        }

        interaction
    }

    /// Cancel any active gizmo operation.
    pub fn cancel(&mut self) {
        self.active_handle = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_mode_default() {
        assert_eq!(GizmoMode::default(), GizmoMode::Translate);
    }

    #[test]
    fn test_gizmo_mode_names() {
        assert_eq!(GizmoMode::None.name(), "None");
        assert_eq!(GizmoMode::Translate.name(), "Translate");
        assert_eq!(GizmoMode::Rotate.name(), "Rotate");
        assert_eq!(GizmoMode::Scale.name(), "Scale");
    }

    #[test]
    fn test_gizmo_new() {
        let gizmo = Gizmo::new();
        assert_eq!(gizmo.mode(), GizmoMode::Translate);
        assert_eq!(gizmo.position(), Vec2::ZERO);
        assert!(!gizmo.is_active());
    }

    #[test]
    fn test_gizmo_set_mode() {
        let mut gizmo = Gizmo::new();
        gizmo.set_mode(GizmoMode::Rotate);
        assert_eq!(gizmo.mode(), GizmoMode::Rotate);
    }

    #[test]
    fn test_gizmo_set_position() {
        let mut gizmo = Gizmo::new();
        gizmo.set_position(Vec2::new(100.0, 200.0));
        assert_eq!(gizmo.position(), Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_gizmo_cancel() {
        let mut gizmo = Gizmo::new();
        gizmo.active_handle = Some(GizmoHandle::AxisX);
        assert!(gizmo.is_active());

        gizmo.cancel();
        assert!(!gizmo.is_active());
    }

    #[test]
    fn test_gizmo_interaction_default() {
        let interaction = GizmoInteraction::default();
        assert!(interaction.handle.is_none());
        assert_eq!(interaction.delta, Vec2::ZERO);
        assert_eq!(interaction.rotation_delta, 0.0);
        assert_eq!(interaction.scale_delta, Vec2::ZERO);
    }

    #[test]
    fn test_corner_enum() {
        // Test that corners can be compared
        assert_eq!(Corner::TopLeft, Corner::TopLeft);
        assert_ne!(Corner::TopLeft, Corner::BottomRight);
    }
}
