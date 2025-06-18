use hecs::World;
use components::renderable::RenderableSprite;

/// Data for stepping through an N-frame animation.
pub struct SpriteAnimator {
    pub frames: usize,
    pub current: usize,
    pub frame_time: f32,
    pub accumulator: f32,
}

impl SpriteAnimator {
    pub fn new(frames: usize, frame_time: f32) -> Self {
        Self { frames, current: 0, frame_time, accumulator: 0.0 }
    }
}

/// System: advance each animator and update its RenderableSprite.frame.
pub fn sprite_animator_system(world: &mut World, delta: f32) {
    for (_ent, (anim, sprite)) in
        &mut world.query::<(&mut SpriteAnimator, &mut RenderableSprite)>()
    {
        anim.accumulator += delta;
        while anim.accumulator >= anim.frame_time {
            anim.accumulator -= anim.frame_time;
            anim.current = (anim.current + 1) % anim.frames;
            sprite.frame = anim.current;
        }
    }
}
