//! Grid overlay rendering for the scene viewport.
//!
//! Renders a configurable grid with origin axes to help with entity placement
//! and scene navigation. Uses sprites (thin rectangles) for efficient rendering.

use glam::{Vec2, Vec4};
use renderer::sprite::{Sprite, SpriteBatcher};
use renderer::texture::TextureHandle;

/// Colors for grid rendering.
#[derive(Debug, Clone)]
pub struct GridColors {
    /// Primary grid line color
    pub primary: Vec4,
    /// Secondary (subdivision) grid line color
    pub secondary: Vec4,
    /// X axis color (typically red)
    pub axis_x: Vec4,
    /// Y axis color (typically green)
    pub axis_y: Vec4,
}

impl Default for GridColors {
    fn default() -> Self {
        Self {
            primary: Vec4::new(0.3, 0.3, 0.3, 0.5),
            secondary: Vec4::new(0.25, 0.25, 0.25, 0.3),
            axis_x: Vec4::new(0.8, 0.2, 0.2, 0.8),
            axis_y: Vec4::new(0.2, 0.8, 0.2, 0.8),
        }
    }
}

/// Configuration for grid rendering.
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Size of primary grid cells in world units
    pub primary_size: f32,
    /// Number of subdivisions per primary cell (0 = no subdivisions)
    pub subdivisions: u32,
    /// Line thickness in screen pixels
    pub line_thickness: f32,
    /// Axis line thickness in screen pixels
    pub axis_thickness: f32,
    /// Maximum number of grid lines to render (LOD limit)
    pub max_lines: usize,
    /// Minimum zoom level to show subdivisions
    pub subdivision_min_zoom: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            primary_size: 32.0,
            subdivisions: 4,
            line_thickness: 1.0,
            axis_thickness: 2.0,
            max_lines: 200,
            subdivision_min_zoom: 0.5,
        }
    }
}

