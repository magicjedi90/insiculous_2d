//! Centralized editor theme with design system color tokens.
//!
//! All editor colors and visual constants are defined here so that a single
//! change propagates across the entire editor. Derived from the target
//! mockup (`crates/editor/IdealEditor.png`).
//!
//! # Usage
//! ```ignore
//! let theme = EditorTheme::default();
//! ui.rect_filled(rect, theme.bg_primary);
//! ui.label_colored("Title", pos, theme.accent_cyan);
//! ```

use glam::Vec4;
use ui::Color;

/// Centralized design-system theme for the entire editor.
///
/// Every color used in panels, toolbars, inspector, hierarchy, status bar,
/// gizmos, and grid should reference a field on this struct — never a
/// hardcoded literal.
#[derive(Debug, Clone)]
pub struct EditorTheme {
    // ── Backgrounds ─────────────────────────────────────────────
    /// Main panel backgrounds (`#1e1e1e`)
    pub bg_primary: Color,
    /// Viewport / canvas area (`#000000`)
    pub bg_viewport: Color,
    /// Input fields, dropdowns (`#2d2d2d`)
    pub bg_input: Color,
    /// Panel header background (darker than bg_primary)
    pub bg_header: Color,

    // ── Accents ─────────────────────────────────────────────────
    /// Selection highlights, active buttons, "+ Add Component" (`#0078d4`)
    pub accent_blue: Color,
    /// Panel headers, interactive highlights, gizmo labels (`#00d9ff`)
    pub accent_cyan: Color,

    // ── Borders ─────────────────────────────────────────────────
    /// Panel borders — bright blue (`#007acc`)
    pub border_panel: Color,
    /// Grid lines, separators (`#333333`)
    pub border_subtle: Color,

    // ── Text ────────────────────────────────────────────────────
    /// Primary text (`#ffffff`)
    pub text_primary: Color,
    /// Secondary text, labels (`#cccccc`)
    pub text_secondary: Color,
    /// Disabled text, placeholders (`#888888`)
    pub text_muted: Color,

    // ── Gizmos ──────────────────────────────────────────────────
    /// X-axis gizmo color (green, horizontal) (`#00ff00`)
    pub gizmo_x: Color,
    /// Y-axis gizmo color (red, vertical) (`#ff0000`)
    pub gizmo_y: Color,
    /// Center/free-move gizmo handle
    pub gizmo_center: Color,

    // ── Play state ──────────────────────────────────────────────
    /// Play button / playing border tint (`#00cc44`)
    pub play_green: Color,
    /// Pause border tint (`#ffcc00`)
    pub pause_yellow: Color,
    /// Stop button (`#cc3333`)
    pub stop_red: Color,

    // ── Semantic ────────────────────────────────────────────────
    /// Error logs, validation (`#ff4444`)
    pub error_red: Color,
    /// Warning logs (`#ffcc00`)
    pub warn_yellow: Color,

    // ── Play control button backgrounds ─────────────────────────
    /// Dark green tint behind play/resume button
    pub play_button_bg: Color,
    /// Dark red tint behind stop button
    pub stop_button_bg: Color,

    // ── Separator ───────────────────────────────────────────────
    /// Thin separator lines between toolbar sections
    pub separator: Color,

    // ── Grid (Vec4 for renderer sprite pipeline) ────────────────
    /// Primary grid line color
    pub grid_primary: Vec4,
    /// Secondary (subdivision) grid line color
    pub grid_secondary: Vec4,
    /// Grid X-axis color (red)
    pub grid_axis_x: Vec4,
    /// Grid Y-axis color (green)
    pub grid_axis_y: Vec4,

    // ── Status bar ──────────────────────────────────────────────
    /// Status bar background (slightly darker than panels)
    pub status_bar_bg: Color,

    // ── Inspector ───────────────────────────────────────────────
    /// Inspector label color (field names)
    pub inspector_label: Color,
    /// Inspector value color (field values)
    pub inspector_value: Color,
    /// Inspector section header color
    pub inspector_header: Color,

