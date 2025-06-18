/// Represents a sprite that can be rendered.
pub struct RenderableSprite {
    pub texture_identifier: u32,
    pub texture_coordinates: [f32; 4],
    pub frame: usize
}
