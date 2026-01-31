//! Font loading and text rendering with fontdue.
//!
//! This module provides font loading, glyph rasterization, and text measurement
//! using the fontdue library for CPU-side font rendering.

use std::collections::HashMap;
use fontdue::{Font, FontSettings};
use glam::Vec2;

/// Error type for font operations.
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Failed to load font: {0}")]
    LoadError(String),
    #[error("Font not found: {0}")]
    NotFound(String),
    #[error("Failed to rasterize glyph: {0}")]
    RasterizeError(String),
}

/// A handle to a loaded font.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct FontHandle {
    pub id: u32,
}

/// Font measurement information for layout calculations.
///
/// These metrics are essential for proper text positioning:
/// - `ascent`: Use to know how much space text needs above the baseline
/// - `descent`: Use to know how much space text needs below the baseline
/// - `line_height`: Use for spacing between lines of text
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// Distance from baseline to top of tallest glyph (positive value)
    pub ascent: f32,
    /// Distance from baseline to bottom of descenders (negative value)
    pub descent: f32,
    /// Recommended distance between baselines for line spacing
    pub line_height: f32,
}


/// Rasterized glyph data ready for rendering.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Glyph bitmap data (grayscale, one byte per pixel)
    pub bitmap: Vec<u8>,
    /// Width of the bitmap in pixels
    pub width: u32,
    /// Height of the bitmap in pixels
    pub height: u32,
    /// Horizontal offset from cursor position to glyph origin
    pub offset_x: f32,
    /// Vertical offset from baseline to top of glyph
    pub offset_y: f32,
    /// How much to advance the cursor after this glyph
    pub advance: f32,
}

/// Cached glyph information for a specific font size.
#[derive(Debug, Clone)]
pub struct GlyphInfo {
    /// Rasterized bitmap
    pub rasterized: RasterizedGlyph,
    /// Character this glyph represents
    pub character: char,
    /// Font size this was rasterized at
    pub font_size: f32,
}

/// Key for glyph cache lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font_id: u32,
    character: char,
    /// Font size in tenths of a pixel (to allow float sizes with integer key)
    size_tenths: u32,
}

impl GlyphKey {
    fn new(font_id: u32, character: char, font_size: f32) -> Self {
        Self {
            font_id,
            character,
            size_tenths: (font_size * 10.0) as u32,
        }
    }
}

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