    // ── Play-state viewport borders ─────────────────────────────
    /// Viewport border tint while editing
    pub border_editing: Color,
    /// Viewport border tint while playing
    pub border_playing: Color,
    /// Viewport border tint while paused
    pub border_paused: Color,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            // Backgrounds
            bg_primary: Color::from_hex(0x1e1e1e),
            bg_viewport: Color::BLACK,
            bg_input: Color::from_hex(0x2d2d2d),
            bg_header: Color::new(0.12, 0.12, 0.12, 1.0),

            // Accents
            accent_blue: Color::from_hex(0x0078d4),
            accent_cyan: Color::from_hex(0x00d9ff),

            // Borders
            border_panel: Color::from_hex(0x007acc),
            border_subtle: Color::from_hex(0x333333),

            // Text
            text_primary: Color::WHITE,
            text_secondary: Color::from_hex(0xcccccc),
            text_muted: Color::from_hex(0x888888),

            // Gizmos
            gizmo_x: Color::new(0.0, 1.0, 0.0, 1.0),
            gizmo_y: Color::new(1.0, 0.0, 0.0, 1.0),
            gizmo_center: Color::new(0.9, 0.9, 0.2, 1.0),

            // Play state
            play_green: Color::from_hex(0x00cc44),
            pause_yellow: Color::from_hex(0xffcc00),
            stop_red: Color::from_hex(0xcc3333),

            // Semantic
            error_red: Color::from_hex(0xff4444),
            warn_yellow: Color::from_hex(0xffcc00),

            // Play control button backgrounds
            play_button_bg: Color::new(0.15, 0.35, 0.15, 1.0),
            stop_button_bg: Color::new(0.4, 0.15, 0.15, 1.0),

            // Separator
            separator: Color::new(0.4, 0.4, 0.4, 0.6),

            // Grid (Vec4 for sprite renderer)
            grid_primary: Vec4::new(0.3, 0.3, 0.3, 0.5),
            grid_secondary: Vec4::new(0.25, 0.25, 0.25, 0.3),
            grid_axis_x: Vec4::new(0.8, 0.2, 0.2, 0.8),
            grid_axis_y: Vec4::new(0.2, 0.8, 0.2, 0.8),

            // Status bar
            status_bar_bg: Color::new(0.10, 0.10, 0.10, 1.0),

            // Inspector
            inspector_label: Color::from_hex(0xcccccc),
            inspector_value: Color::WHITE,
            inspector_header: Color::from_hex(0x00d9ff),

            // Play-state viewport borders
            border_editing: Color::new(0.0, 0.48, 0.83, 0.5),
            border_playing: Color::new(0.0, 0.8, 0.27, 0.8),
            border_paused: Color::new(1.0, 0.8, 0.0, 0.8),
        }
    }
}

impl EditorTheme {
    /// Create the default dark editor theme (matches the design mockup).
    pub fn dark() -> Self {
        Self::default()
    }

    /// Convert a theme `Color` to a `Vec4` for the renderer sprite pipeline.
    pub fn color_to_vec4(color: Color) -> Vec4 {
        Vec4::new(color.r, color.g, color.b, color.a)
    }

    /// Create `GridColors` from this theme.
    pub fn grid_colors(&self) -> crate::GridColors {
        crate::GridColors {
            primary: self.grid_primary,
            secondary: self.grid_secondary,
            axis_x: self.grid_axis_x,
            axis_y: self.grid_axis_y,
        }
    }

    /// Create `InspectorStyle` from this theme.
    pub fn inspector_style(&self) -> crate::InspectorStyle {
        crate::InspectorStyle {
            label_color: self.inspector_label,
            value_color: self.inspector_value,
            header_color: self.inspector_header,
            ..Default::default()
        }
    }