/// Renders a grid overlay in the scene viewport.
///
/// The grid consists of:
/// - Primary grid lines at regular intervals
/// - Secondary subdivision lines (visible at higher zoom)
/// - X and Y axis lines through the origin
#[derive(Debug, Clone)]
pub struct GridRenderer {
    /// Grid configuration
    pub config: GridConfig,
    /// Grid colors
    pub colors: GridColors,
    /// Whether the grid is visible
    visible: bool,
    /// Whether axes are visible
    axes_visible: bool,
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GridRenderer {
    /// Create a new grid renderer with default settings.
    pub fn new() -> Self {
        Self {
            config: GridConfig::default(),
            colors: GridColors::default(),
            visible: true,
            axes_visible: true,
        }
    }

    /// Set grid visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if the grid is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle grid visibility.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// Set axes visibility.
    pub fn set_axes_visible(&mut self, visible: bool) {
        self.axes_visible = visible;
    }

    /// Set primary grid size.
    pub fn set_grid_size(&mut self, size: f32) {
        self.config.primary_size = size.max(1.0);
    }

    /// Get primary grid size.
    pub fn grid_size(&self) -> f32 {
        self.config.primary_size
    }

    /// Generate grid sprites for the visible area.
    ///
    /// Takes the visible world bounds and camera zoom, generates thin rectangle
    /// sprites for grid lines.
    ///
    /// Returns sprites to be added to a batcher.
    pub fn generate_grid_sprites(
        &self,
        visible_bounds: (f32, f32, f32, f32), // (min_x, min_y, max_x, max_y)
        camera_zoom: f32,
        white_texture: TextureHandle,
    ) -> Vec<Sprite> {
        if !self.visible {
            return Vec::new();
        }

        let mut sprites = Vec::new();
        let (min_x, min_y, max_x, max_y) = visible_bounds;

        // Calculate effective grid size based on zoom (LOD)
        let effective_grid_size = self.calculate_lod_grid_size(camera_zoom);

        // Calculate line thickness in world units
        let line_thickness_world = self.config.line_thickness / camera_zoom;
        let axis_thickness_world = self.config.axis_thickness / camera_zoom;

        // Generate primary grid lines
        let (h_lines, v_lines) = self.calculate_grid_lines(
            min_x, min_y, max_x, max_y,
            effective_grid_size,
        );

        // Check LOD limit
        let total_lines = h_lines.len() + v_lines.len();
        if total_lines <= self.config.max_lines {
            // Generate subdivision lines if zoom is high enough
            if camera_zoom >= self.config.subdivision_min_zoom && self.config.subdivisions > 0 {
                let sub_size = effective_grid_size / self.config.subdivisions as f32;
                let (h_sub, v_sub) = self.calculate_grid_lines(
                    min_x, min_y, max_x, max_y,
                    sub_size,
                );

                // Add subdivision lines (skip if they coincide with primary)
                for y in h_sub {
                    if !is_on_grid(y, effective_grid_size) {
                        sprites.push(self.create_horizontal_line(
                            min_x, max_x, y,
                            line_thickness_world * 0.5,
                            self.colors.secondary,
                            white_texture,
                            -0.2, // Behind primary grid
                        ));
                    }
                }

                for x in v_sub {
                    if !is_on_grid(x, effective_grid_size) {
                        sprites.push(self.create_vertical_line(
                            min_y, max_y, x,
                            line_thickness_world * 0.5,
                            self.colors.secondary,
                            white_texture,
                            -0.2,
                        ));
                    }
                }
            }

            // Add primary grid lines
            for y in h_lines {
                // Skip axis line (will be drawn separately)
                if y.abs() < 0.001 {
                    continue;
                }
                sprites.push(self.create_horizontal_line(
                    min_x, max_x, y,
                    line_thickness_world,
                    self.colors.primary,
                    white_texture,
                    -0.1, // Above subdivisions
                ));
            }

            for x in v_lines {
                // Skip axis line
                if x.abs() < 0.001 {
                    continue;
                }
                sprites.push(self.create_vertical_line(
                    min_y, max_y, x,
                    line_thickness_world,
                    self.colors.primary,
                    white_texture,
                    -0.1,
                ));
            }
        }

        // Always render axes if visible
        if self.axes_visible {
            // X axis (horizontal line at y=0, red)
            sprites.push(self.create_horizontal_line(
                min_x, max_x, 0.0,
                axis_thickness_world,
                self.colors.axis_x,
                white_texture,
                0.0, // On top of grid
            ));

            // Y axis (vertical line at x=0, green)
            sprites.push(self.create_vertical_line(
                min_y, max_y, 0.0,
                axis_thickness_world,
                self.colors.axis_y,
                white_texture,
                0.0,
            ));
        }

        sprites
    }

    /// Add grid sprites to a batcher.
    pub fn render_to_batcher(
        &self,
        batcher: &mut SpriteBatcher,
        visible_bounds: (f32, f32, f32, f32),
        camera_zoom: f32,
        white_texture: TextureHandle,
    ) {
        let sprites = self.generate_grid_sprites(visible_bounds, camera_zoom, white_texture);
        for sprite in &sprites {
            batcher.add_sprite(sprite);
        }
    }

    /// Calculate grid size with LOD (level of detail) based on zoom.
    ///
    /// At lower zoom levels, grid cells are merged to maintain readable density.
    fn calculate_lod_grid_size(&self, camera_zoom: f32) -> f32 {
        let base_size = self.config.primary_size;

        // Scale grid size inversely with zoom to maintain visual density
        // At zoom 0.5, double the grid size; at zoom 0.25, quadruple it
        if camera_zoom < 1.0 {
            let multiplier = (1.0 / camera_zoom).log2().ceil().exp2();
            base_size * multiplier
        } else {
            base_size
        }
    }

    /// Calculate grid line positions for the given bounds.
    fn calculate_grid_lines(
        &self,
        min_x: f32, min_y: f32, max_x: f32, max_y: f32,
        grid_size: f32,
    ) -> (Vec<f32>, Vec<f32>) {
        // Horizontal lines (varying Y)
        let start_y = (min_y / grid_size).floor() * grid_size;
        let end_y = (max_y / grid_size).ceil() * grid_size;
        let h_lines: Vec<f32> = (0..)
            .map(|i| start_y + i as f32 * grid_size)
            .take_while(|&y| y <= end_y)
            .collect();

        // Vertical lines (varying X)
        let start_x = (min_x / grid_size).floor() * grid_size;
        let end_x = (max_x / grid_size).ceil() * grid_size;
        let v_lines: Vec<f32> = (0..)
            .map(|i| start_x + i as f32 * grid_size)
            .take_while(|&x| x <= end_x)
            .collect();

        (h_lines, v_lines)
    }

    /// Create a horizontal line sprite.
    fn create_horizontal_line(
        &self,
        min_x: f32, max_x: f32, y: f32,
        thickness: f32,
        color: Vec4,
        texture: TextureHandle,
        depth: f32,
    ) -> Sprite {
        let length = max_x - min_x;
        let center_x = (min_x + max_x) * 0.5;

        Sprite::new(texture)
            .with_position(Vec2::new(center_x, y))
            .with_scale(Vec2::new(length, thickness))
            .with_color(color)
            .with_depth(depth)
    }

    /// Create a vertical line sprite.
    fn create_vertical_line(
        &self,
        min_y: f32, max_y: f32, x: f32,
        thickness: f32,
        color: Vec4,
        texture: TextureHandle,
        depth: f32,
    ) -> Sprite {
        let length = max_y - min_y;
        let center_y = (min_y + max_y) * 0.5;

        Sprite::new(texture)
            .with_position(Vec2::new(x, center_y))
            .with_scale(Vec2::new(thickness, length))
            .with_color(color)
            .with_depth(depth)
    }
}

/// Check if a value falls on the grid (within floating point tolerance).
fn is_on_grid(value: f32, grid_size: f32) -> bool {
    let remainder = (value / grid_size).fract().abs();
    remainder < 0.001 || remainder > 0.999
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_renderer_new() {
        let grid = GridRenderer::new();
        assert!(grid.is_visible());
        assert_eq!(grid.grid_size(), 32.0);
    }

    #[test]
    fn test_grid_visibility_toggle() {
        let mut grid = GridRenderer::new();
        assert!(grid.is_visible());

        grid.toggle_visible();
        assert!(!grid.is_visible());

        grid.toggle_visible();
        assert!(grid.is_visible());
    }

    #[test]
    fn test_grid_size_setting() {
        let mut grid = GridRenderer::new();
        grid.set_grid_size(64.0);
        assert_eq!(grid.grid_size(), 64.0);

        // Minimum size enforcement
        grid.set_grid_size(0.5);
        assert_eq!(grid.grid_size(), 1.0);
    }

    #[test]
    fn test_generate_grid_sprites_when_hidden() {
        let mut grid = GridRenderer::new();
        grid.set_visible(false);

        let sprites = grid.generate_grid_sprites(
            (-100.0, -100.0, 100.0, 100.0),
            1.0,
            TextureHandle::default(),
        );

        assert!(sprites.is_empty());
    }

    #[test]
    fn test_generate_grid_sprites_includes_axes() {
        let grid = GridRenderer::new();

        let sprites = grid.generate_grid_sprites(
            (-100.0, -100.0, 100.0, 100.0),
            1.0,
            TextureHandle::default(),
        );

        // Should have grid lines plus axes
        assert!(!sprites.is_empty());

        // Check that axis colors are present
        let has_x_axis = sprites.iter().any(|s| {
            s.color.x > 0.7 && s.color.y < 0.3 // Red-ish
        });
        let has_y_axis = sprites.iter().any(|s| {
            s.color.y > 0.7 && s.color.x < 0.3 // Green-ish
        });

        assert!(has_x_axis, "X axis should be rendered");
        assert!(has_y_axis, "Y axis should be rendered");
    }

    #[test]
    fn test_calculate_grid_lines() {
        let grid = GridRenderer::new();
        let (h_lines, v_lines) = grid.calculate_grid_lines(
            -64.0, -64.0, 64.0, 64.0,
            32.0,
        );

        // Should have lines at -64, -32, 0, 32, 64
        assert!(h_lines.len() >= 5);
        assert!(v_lines.len() >= 5);

        // Check that 0 is included
        assert!(h_lines.iter().any(|&y| y.abs() < 0.001));
        assert!(v_lines.iter().any(|&x| x.abs() < 0.001));
    }

    #[test]
    fn test_lod_grid_size() {
        let grid = GridRenderer::new();

        // At zoom 1.0, should use base size
        let size_1x = grid.calculate_lod_grid_size(1.0);
        assert_eq!(size_1x, 32.0);

        // At zoom 0.5, should double
        let size_05x = grid.calculate_lod_grid_size(0.5);
        assert_eq!(size_05x, 64.0);

        // At zoom 2.0, should use base size (no reduction)
        let size_2x = grid.calculate_lod_grid_size(2.0);
        assert_eq!(size_2x, 32.0);
    }

    #[test]
    fn test_is_on_grid() {
        assert!(is_on_grid(0.0, 32.0));
        assert!(is_on_grid(32.0, 32.0));
        assert!(is_on_grid(64.0, 32.0));
        assert!(is_on_grid(-32.0, 32.0));

        assert!(!is_on_grid(16.0, 32.0));
        assert!(!is_on_grid(8.0, 32.0));
    }

    #[test]
    fn test_grid_respects_max_lines() {
        let mut grid = GridRenderer::new();
        grid.config.max_lines = 10;

        // Large visible area would generate many lines
        let sprites = grid.generate_grid_sprites(
            (-1000.0, -1000.0, 1000.0, 1000.0),
            1.0,
            TextureHandle::default(),
        );

        // LOD should kick in and axes should still be visible
        assert!(!sprites.is_empty());
    }
}
