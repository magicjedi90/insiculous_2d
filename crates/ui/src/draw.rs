//! UI draw commands for rendering.
//!
//! This module defines the draw primitives that the UI system generates.
//! These are converted to sprites by the renderer integration layer.

use glam::Vec2;
use crate::{Color, Rect};

/// Data for rendering a single glyph.
#[derive(Debug, Clone)]
pub struct GlyphDrawData {
    /// Glyph bitmap data (grayscale, one byte per pixel)
    pub bitmap: Vec<u8>,
    /// Width of the glyph bitmap
    pub width: u32,
    /// Height of the glyph bitmap
    pub height: u32,
    /// X position relative to text origin
    pub x: f32,
    /// Y position relative to text origin
    pub y: f32,
    /// The character this glyph represents
    pub character: char,
}

/// Data for rendering text with rasterized glyphs.
#[derive(Debug, Clone)]
pub struct TextDrawData {
    /// Text string (for reference)
    pub text: String,
    /// Position of the text origin (top-left)
    pub position: Vec2,
    /// Text color
    pub color: Color,
    /// Font size used
    pub font_size: f32,
    /// Total width of the laid out text
    pub width: f32,
    /// Total height of the laid out text
    pub height: f32,
    /// Individual glyphs with positions and bitmaps
    pub glyphs: Vec<GlyphDrawData>,
}

/// A UI draw command representing a visual primitive to render.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect {
        bounds: Rect,
        color: Color,
        corner_radius: f32,
        depth: f32,
    },
    /// Draw a rectangle border (outline)
    RectBorder {
        bounds: Rect,
        color: Color,
        width: f32,
        corner_radius: f32,
        depth: f32,
    },
    /// Draw text with rasterized glyph data
    Text {
        data: TextDrawData,
        depth: f32,
    },
    /// Draw text without font data (fallback/placeholder)
    TextPlaceholder {
        text: String,
        position: Vec2,
        color: Color,
        font_size: f32,
        depth: f32,
    },
    /// Draw a circle
    Circle {
        center: Vec2,
        radius: f32,
        color: Color,
        depth: f32,
    },
    /// Draw a line
    Line {
        start: Vec2,
        end: Vec2,
        color: Color,
        width: f32,
        depth: f32,
    },
}

impl DrawCommand {
    /// Get the depth of this draw command for sorting.
    pub fn depth(&self) -> f32 {
        match self {
            DrawCommand::Rect { depth, .. } => *depth,
            DrawCommand::RectBorder { depth, .. } => *depth,
            DrawCommand::Text { depth, .. } => *depth,
            DrawCommand::TextPlaceholder { depth, .. } => *depth,
            DrawCommand::Circle { depth, .. } => *depth,
            DrawCommand::Line { depth, .. } => *depth,
        }
    }
}

/// A draw list that collects all UI draw commands for a frame.
#[derive(Debug, Clone, Default)]
pub struct DrawList {
    commands: Vec<DrawCommand>,
    /// Base depth for all commands in this list
    base_depth: f32,
}

