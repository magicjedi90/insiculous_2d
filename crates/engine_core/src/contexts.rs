//! Core contexts for the Game API.
//!
//! This module provides the main context structures used by the Game trait
//! to give access to engine systems during the game loop.

use glam::Vec2;
use ecs::World;
use input::{InputHandler, InputSettings};
use audio::AudioManager;
use ui::UIContext;
use renderer::{line_pipeline::LineVertex, sprite::SpriteBatcher, Camera, texture::TextureHandle};
use std::collections::HashMap;
use crate::assets::AssetManager;
use crate::chaos_mode::ChaosMode;
use crate::achievements::AchievementManager;
use crate::particles::ParticleManager;

/// Key for caching glyph textures.
///
/// Note: Color is NOT included in the cache key because glyph textures are
/// grayscale alpha masks. The color is applied at render time by multiplying
/// the sprite color with the texture, allowing the same glyph texture to be
/// reused for any color.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    /// Character being rendered
    character: char,
    /// Width of the glyph bitmap
    width: u32,
    /// Height of the glyph bitmap
    height: u32,
}

impl GlyphCacheKey {
    pub(crate) fn new(character: char, width: u32, height: u32) -> Self {
        Self {
            character,
            width,
            height,
        }
    }
}

/// Context passed to game methods, providing access to engine systems.
pub struct GameContext<'a> {
    /// Input handler for keyboard, mouse, and gamepad
    pub input: &'a InputHandler,
    /// Player-aware input bindings: the universal per-player mapping layer.
    /// Query with `ctx.players.is_active(PlayerId::P1, GameAction::Action1,
    /// ctx.input)` or `ctx.players.move_x(PlayerId::P2, ctx.input)`.
    /// Mutable so games can re-point pads at runtime (`assign_pad`); loaded
    /// from `GameConfig::input_settings_path` when set.
    pub players: &'a mut InputSettings,
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
    /// Project-wide gameplay intensity theme. Seeded from `GameConfig` and
    /// **read-write**: assign to it when the player picks a mode at runtime
    /// and the engine persists the change, so `ctx.chaos_mode` is always the
    /// current selection on later frames (no stale startup value).
    pub chaos_mode: ChaosMode,
    /// Achievement / trophy manager. Register achievements in `init()`, then
    /// call `ctx.achievements.unlock("id")` from gameplay code.
    pub achievements: &'a mut AchievementManager,
    /// Particle system. Spawn bursts directly with
    /// `ctx.particles.spawn_burst(pos, &config)`, or attach a
    /// [`ParticleEmitter`](crate::particles::ParticleEmitter) component to
    /// any entity with a `Transform2D` for continuous emission.
    pub particles: &'a mut ParticleManager,
    /// Line-list vertex buffer for the line render pipeline. Pairs of
    /// vertices form line segments. Cleared each frame before `update()`.
    /// Typical use: step a [`GridMesh`](crate::grid::GridMesh) and append
    /// its `build_line_vertices()` output here, or push debug-draw segments.
    pub lines: &'a mut Vec<LineVertex>,
}

/// Render context passed to the render method.
pub struct RenderContext<'a> {
    /// The ECS world (read-only during render)
    pub world: &'a World,
    /// Sprite batcher for adding sprites to render
    pub sprites: &'a mut SpriteBatcher,
    /// The 2D camera
    pub camera: &'a mut Camera,
    /// Current window size
    pub window_size: Vec2,
    /// UI draw commands to render
    pub ui_commands: &'a [ui::DrawCommand],
    /// Cached glyph textures for text rendering
    pub glyph_textures: &'a HashMap<GlyphCacheKey, TextureHandle>,
}