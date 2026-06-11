//! Text layout and measurement.
//!
//! Lays out a string into positioned glyphs (filling the glyph cache on the
//! way) and measures text dimensions without rasterizing.

use fontdue::Font;
use glam::Vec2;

use super::glyph_cache::GlyphCache;
use super::{FontError, GlyphInfo};

/// Text layout information for a string of text.
#[derive(Debug, Clone)]
pub struct TextLayout {
    /// Total width of the text in pixels
    pub width: f32,
    /// Total height of the text in pixels
    pub height: f32,
    /// Individual glyph positions and info
    pub glyphs: Vec<LayoutGlyph>,
}

/// A single glyph in a text layout.
#[derive(Debug, Clone)]
pub struct LayoutGlyph {
    /// Character this glyph represents
    pub character: char,
    /// X position relative to text origin
    pub x: f32,
    /// Y position relative to text origin (baseline)
    pub y: f32,
    /// Glyph info with bitmap data
    pub info: GlyphInfo,
}

/// Layout a string of text, returning positions and glyph info for each character.
///
/// Glyphs are pulled from (and inserted into) `cache` so repeated layout of
/// the same font/size combination never re-rasterizes.
///
/// Coordinate system:
/// - The text origin (position.y) is at the BASELINE
/// - glyph.y is the offset from baseline to glyph top (negative = above baseline)
/// - The rendering code subtracts glyph.y from position.y to place the glyph correctly
pub(super) fn layout_text(
    font: &Font,
    font_id: u32,
    cache: &mut GlyphCache,
    text: &str,
    font_size: f32,
) -> Result<TextLayout, FontError> {
    let line_metrics = font.horizontal_line_metrics(font_size).unwrap_or_else(|| fontdue::LineMetrics {
        ascent: font_size * 0.8,
        descent: font_size * -0.2,
        line_gap: 0.0,
        new_line_size: font_size * 1.2,
    });

    let mut glyphs = Vec::new();
    let mut cursor_x = 0.0f32;
    let mut max_descent = 0.0f32;

    for character in text.chars() {
        // Handle special characters
        if character == '\n' {
            // Newlines not fully supported yet, just skip
            continue;
        }

        // Use the glyph cache, which rasterizes on miss (including spaces)
        let glyph_info = cache.get_or_rasterize(font, font_id, character, font_size)?;

        // Skip rendering for zero-width glyphs but still advance cursor
        let advance = glyph_info.rasterized.advance;

        if character != ' ' && glyph_info.rasterized.width > 0 {
            // glyph.y is the offset from baseline to glyph top
            // offset_y (ymin) from fontdue is already this: negative = above baseline
            let glyph_y = glyph_info.rasterized.offset_y;

            // Track max descent to calculate total text height
            // Descent is how far below baseline the glyph extends
            let glyph_bottom_from_baseline = glyph_y + glyph_info.rasterized.height as f32;
            if glyph_bottom_from_baseline > max_descent {
                max_descent = glyph_bottom_from_baseline;
            }

            glyphs.push(LayoutGlyph {
                character,
                x: cursor_x + glyph_info.rasterized.offset_x,
                y: glyph_y,  // Offset from baseline (negative = above baseline)
                info: glyph_info.clone(),
            });
        }

        cursor_x += advance;
    }

    // Text height is from top of highest ascender to bottom of lowest descender
    // ascent = distance from baseline to top of line
    // max_descent = distance from baseline to bottom of lowest glyph
    let text_height = line_metrics.ascent + max_descent.max(-line_metrics.descent);

    Ok(TextLayout {
        width: cursor_x,
        height: text_height.max(line_metrics.new_line_size),
        glyphs,
    })
}

/// Measure the size of a text string without rasterizing.
///
/// Uses `font.metrics()` instead of `font.rasterize()` to get advance widths
/// without the expensive bitmap generation step.
pub(super) fn measure_text(font: &Font, text: &str, font_size: f32) -> Vec2 {
    let mut width = 0.0f32;
    let line_metrics = font.horizontal_line_metrics(font_size);
    let height = line_metrics.map(|m| m.new_line_size).unwrap_or(font_size * 1.2);

    for character in text.chars() {
        let metrics = font.metrics(character, font_size);
        width += metrics.advance_width;
    }

    Vec2::new(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_layout() {
        let layout = TextLayout {
            width: 100.0,
            height: 16.0,
            glyphs: vec![],
        };
        assert_eq!(layout.width, 100.0);
        assert_eq!(layout.height, 16.0);
    }
}