impl DrawList {
    /// Create a new empty draw list.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            base_depth: 900.0, // UI renders on top of game content (< camera far=1000)
        }
    }

    /// Create a new draw list with a custom base depth.
    pub fn with_base_depth(base_depth: f32) -> Self {
        Self {
            commands: Vec::new(),
            base_depth,
        }
    }

    /// Clear all draw commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Get all draw commands.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Get the number of draw commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the draw list is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Add a filled rectangle.
    pub fn rect(&mut self, bounds: Rect, color: Color) {
        self.rect_rounded(bounds, color, 0.0);
    }

    /// Add a filled rectangle with rounded corners.
    pub fn rect_rounded(&mut self, bounds: Rect, color: Color, corner_radius: f32) {
        self.commands.push(DrawCommand::Rect {
            bounds,
            color,
            corner_radius,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add a rectangle border.
    pub fn rect_border(&mut self, bounds: Rect, color: Color, width: f32) {
        self.rect_border_rounded(bounds, color, width, 0.0);
    }

    /// Add a rectangle border with rounded corners.
    pub fn rect_border_rounded(&mut self, bounds: Rect, color: Color, width: f32, corner_radius: f32) {
        self.commands.push(DrawCommand::RectBorder {
            bounds,
            color,
            width,
            corner_radius,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add text placeholder (renders as approximate rectangle without font).
    pub fn text_placeholder(&mut self, text: impl Into<String>, position: Vec2, color: Color, font_size: f32) {
        self.commands.push(DrawCommand::TextPlaceholder {
            text: text.into(),
            position,
            color,
            font_size,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add text with rasterized glyph data.
    pub fn text(&mut self, data: TextDrawData) {
        self.commands.push(DrawCommand::Text {
            data,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add text - convenience method that takes raw parameters.
    /// Creates a TextDrawData with empty glyphs (use text() with TextDrawData for actual rendering).
    pub fn text_simple(&mut self, text: impl Into<String>, position: Vec2, color: Color, font_size: f32) {
        let text_str = text.into();
        // Estimate width based on character count (rough approximation)
        let estimated_width = text_str.len() as f32 * font_size * 0.6;
        self.commands.push(DrawCommand::Text {
            data: TextDrawData {
                text: text_str,
                position,
                color,
                font_size,
                width: estimated_width,
                height: font_size,
                glyphs: Vec::new(), // Empty - will be rendered as placeholder if no glyphs
            },
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add a filled circle.
    pub fn circle(&mut self, center: Vec2, radius: f32, color: Color) {
        self.commands.push(DrawCommand::Circle {
            center,
            radius,
            color,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Add a line.
    pub fn line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) {
        self.commands.push(DrawCommand::Line {
            start,
            end,
            color,
            width,
            depth: self.base_depth + self.commands.len() as f32 * 0.001,
        });
    }

    /// Draw a panel background with border.
    pub fn panel(&mut self, bounds: Rect, background: Color, border: Color, border_width: f32, corner_radius: f32) {
        // Background first
        self.rect_rounded(bounds, background, corner_radius);
        // Then border on top
        if border_width > 0.0 {
            self.rect_border_rounded(bounds, border, border_width, corner_radius);
        }
    }

    /// Draw a button.
    #[allow(clippy::too_many_arguments)]
    pub fn button(&mut self, bounds: Rect, background: Color, border: Color, border_width: f32, corner_radius: f32, label: &str, text_color: Color, font_size: f32) {
        self.panel(bounds, background, border, border_width, corner_radius);
        // Center text in button
        let text_pos = bounds.center();
        self.text_placeholder(label, text_pos, text_color, font_size);
    }

    /// Draw a slider track and thumb.
    #[allow(clippy::too_many_arguments)]
    pub fn slider(&mut self, track_bounds: Rect, thumb_center: Vec2, thumb_radius: f32, track_background: Color, track_fill: Color, thumb_color: Color, fill_amount: f32) {
        // Draw track background
        self.rect_rounded(track_bounds, track_background, track_bounds.height / 2.0);

        // Draw filled portion
        if fill_amount > 0.0 {
            let fill_width = track_bounds.width * fill_amount;
            let fill_bounds = Rect::new(track_bounds.x, track_bounds.y, fill_width, track_bounds.height);
            self.rect_rounded(fill_bounds, track_fill, track_bounds.height / 2.0);
        }

        // Draw thumb
        self.circle(thumb_center, thumb_radius, thumb_color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_list_new() {
        let list = DrawList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_draw_list_rect() {
        let mut list = DrawList::new();
        list.rect(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED);
        assert_eq!(list.len(), 1);

        if let DrawCommand::Rect { bounds, color, corner_radius, .. } = &list.commands()[0] {
            assert_eq!(bounds.width, 100.0);
            assert_eq!(bounds.height, 50.0);
            assert_eq!(*color, Color::RED);
            assert_eq!(*corner_radius, 0.0);
        } else {
            panic!("Expected Rect command");
        }
    }

    #[test]
    fn test_draw_list_rect_rounded() {
        let mut list = DrawList::new();
        list.rect_rounded(Rect::new(0.0, 0.0, 100.0, 50.0), Color::BLUE, 8.0);

        if let DrawCommand::Rect { corner_radius, .. } = &list.commands()[0] {
            assert_eq!(*corner_radius, 8.0);
        } else {
            panic!("Expected Rect command");
        }
    }

    #[test]
    fn test_draw_list_text_simple() {
        let mut list = DrawList::new();
        list.text_simple("Hello", Vec2::new(10.0, 20.0), Color::WHITE, 16.0);

        if let DrawCommand::Text { data, .. } = &list.commands()[0] {
            assert_eq!(data.text, "Hello");
            assert_eq!(data.position, Vec2::new(10.0, 20.0));
            assert_eq!(data.font_size, 16.0);
            assert!(data.glyphs.is_empty()); // text_simple creates empty glyphs
        } else {
            panic!("Expected Text command");
        }
    }

    #[test]
    fn test_draw_list_text_placeholder() {
        let mut list = DrawList::new();
        list.text_placeholder("World", Vec2::new(50.0, 60.0), Color::RED, 24.0);

        if let DrawCommand::TextPlaceholder { text, position, font_size, color, .. } = &list.commands()[0] {
            assert_eq!(text, "World");
            assert_eq!(*position, Vec2::new(50.0, 60.0));
            assert_eq!(*font_size, 24.0);
            assert_eq!(*color, Color::RED);
        } else {
            panic!("Expected TextPlaceholder command");
        }
    }

    #[test]
    fn test_draw_list_text_with_data() {
        let mut list = DrawList::new();
        let text_data = TextDrawData {
            text: "Test".to_string(),
            position: Vec2::new(100.0, 200.0),
            color: Color::GREEN,
            font_size: 32.0,
            width: 80.0,
            height: 32.0,
            glyphs: vec![
                GlyphDrawData {
                    bitmap: vec![255; 16],
                    width: 4,
                    height: 4,
                    x: 0.0,
                    y: 0.0,
                    character: 'T',
                },
            ],
        };
        list.text(text_data);

        if let DrawCommand::Text { data, .. } = &list.commands()[0] {
            assert_eq!(data.text, "Test");
            assert_eq!(data.position, Vec2::new(100.0, 200.0));
            assert_eq!(data.glyphs.len(), 1);
            assert_eq!(data.glyphs[0].character, 'T');
        } else {
            panic!("Expected Text command");
        }
    }

    #[test]
    fn test_draw_list_circle() {
        let mut list = DrawList::new();
        list.circle(Vec2::new(50.0, 50.0), 25.0, Color::GREEN);

        if let DrawCommand::Circle { center, radius, color, .. } = &list.commands()[0] {
            assert_eq!(*center, Vec2::new(50.0, 50.0));
            assert_eq!(*radius, 25.0);
            assert_eq!(*color, Color::GREEN);
        } else {
            panic!("Expected Circle command");
        }
    }

    #[test]
    fn test_draw_list_clear() {
        let mut list = DrawList::new();
        list.rect(Rect::default(), Color::RED);
        list.rect(Rect::default(), Color::BLUE);
        assert_eq!(list.len(), 2);

        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_draw_command_depth() {
        let cmd = DrawCommand::Rect {
            bounds: Rect::default(),
            color: Color::RED,
            corner_radius: 0.0,
            depth: 5.0,
        };
        assert_eq!(cmd.depth(), 5.0);
    }

    #[test]
    fn test_draw_list_depth_ordering() {
        let mut list = DrawList::new();
        list.rect(Rect::default(), Color::RED);
        list.rect(Rect::default(), Color::BLUE);
        list.rect(Rect::default(), Color::GREEN);

        // Each command should have increasing depth
        let depths: Vec<f32> = list.commands().iter().map(|c| c.depth()).collect();
        assert!(depths[0] < depths[1]);
        assert!(depths[1] < depths[2]);
    }
}
