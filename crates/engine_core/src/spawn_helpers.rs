//! Shared entity recipes used by every arcade game.
//!
//! Games spawn their own gameplay entities (paddles, ships, bricks differ
//! per game), but some scenery is identical everywhere — the engine owns
//! those recipes so the look stays consistent and tuning happens once.

use ecs::sprite_components::{Name, Sprite, Transform2D};
use ecs::{EntityId, World};
use glam::{Vec2, Vec4};

use crate::RENDER_UNIT;

/// Spawn the full-window backdrop sprite: a flat tint at depth -100,
/// oversized 20% so window resizes don't reveal a seam. Non-emissive on
/// purpose — the grid lines and gameplay sprites pop against it.
///
/// `window_size` is the window's pixel size (`WIN_W`, `WIN_H`).
pub fn spawn_background(world: &mut World, tex: u32, color: Vec4, window_size: Vec2) -> EntityId {
    let size = window_size * 1.2;
    world
        .spawn()
        .with(Name::new("Background"))
        .with(Transform2D::from_parts(Vec2::ZERO, 0.0, size / RENDER_UNIT))
        .with(Sprite::new(tex).with_color(color).with_depth(-100.0))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_covers_window_with_overscan_behind_everything() {
        let mut world = World::new();
        let e = spawn_background(&mut world, 0, Vec4::ONE, Vec2::new(800.0, 600.0));

        let t = world.get::<Transform2D>(e).expect("background has a transform");
        let px = t.scale * RENDER_UNIT;
        assert!((px - Vec2::new(960.0, 720.0)).length() < 0.01, "20% overscan, got {px:?}");

        let s = world.get::<Sprite>(e).expect("background has a sprite");
        assert_eq!(s.depth, -100.0, "background must render behind gameplay sprites");
        assert_eq!(s.emissive, 0.0, "background must not bloom");
    }
}
