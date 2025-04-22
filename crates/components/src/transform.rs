use glam::Vec2;

#[derive(Clone, Copy)]
pub struct SpatialTransform {
    pub position:  Vec2,
    pub rotation:  f32,
    pub scale:     Vec2,
}