/// Font manager for loading fonts and rasterizing glyphs.
pub struct FontManager {
    /// Loaded fonts by handle
    fonts: HashMap<u32, Font>,
    /// Glyph cache
    glyph_cache: HashMap<GlyphKey, GlyphInfo>,
    /// Next font handle ID
    next_id: u32,
    /// Default font handle (if loaded)
    default_font: Option<FontHandle>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    /// Create a new font manager.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            glyph_cache: HashMap::new(),
            next_id: 1,
            default_font: None,
        }
    }

    /// Load a font from file bytes.
    pub fn load_font(&mut self, font_data: &[u8]) -> Result<FontHandle, FontError> {
        let font = Font::from_bytes(font_data, FontSettings::default())
            .map_err(|e| FontError::LoadError(e.to_string()))?;

        let handle = FontHandle { id: self.next_id };
        self.fonts.insert(self.next_id, font);
        self.next_id += 1;

        // Set as default if this is the first font
        if self.default_font.is_none() {
            self.default_font = Some(handle);
        }

        log::info!("Loaded font with handle {}", handle.id);
        Ok(handle)
    }

    /// Load a font from a file path.
    pub fn load_font_file(&mut self, path: &str) -> Result<FontHandle, FontError> {
        let font_data = std::fs::read(path)
            .map_err(|e| FontError::LoadError(format!("Failed to read file {}: {}", path, e)))?;
        self.load_font(&font_data)
    }

    /// Load the embedded default font (a simple built-in font for fallback).
    /// This uses a subset of the Roboto font embedded in the binary.
    pub fn load_default_font(&mut self) -> Result<FontHandle, FontError> {
        // Use a simple embedded font - we'll embed a minimal font
        // For now, return an error if no font data is available
        Err(FontError::LoadError("No embedded font available. Load a TTF/OTF file.".to_string()))
    }

    /// Get the default font handle.
    pub fn default_font(&self) -> Option<FontHandle> {
        self.default_font
    }

    /// Set the default font.
    pub fn set_default_font(&mut self, handle: FontHandle) {
        self.default_font = Some(handle);
    }

    /// Get a font by handle.
    pub fn get_font(&self, handle: FontHandle) -> Option<&Font> {
        self.fonts.get(&handle.id)
    }

    /// Get font metrics for layout calculations.
    ///
    /// Returns None if the font handle is invalid or metrics unavailable.
    pub fn metrics(&self, handle: FontHandle, font_size: f32) -> Option<FontMetrics> {
        let font = self.fonts.get(&handle.id)?;
        let line_metrics = font.horizontal_line_metrics(font_size)?;

        Some(FontMetrics {
            ascent: line_metrics.ascent,
            descent: line_metrics.descent,
            line_height: line_metrics.ascent - line_metrics.descent + line_metrics.line_gap,
        })
    }

    /// Rasterize a single glyph at a specific size.
    pub fn rasterize_glyph(
        &mut self,
        handle: FontHandle,
        character: char,
        font_size: f32,
    ) -> Result<&GlyphInfo, FontError> {
        let key = GlyphKey::new(handle.id, character, font_size);

        // Check cache first
        if self.glyph_cache.contains_key(&key) {
            return Ok(self.glyph_cache.get(&key).unwrap());
        }

        // Rasterize the glyph
        let font = self.fonts.get(&handle.id)
            .ok_or_else(|| FontError::NotFound(format!("Font {} not found", handle.id)))?;

        let (metrics, bitmap) = font.rasterize(character, font_size);

        let glyph_info = GlyphInfo {
            rasterized: RasterizedGlyph {
                bitmap,
                width: metrics.width as u32,
                height: metrics.height as u32,
                offset_x: metrics.xmin as f32,
                // Convert from fontdue coords (ymin = bottom of glyph, +Y = up)
                // to UI coords (offset_y = top of glyph relative to baseline, +Y = down)
                // Top in fontdue = ymin + height, flip sign for UI = -(ymin + height)
                offset_y: -(metrics.ymin as f32 + metrics.height as f32),
                advance: metrics.advance_width,
            },
            character,
            font_size,
        };

        self.glyph_cache.insert(key, glyph_info);
        Ok(self.glyph_cache.get(&key).unwrap())
    }

    /// Layout a string of text, returning positions and glyph info for each character.
    ///
    /// This method uses the internal glyph cache to avoid re-rasterizing glyphs that have
    /// already been rendered at the same font/size combination. The CPU bitmap cache here
    /// works in tandem with engine_core's GPU texture cache - this caches the rasterized
    /// bitmaps, while engine_core caches the GPU textures created from those bitmaps.
    ///
    /// Coordinate system:
    /// - The text origin (position.y) is at the BASELINE
    /// - glyph.y is the offset from baseline to glyph top (negative = above baseline)
    /// - The rendering code subtracts glyph.y from position.y to place the glyph correctly
    pub fn layout_text(
        &mut self,
        handle: FontHandle,
        text: &str,
        font_size: f32,
    ) -> Result<TextLayout, FontError> {
        // Verify font exists first
        if !self.fonts.contains_key(&handle.id) {
            return Err(FontError::NotFound(format!("Font {} not found", handle.id)));
        }

        // Get line metrics (need font reference)
        let line_metrics = {
            let font = self.fonts.get(&handle.id).unwrap();
            font.horizontal_line_metrics(font_size).unwrap_or_else(|| fontdue::LineMetrics {
                ascent: font_size * 0.8,
                descent: font_size * -0.2,
                line_gap: 0.0,
                new_line_size: font_size * 1.2,
            })
        };

        let mut glyphs = Vec::new();
        let mut cursor_x = 0.0f32;
        let mut max_descent = 0.0f32;

        for character in text.chars() {
            // Handle special characters
            if character == '\n' {
                // Newlines not fully supported yet, just skip
                continue;
            }

            // Use rasterize_glyph which properly caches (including spaces)
            let glyph_info = self.rasterize_glyph(handle, character, font_size)?;

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

    /// Measure the size of a text string without fully rasterizing.
    pub fn measure_text(
        &self,
        handle: FontHandle,
        text: &str,
        font_size: f32,
    ) -> Result<Vec2, FontError> {
        let font = self.fonts.get(&handle.id)
            .ok_or_else(|| FontError::NotFound(format!("Font {} not found", handle.id)))?;

        let mut width = 0.0f32;
        let line_metrics = font.horizontal_line_metrics(font_size);
        let height = line_metrics.map(|m| m.new_line_size).unwrap_or(font_size * 1.2);

        for character in text.chars() {
            let (metrics, _) = font.rasterize(character, font_size);
            width += metrics.advance_width;
        }

        Ok(Vec2::new(width, height))
    }

    /// Clear the glyph cache to free memory.
    pub fn clear_cache(&mut self) {
        self.glyph_cache.clear();
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.fonts.len(), self.glyph_cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_handle_default() {
        let handle = FontHandle::default();
        assert_eq!(handle.id, 0);
    }

    #[test]
    fn test_font_manager_new() {
        let manager = FontManager::new();
        assert!(manager.default_font().is_none());
        let (fonts, glyphs) = manager.cache_stats();
        assert_eq!(fonts, 0);
        assert_eq!(glyphs, 0);
    }

    #[test]
    fn test_glyph_key() {
        let key1 = GlyphKey::new(1, 'A', 16.0);
        let key2 = GlyphKey::new(1, 'A', 16.0);
        let key3 = GlyphKey::new(1, 'A', 18.0);
        let key4 = GlyphKey::new(1, 'B', 16.0);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_rasterized_glyph() {
        let glyph = RasterizedGlyph {
            bitmap: vec![255; 16],
            width: 4,
            height: 4,
            offset_x: 0.0,
            offset_y: -3.0,
            advance: 5.0,
        };
        assert_eq!(glyph.bitmap.len(), 16);
        assert_eq!(glyph.width, 4);
        assert_eq!(glyph.height, 4);
    }

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

    #[test]
    fn test_font_metrics_struct() {
        let metrics = FontMetrics {
            ascent: 14.0,
            descent: -4.0,
            line_height: 20.0,
        };
        assert_eq!(metrics.ascent, 14.0);
        assert_eq!(metrics.descent, -4.0);
        assert_eq!(metrics.line_height, 20.0);
    }

    #[test]
    fn test_font_manager_metrics_no_font() {
        let manager = FontManager::new();
        let handle = FontHandle { id: 999 };
        assert!(manager.metrics(handle, 16.0).is_none());
    }
}
