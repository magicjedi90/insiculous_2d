//! Glyph texture cache for UI text rendering.
//!
//! Scans UI draw commands for text glyphs and creates one GPU texture per
//! unique glyph bitmap, caching handles across frames so each glyph is only
//! uploaded once.

use std::collections::HashMap;

use renderer::texture::TextureHandle;
use ui::{DrawCommand, GlyphDrawData};

use crate::assets::AssetManager;
use crate::contexts::GlyphCacheKey;

/// Caches one texture per unique glyph so text rendering reuses GPU
/// textures across frames.
///
/// Cache keys are color-agnostic: glyph textures are grayscale alpha masks
/// and the color is applied at render time (see [`GlyphCacheKey`]).
#[derive(Default)]
pub struct GlyphTextureCache {
    textures: HashMap<GlyphCacheKey, TextureHandle>,
}

impl GlyphTextureCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached glyph textures, keyed for lookup during UI rendering.
    pub fn textures(&self) -> &HashMap<GlyphCacheKey, TextureHandle> {
        &self.textures
    }

    /// Create textures for any glyphs in `commands` that are not cached yet.
    ///
    /// Called once per frame before rendering. Glyphs already in the cache
    /// (including duplicates within the same command list) are skipped.
    pub fn prepare(&mut self, commands: &[DrawCommand], assets: &mut AssetManager) {
        let missing = self.uncached_glyphs(commands);
        for (key, glyph) in missing {
            // Re-check: the same glyph can appear more than once per frame,
            // and the first occurrence has already created the texture.
            if self.textures.contains_key(&key) {
                continue;
            }

            // Create glyph texture (grayscale alpha mask)
            match assets.create_glyph_texture(glyph.width, glyph.height, &glyph.bitmap) {
                Ok(handle) => {
                    self.textures.insert(key, handle);
                }
                Err(e) => {
                    log::warn!("Failed to create glyph texture for '{}': {}", glyph.character, e);
                }
            }
        }
    }

    /// Collect glyphs from `commands` that have no cached texture yet,
    /// in command order. Duplicates are not removed here; `prepare` skips
    /// them once the first occurrence has been created.
    fn uncached_glyphs<'a>(
        &self,
        commands: &'a [DrawCommand],
    ) -> Vec<(GlyphCacheKey, &'a GlyphDrawData)> {
        Self::renderable_glyphs(commands)
            .filter(|(key, _)| !self.textures.contains_key(key))
            .collect()
    }

    /// Iterate all glyphs in Text commands that need a texture to render
    /// (skips empty glyphs such as spaces).
    fn renderable_glyphs(
        commands: &[DrawCommand],
    ) -> impl Iterator<Item = (GlyphCacheKey, &GlyphDrawData)> {
        commands
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Text { data, .. } => Some(data.glyphs.iter()),
                _ => None,
            })
            .flatten()
            .filter(|glyph| glyph.width > 0 && glyph.height > 0 && !glyph.bitmap.is_empty())
            .map(|glyph| {
                (
                    GlyphCacheKey::new(glyph.character, glyph.width, glyph.height),
                    glyph,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{Color, Rect};
    use glam::Vec2;
    use std::sync::Arc;
    use ui::TextDrawData;

    fn glyph(character: char, width: u32, height: u32, bitmap: &[u8]) -> GlyphDrawData {
        GlyphDrawData {
            bitmap: Arc::from(bitmap),
            width,
            height,
            x: 0.0,
            y: 0.0,
            character,
        }
    }

    fn text_command(glyphs: Vec<GlyphDrawData>) -> DrawCommand {
        DrawCommand::Text {
            data: TextDrawData {
                text: String::new(),
                position: Vec2::ZERO,
                color: Color::new(1.0, 1.0, 1.0, 1.0),
                font_size: 14.0,
                width: 0.0,
                height: 0.0,
                glyphs,
            },
            depth: 0.0,
        }
    }

    #[test]
    fn fresh_cache_starts_empty() {
        let cache = GlyphTextureCache::new();
        assert!(cache.textures().is_empty());
    }

    #[test]
    fn empty_glyphs_and_non_text_commands_need_no_textures() {
        let cache = GlyphTextureCache::new();
        let commands = vec![
            DrawCommand::Rect {
                bounds: Rect::new(0.0, 0.0, 10.0, 10.0),
                color: Color::new(1.0, 1.0, 1.0, 1.0),
                corner_radius: 0.0,
                depth: 0.0,
            },
            text_command(vec![
                glyph('a', 4, 4, &[255; 16]), // renderable
                glyph(' ', 0, 0, &[]),        // space: zero size, no bitmap
                glyph('b', 4, 4, &[]),        // empty bitmap
            ]),
        ];

        let missing = cache.uncached_glyphs(&commands);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].1.character, 'a');
    }

    #[test]
    fn cached_glyphs_are_not_reported_missing() {
        let mut cache = GlyphTextureCache::new();
        let commands = vec![text_command(vec![
            glyph('a', 4, 4, &[255; 16]),
            glyph('b', 8, 8, &[255; 64]),
        ])];

        // Simulate a previously created texture for 'a'.
        cache
            .textures
            .insert(GlyphCacheKey::new('a', 4, 4), TextureHandle { id: 7 });

        let missing = cache.uncached_glyphs(&commands);
        assert_eq!(missing.len(), 1, "only the uncached glyph should be missing");
        assert_eq!(missing[0].1.character, 'b');
        assert_eq!(cache.textures().len(), 1);
    }

    #[test]
    fn same_glyph_at_different_sizes_needs_separate_textures() {
        let cache = GlyphTextureCache::new();
        let commands = vec![text_command(vec![
            glyph('a', 4, 4, &[255; 16]),
            glyph('a', 8, 8, &[255; 64]),
        ])];

        let missing = cache.uncached_glyphs(&commands);
        assert_eq!(missing.len(), 2, "each rasterized size is a distinct cache entry");
    }
}
