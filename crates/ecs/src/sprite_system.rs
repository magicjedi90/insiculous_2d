//! Sprite animation system for ECS integration

use crate::{
    sprite_components::{Sprite, SpriteAnimation},
    System, World,
};

/// Advances every [`SpriteAnimation`] and writes the resulting cell region
/// onto the entity's [`Sprite`].
///
/// This is the link that makes animation visible: without it a component can
/// hold clips but nothing ever reaches `Sprite.tex_region`. Entities that have
/// an animation but no sprite are skipped, and an animation whose current
/// frame does not resolve (nothing playing, empty clip, index past the grid)
/// leaves the sprite's region untouched.
pub struct SpriteAnimationSystem;

impl System for SpriteAnimationSystem {
    fn update(&mut self, world: &mut World, delta_time: f32) {
        for entity_id in world.entities() {
            // Advance first, then hand the resolved region to the sprite in a
            // second lookup — two components on one entity cannot be borrowed
            // mutably at once.
            let region = match world.get_mut::<SpriteAnimation>(entity_id) {
                Some(animation) => {
                    animation.update(delta_time);
                    animation.current_uv()
                }
                None => continue,
            };

            if let (Some(region), Some(sprite)) = (region, world.get_mut::<Sprite>(entity_id)) {
                sprite.tex_region = region;
            }
        }
    }

    fn name(&self) -> &str {
        "SpriteAnimationSystem"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprite_components::{AnimationClip, SheetGrid};

    /// A 4x1 sheet with a two-frame clip selected and playing.
    fn animated_entity(world: &mut World) -> crate::EntityId {
        let entity = world.create_entity();
        let mut animation = SpriteAnimation::new(SheetGrid::new(4, 1))
            .with_clip("walk", AnimationClip::new(vec![0, 1], 10.0));
        animation.play("walk");
        world.add_component(&entity, animation).unwrap();
        world.add_component(&entity, Sprite::new(0)).unwrap();
        entity
    }

    #[test]
    fn test_system_writes_current_frame_region_to_sprite() {
        let mut world = World::new();
        let entity = animated_entity(&mut world);

        SpriteAnimationSystem.update(&mut world, 0.0);
        assert_eq!(world.get::<Sprite>(entity).unwrap().tex_region, [0.0, 0.0, 0.25, 1.0]);

        // One full frame at 10 fps advances to cell 1.
        SpriteAnimationSystem.update(&mut world, 0.1);
        assert_eq!(world.get::<Sprite>(entity).unwrap().tex_region, [0.25, 0.0, 0.25, 1.0]);
    }

    #[test]
    fn test_system_skips_animation_without_sprite() {
        let mut world = World::new();
        let entity = world.create_entity();
        let mut animation = SpriteAnimation::new(SheetGrid::new(2, 1))
            .with_clip("walk", AnimationClip::new(vec![0, 1], 10.0));
        animation.play("walk");
        world.add_component(&entity, animation).unwrap();

        // No Sprite on the entity — must advance the animation, not panic.
        SpriteAnimationSystem.update(&mut world, 0.1);
        assert_eq!(world.get::<SpriteAnimation>(entity).unwrap().current_frame, 1);
    }

    #[test]
    fn test_system_with_zero_delta_freezes_the_frame() {
        let mut world = World::new();
        let entity = animated_entity(&mut world);

        // dt 0 is how a paused game reaches the system (time_scale 0.0).
        for _ in 0..100 {
            SpriteAnimationSystem.update(&mut world, 0.0);
        }
        assert_eq!(world.get::<SpriteAnimation>(entity).unwrap().current_frame, 0);
        assert_eq!(world.get::<Sprite>(entity).unwrap().tex_region, [0.0, 0.0, 0.25, 1.0]);
    }

    #[test]
    fn test_system_leaves_sprite_region_untouched_when_nothing_resolves() {
        let mut world = World::new();
        let entity = world.create_entity();
        // A frame index past the end of a 2-cell grid never resolves.
        let mut animation = SpriteAnimation::new(SheetGrid::new(2, 1))
            .with_clip("bad", AnimationClip::new(vec![99], 10.0));
        animation.play("bad");
        world.add_component(&entity, animation).unwrap();
        let sprite = Sprite::new(0).with_tex_region(0.5, 0.5, 0.5, 0.5);
        world.add_component(&entity, sprite).unwrap();

        SpriteAnimationSystem.update(&mut world, 0.1);
        assert_eq!(world.get::<Sprite>(entity).unwrap().tex_region, [0.5, 0.5, 0.5, 0.5]);
    }
}
