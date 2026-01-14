//! UI rendering integration - bridges UI draw commands to renderer sprites.
//!
//! This module provides the integration layer between the UI system and the renderer,
//! converting UI draw commands (rectangles, text, circles, lines) into sprites that
//! can be rendered by the sprite batching system.

use glam::Vec2;
use renderer::{
    sprite::{SpriteBatcher, Sprite},
    texture::TextureHandle,
};
use ui::{Color as UIColor, DrawCommand, Rect};
use std::collections::HashMap;
use crate::contexts::GlyphCacheKey;

/// Renders UI draw commands as sprites in the sprite batcher.
/// 
/// This function converts UI draw commands into renderer sprites, handling:
/// - Rectangle rendering with proper coordinate transformation
/// - Rectangle borders as 4 separate rectangles
/// - Text rendering using cached glyph textures
/// - Text placeholders for debugging
/// - Circle rendering (as squares until circle shader is available)
/// - Line rendering as rotated rectangles
/// 
/// Coordinate transformation: UI uses screen coordinates (0,0 = top-left) while
/// the renderer uses world coordinates (0,0 = center of screen).
pub fn render_ui_commands(
    sprites: &mut SpriteBatcher,
    commands: &[DrawCommand],
    window_size: Vec2,
    glyph_textures: &HashMap<GlyphCacheKey, TextureHandle>,
) {
    let white_texture = TextureHandle { id: 0 };

    for cmd in commands {
        match cmd {
            DrawCommand::Rect { bounds, color, depth, .. } => {
                // Convert screen coordinates (0,0 = top-left) to world coordinates (0,0 = center)
                let center_x = bounds.x + bounds.width / 2.0 - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - (bounds.y + bounds.height / 2.0);

                let sprite = Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(bounds.width, bounds.height))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::RectBorder { bounds, color, width, depth, .. } => {
                // Render border as 4 thin rectangles
                let half_width = *width / 2.0;

                // Top edge
                let top = Rect::new(bounds.x - half_width, bounds.y - half_width, bounds.width + *width, *width);
                render_ui_rect(sprites, &top, color, *depth, window_size);

                // Bottom edge
                let bottom = Rect::new(bounds.x - half_width, bounds.y + bounds.height - half_width, bounds.width + *width, *width);
                render_ui_rect(sprites, &bottom, color, *depth, window_size);

                // Left edge
                let left = Rect::new(bounds.x - half_width, bounds.y + half_width, *width, bounds.height - *width);
                render_ui_rect(sprites, &left, color, *depth, window_size);

                // Right edge
                let right = Rect::new(bounds.x + bounds.width - half_width, bounds.y + half_width, *width, bounds.height - *width);
                render_ui_rect(sprites, &right, color, *depth, window_size);
            }
            DrawCommand::Text { data, depth } => {
                // Render text with rasterized glyph data
                if data.glyphs.is_empty() {
                    // No glyphs - render as placeholder rectangle
                    let center_x = data.position.x + data.width / 2.0 - window_size.x / 2.0;
                    let center_y = window_size.y / 2.0 - (data.position.y + data.height / 2.0);

                    let sprite = Sprite::new(white_texture)
                        .with_position(Vec2::new(center_x, center_y))
                        .with_scale(Vec2::new(data.width.max(data.font_size * 4.0), data.height.max(data.font_size)))
                        .with_color(glam::Vec4::new(data.color.r, data.color.g, data.color.b, data.color.a * 0.3))
                        .with_depth(*depth);

                    sprites.add_sprite(&sprite);
                } else {
                    // Render each glyph using cached glyph textures
                    for glyph in &data.glyphs {
                        // Skip glyphs with no bitmap (spaces, etc.)
                        if glyph.width == 0 || glyph.height == 0 {
                            continue;
                        }

                        // Calculate glyph position in world coordinates
                        // glyph.x and glyph.y are offsets from the text origin
                        let glyph_x = data.position.x + glyph.x + glyph.width as f32 / 2.0 - window_size.x / 2.0;
                        let glyph_y = window_size.y / 2.0 - (data.position.y + glyph.y + glyph.height as f32 / 2.0);

                        // Look up glyph texture in cache (color-agnostic)
                        let glyph_key = GlyphCacheKey::new(
                            glyph.character,
                            glyph.width,
                            glyph.height,
                        );

                        let texture = match glyph_textures.get(&glyph_key) {
                            Some(&tex) => tex,
                            None => {
                                log::warn!(
                                    "Missing glyph texture for '{}' ({}x{}). Using white fallback. \
                                     This may indicate the glyph wasn't pre-cached.",
                                    glyph.character, glyph.width, glyph.height
                                );
                                white_texture
                            }
                        };

                        // Render glyph with text color - texture is grayscale alpha mask
                        let render_width = glyph.width as f32;
                        let render_height = glyph.height as f32;

                        let sprite = Sprite::new(texture)
                            .with_position(Vec2::new(glyph_x, glyph_y))
                            .with_scale(Vec2::new(render_width, render_height))
                            .with_color(glam::Vec4::new(data.color.r, data.color.g, data.color.b, data.color.a))
                            .with_depth(*depth);

                        sprites.add_sprite(&sprite);
                    }

                    // Debug output removed - font rendering is working
                }
            }
            DrawCommand::TextPlaceholder { text, position, color, font_size, depth } => {
                // Placeholder: render a small rectangle where text would be
                let estimated_width = text.len() as f32 * *font_size * 0.6;
                let center_x = position.x + estimated_width / 2.0 - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - (position.y + *font_size / 2.0);

                let sprite = Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(estimated_width, *font_size))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a * 0.3))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::Circle { center, radius, color, depth } => {
                // Render circle as a square (approximation until we have circle shader)
                let center_x = center.x - window_size.x / 2.0;
                let center_y = window_size.y / 2.0 - center.y;

                let sprite = Sprite::new(white_texture)
                    .with_position(Vec2::new(center_x, center_y))
                    .with_scale(Vec2::new(*radius * 2.0, *radius * 2.0))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
            DrawCommand::Line { start, end, color, width, depth } => {
                // Render line as a thin rotated rectangle
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let length = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                let mid_x = (start.x + end.x) / 2.0 - window_size.x / 2.0;
                let mid_y = window_size.y / 2.0 - (start.y + end.y) / 2.0;

                let sprite = Sprite::new(white_texture)
                    .with_position(Vec2::new(mid_x, mid_y))
                    .with_rotation(-angle) // Negate for coordinate system
                    .with_scale(Vec2::new(length, *width))
                    .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
                    .with_depth(*depth);

                sprites.add_sprite(&sprite);
            }
        }
    }
}

/// Helper to render a single UI rect as a sprite.
/// 
/// Converts UI rectangle bounds to sprite position and scale, handling the
/// coordinate transformation from UI coordinates (top-left origin) to renderer
/// coordinates (center origin).
fn render_ui_rect(sprites: &mut SpriteBatcher, bounds: &Rect, color: &UIColor, depth: f32, window_size: Vec2) {
    let white_texture = TextureHandle { id: 0 };
    let center_x = bounds.x + bounds.width / 2.0 - window_size.x / 2.0;
    let center_y = window_size.y / 2.0 - (bounds.y + bounds.height / 2.0);

    let sprite = Sprite::new(white_texture)
        .with_position(Vec2::new(center_x, center_y))
        .with_scale(Vec2::new(bounds.width, bounds.height))
        .with_color(glam::Vec4::new(color.r, color.g, color.b, color.a))
        .with_depth(depth);

    sprites.add_sprite(&sprite);
}