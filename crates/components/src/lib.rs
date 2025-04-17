pub use hecs::{World, Entity};        
use glam::Vec2;

#[derive(Clone, Copy)]
pub struct SpatialTransform {
    pub position:  Vec2,
    pub rotation:  f32,
    pub scale:     Vec2,
}

#[derive(Clone, Copy)]
pub struct RenderableSprite {
    pub texture_identifier: u32,
    pub texture_coordinates: [f32; 4],
}
