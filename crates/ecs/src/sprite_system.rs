//! Sprite animation system for ECS integration

use crate::{
    World, System,
    sprite_components::SpriteAnimation,
};

/// System that updates sprite animations
pub struct SpriteAnimationSystem;

impl System for SpriteAnimationSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        // Update all sprite animations
        for entity_id in world.entities() {
            if let Some(animation) = world.get_mut::<SpriteAnimation>(entity_id) {
                animation.update(delta_time);
            }
        }
    }

    fn name(&self) -> &str {
        "SpriteAnimationSystem"
    }
}