    /// Create `EditableFieldStyle` from this theme.
    pub fn editable_field_style(&self) -> crate::EditableFieldStyle {
        crate::EditableFieldStyle {
            label_color: self.inspector_label,
            value_color: self.inspector_value,
            header_color: self.inspector_header,
            ..Default::default()
        }
    }

    /// Get the viewport border color for a given play state.
    pub fn play_state_border(&self, state: crate::EditorPlayState) -> Color {
        match state {
            crate::EditorPlayState::Editing => self.border_editing,
            crate::EditorPlayState::Playing => self.border_playing,
            crate::EditorPlayState::Paused => self.border_paused,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_colors_are_opaque() {
        let theme = EditorTheme::default();
        // Background colors should be fully opaque
        assert_eq!(theme.bg_primary.a, 1.0);
        assert_eq!(theme.bg_viewport.a, 1.0);
        assert_eq!(theme.bg_input.a, 1.0);
        // Text colors should be fully opaque
        assert_eq!(theme.text_primary.a, 1.0);
        assert_eq!(theme.text_secondary.a, 1.0);
        assert_eq!(theme.text_muted.a, 1.0);
    }

    #[test]
    fn test_accent_colors_are_distinct() {
        let theme = EditorTheme::default();
        // Blue (#0078d4) vs Cyan (#00d9ff) differ in green and blue channels
        assert_ne!(theme.accent_blue.g, theme.accent_cyan.g);
        assert_ne!(theme.accent_blue.b, theme.accent_cyan.b);
    }

    #[test]
    fn test_play_state_borders_are_distinct() {
        let theme = EditorTheme::default();
        let editing = theme.border_editing;
        let playing = theme.border_playing;
        let paused = theme.border_paused;
        // Each state has a unique dominant channel
        assert!(editing.b > editing.r && editing.b > editing.g); // blue-ish
        assert!(playing.g > playing.r && playing.g > playing.b); // green-ish
        assert!(paused.r > paused.b); // warm/yellow-ish
    }

    #[test]
    fn test_grid_colors_conversion() {
        let theme = EditorTheme::default();
        let grid = theme.grid_colors();
        assert_eq!(grid.primary, theme.grid_primary);
        assert_eq!(grid.secondary, theme.grid_secondary);
        assert_eq!(grid.axis_x, theme.grid_axis_x);
        assert_eq!(grid.axis_y, theme.grid_axis_y);
    }

    #[test]
    fn test_inspector_style_conversion() {
        let theme = EditorTheme::default();
        let style = theme.inspector_style();
        assert_eq!(style.label_color, theme.inspector_label);
        assert_eq!(style.value_color, theme.inspector_value);
        assert_eq!(style.header_color, theme.inspector_header);
    }

    #[test]
    fn test_editable_field_style_conversion() {
        let theme = EditorTheme::default();
        let style = theme.editable_field_style();
        assert_eq!(style.label_color, theme.inspector_label);
        assert_eq!(style.value_color, theme.inspector_value);
        assert_eq!(style.header_color, theme.inspector_header);
    }

    #[test]
    fn test_play_state_border_method() {
        let theme = EditorTheme::default();
        assert_eq!(
            theme.play_state_border(crate::EditorPlayState::Editing),
            theme.border_editing
        );
        assert_eq!(
            theme.play_state_border(crate::EditorPlayState::Playing),
            theme.border_playing
        );
        assert_eq!(
            theme.play_state_border(crate::EditorPlayState::Paused),
            theme.border_paused
        );
    }

    #[test]
    fn test_color_to_vec4() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);
        let v = EditorTheme::color_to_vec4(color);
        assert!((v.x - 0.1).abs() < f32::EPSILON);
        assert!((v.y - 0.2).abs() < f32::EPSILON);
        assert!((v.z - 0.3).abs() < f32::EPSILON);
        assert!((v.w - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dark_is_default() {
        let dark = EditorTheme::dark();
        let default = EditorTheme::default();
        assert_eq!(dark.bg_primary, default.bg_primary);
        assert_eq!(dark.accent_blue, default.accent_blue);
    }
}
