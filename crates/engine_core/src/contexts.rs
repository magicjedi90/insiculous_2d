//! Core contexts for the Game API.
//!
//! This module provides the main context structures used by the Game trait
//! to give access to engine systems during the game loop.

use glam::Vec2;
use ecs::World;
use input::InputHandler;
use audio::AudioManager;
use ui::{UIContext, Color as UIColor};
use renderer::{sprite::SpriteBatcher, Camera2D, texture::TextureHandle};
use std::collections::HashMap;
use crate::assets::AssetManager;

/// Key for caching glyph textures
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    /// Character being rendered
    character: char,
    /// Width of the glyph bitmap
    width: u32,
    /// Height of the glyph bitmap
    height: u32,
    /// Color as RGB (u8 each)
    color_rgb: [u8; 3],
}

impl GlyphCacheKey {
    pub(crate) fn new(character: char, width: u32, height: u32, color: &UIColor) -> Self {
        Self {
            character,
            width,
            height,
            color_rgb: [
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ],
        }
    }
}

/// Context passed to game methods, providing access to engine systems.
pub struct GameContext<'a> {
    /// Input handler for keyboard, mouse, and gamepad
    pub input: &'a InputHandler,
    /// The ECS world for entity/component management
    pub world: &'a mut World,
    /// Asset manager for loading textures and other resources
    pub assets: &'a mut AssetManager,
    /// Audio manager for sound playback
    pub audio: &'a mut AudioManager,
    /// UI context for immediate-mode UI
    pub ui: &'a mut UIContext,
    /// Delta time since last frame in seconds
    pub delta_time: f32,
    /// Current window size
    pub window_size: Vec2,
}

/// Render context passed to the render method.
pub struct RenderContext<'a> {
    /// The ECS world (read-only during render)
    pub world: &'a World,
    /// Sprite batcher for adding sprites to render
    pub sprites: &'a mut SpriteBatcher,
    /// The 2D camera
    pub camera: &'a mut Camera2D,
    /// Current window size
    pub window_size: Vec2,
    /// UI draw commands to render
    pub ui_commands: &'a [ui::DrawCommand],
    /// Cached glyph textures for text rendering
    pub glyph_textures: &'a HashMap<GlyphCacheKey, TextureHandle>,
}