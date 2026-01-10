//! Sprite rendering system for ECS integration

use glam::Vec2;
use renderer::{Sprite as RendererSprite, Camera2D as RendererCamera2D};
use crate::{
    World, System, EntityId,
    sprite_components::{Sprite, Transform2D, Camera2D, SpriteAnimation, SpriteRenderData},
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

/// System that collects sprite data for rendering
pub struct SpriteRenderSystem {
    render_data: SpriteRenderData,
}

impl SpriteRenderSystem {
    /// Create a new sprite render system
    pub fn new() -> Self {
        Self {
            render_data: SpriteRenderData::new(),
        }
    }

    /// Get the render data (call this after update)
    pub fn render_data(&self) -> &SpriteRenderData {
        &self.render_data
    }

    /// Get mutable render data
    pub fn render_data_mut(&mut self) -> &mut SpriteRenderData {
        &mut self.render_data
    }

    /// Find the main camera entity
    fn find_main_camera(&self, world: &World) -> Option<EntityId> {
        for entity_id in world.entities() {
            if let Some(camera) = world.get::<Camera2D>(entity_id) {
                if camera.is_main_camera {
                    return Some(entity_id);
                }
            }
        }
        None
    }

    /// Convert ECS sprite to renderer sprite data
    fn convert_sprite(
        &self,
        entity_transform: &Transform2D,
        sprite: &Sprite,
        _animation: Option<&SpriteAnimation>,
    ) -> RendererSprite {
        let world_position = entity_transform.position + entity_transform.transform_point(sprite.offset);
        let world_rotation = entity_transform.rotation + sprite.rotation;
        let world_scale = entity_transform.scale * sprite.scale;

        // Create renderer sprite using builder pattern
        RendererSprite::new(renderer::TextureHandle { id: sprite.texture_handle })
            .with_position(world_position)
            .with_rotation(world_rotation)
            .with_scale(world_scale * 80.0) // Default size
            .with_color(sprite.color)
            .with_depth(sprite.depth)
    }
}

impl Default for SpriteRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for SpriteRenderSystem {
    fn update(&mut self, world: &mut World, _delta_time: f32) {
        // Clear previous frame's data
        self.render_data.clear();

        // Find main camera
        if let Some(camera_entity) = self.find_main_camera(world) {
            if let Some(camera) = world.get::<Camera2D>(camera_entity) {
                let mut renderer_camera = RendererCamera2D::default();
                renderer_camera.position = camera.position;
                renderer_camera.rotation = camera.rotation;
                renderer_camera.zoom = camera.zoom;
                renderer_camera.viewport_size = camera.viewport_size;
                self.render_data.set_camera(renderer_camera);
            }
        }

        // Collect all sprites - need to gather entity IDs first to avoid borrow issues
        let entity_ids: Vec<EntityId> = world.entities();

        for entity_id in entity_ids {
            // Get transform (required)
            let transform = match world.get::<Transform2D>(entity_id) {
                Some(t) => t.clone(),
                None => continue,
            };

            // Get sprite (required)
            let sprite = match world.get::<Sprite>(entity_id) {
                Some(s) => s.clone(),
                None => continue,
            };

            // Get animation (optional)
            let animation = world.get::<SpriteAnimation>(entity_id).cloned();

            // Convert to renderer sprite
            let renderer_sprite = self.convert_sprite(&transform, &sprite, animation.as_ref());
            self.render_data.add_sprite(renderer_sprite);
        }
    }

    fn name(&self) -> &str {
        "SpriteRenderSystem"
    }
}

/// Helper functions for sprite management
pub mod sprite_utils {
    use super::*;

    /// Create a simple sprite entity
    pub fn create_sprite_entity(
        world: &mut World,
        position: Vec2,
        texture_handle: u32,
    ) -> EntityId {
        let entity = world.create_entity();

        world.add_component(&entity, Transform2D::new(position)).ok();
        world.add_component(&entity, Sprite::new(texture_handle)).ok();

        entity
    }

    /// Create an animated sprite entity
    pub fn create_animated_sprite_entity(
        world: &mut World,
        position: Vec2,
        texture_handle: u32,
        frames: Vec<[f32; 4]>,
        fps: f32,
    ) -> EntityId {
        let entity = world.create_entity();

        world.add_component(&entity, Transform2D::new(position)).ok();
        world.add_component(&entity, Sprite::new(texture_handle)).ok();
        world.add_component(&entity, SpriteAnimation::new(fps, frames)).ok();

        entity
    }

    /// Create a camera entity
    pub fn create_camera_entity(
        world: &mut World,
        position: Vec2,
        viewport_size: Vec2,
        is_main: bool,
    ) -> EntityId {
        let entity = world.create_entity();

        let mut camera = Camera2D::new(position, viewport_size);
        if is_main {
            camera.is_main_camera = true;
        }

        world.add_component(&entity, camera).ok();

        entity
    }

    /// Get all entities with sprite components
    pub fn get_sprite_entities(world: &World) -> Vec<EntityId> {
        world.entities()
            .into_iter()
            .filter(|entity_id| {
                world.get::<Sprite>(*entity_id).is_some() &&
                world.get::<Transform2D>(*entity_id).is_some()
            })
            .collect()
    }

    /// Get all camera entities
    pub fn get_camera_entities(world: &World) -> Vec<EntityId> {
        world.entities()
            .into_iter()
            .filter(|entity_id| world.get::<Camera2D>(*entity_id).is_some())
            .collect()
    }
}
