//! Sprite rendering system for ECS integration

use glam::Vec2;
use crate::{
    World, System, SystemContext, EntityId,
    sprite_components::{Sprite, Transform2D, Camera2D, SpriteAnimation, SpriteRenderData},
    component::Component,
};

/// System that updates sprite animations
pub struct SpriteAnimationSystem;

impl System for SpriteAnimationSystem {
    fn update(&mut self, ctx: &mut SystemContext, _world: &mut World) -> Result<(), String> {
        let delta_time = ctx.delta_time;
        
        // Update all sprite animations
        for entity_id in _world.entities() {
            if let Some(animation) = _world.component_registry.get_mut::<SpriteAnimation>(&entity_id) {
                // We need to cast to mutable reference
                if let Some(animation) = animation.as_any_mut().downcast_mut::<SpriteAnimation>() {
                    animation.update(delta_time);
                }
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
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
            if let Some(camera) = world.component_registry.get::<Camera2D>(&entity_id) {
                if let Some(camera) = camera.as_any().downcast_ref::<Camera2D>() {
                    if camera.is_main_camera {
                        return Some(entity_id);
                    }
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
        animation: Option<&SpriteAnimation>,
    ) -> RendererSprite {
        let world_position = entity_transform.position + entity_transform.transform_point(sprite.offset);
        let world_rotation = entity_transform.rotation + sprite.rotation;
        let world_scale = entity_transform.scale * sprite.scale;
        
        // Use animation frame if available
        let tex_region = if let Some(animation) = animation {
            animation.current_frame_region()
        } else {
            sprite.tex_region
        };

        RendererSprite {
            position: world_position,
            rotation: world_rotation,
            scale: world_scale,
            tex_region,
            color: sprite.color,
            depth: sprite.depth,
            texture_handle: sprite.texture_handle,
        }
    }
}

impl Default for SpriteRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for SpriteRenderSystem {
    fn update(&mut self, _ctx: &mut SystemContext, world: &mut World) -> Result<(), String> {
        // Clear previous frame's data
        self.render_data.clear();

        // Find main camera
        if let Some(camera_entity) = self.find_main_camera(world) {
            if let Some(camera) = world.component_registry.get::<Camera2D>(&camera_entity) {
                if let Some(camera) = camera.as_any().downcast_ref::<Camera2D>() {
                    let renderer_camera = RendererCamera2D {
                        position: camera.position,
                        rotation: camera.rotation,
                        zoom: camera.zoom,
                        viewport_size: camera.viewport_size,
                        near: -1000.0,
                        far: 1000.0,
                    };
                    self.render_data.set_camera(renderer_camera);
                }
            }
        }

        // Collect all sprites
        for entity_id in world.entities() {
            // Get transform (required)
            let transform = if let Some(transform) = world.component_registry.get::<Transform2D>(&entity_id) {
                if let Some(transform) = transform.as_any().downcast_ref::<Transform2D>() {
                    transform
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Get sprite (required)
            let sprite = if let Some(sprite) = world.component_registry.get::<Sprite>(&entity_id) {
                if let Some(sprite) = sprite.as_any().downcast_ref::<Sprite>() {
                    sprite
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Get animation (optional)
            let animation = world.component_registry.get::<SpriteAnimation>(&entity_id)
                .and_then(|a| a.as_any().downcast_ref::<SpriteAnimation>());

            // Convert to renderer sprite
            let renderer_sprite = self.convert_sprite(transform, sprite, animation);
            self.render_data.add_sprite(renderer_sprite);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "SpriteRenderSystem"
    }
}

/// Simple renderer-compatible sprite data structure
#[derive(Debug, Clone)]
pub struct RendererSprite {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: Vec2,
    /// Texture region (x, y, width, height) in texture coordinates [0, 1]
    pub tex_region: [f32; 4],
    /// Color tint
    pub color: Vec4,
    /// Layer depth for sorting (higher values render on top)
    pub depth: f32,
    /// Texture handle
    pub texture_handle: u32,
}

/// Simple renderer-compatible camera data structure
#[derive(Debug, Clone)]
pub struct RendererCamera2D {
    /// Camera position in world space
    pub position: Vec2,
    /// Camera rotation in radians
    pub rotation: f32,
    /// Zoom level (1.0 = normal, 2.0 = 2x zoom in, 0.5 = 2x zoom out)
    pub zoom: f32,
    /// Viewport dimensions
    pub viewport_size: Vec2,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
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
        
        world.component_registry.add(entity, Transform2D::new(position));
        world.component_registry.add(entity, Sprite::new(texture_handle));
        
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
        
        world.component_registry.add(entity, Transform2D::new(position));
        world.component_registry.add(entity, Sprite::new(texture_handle));
        world.component_registry.add(entity, SpriteAnimation::new(fps, frames));
        
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
        
        world.component_registry.add(entity, camera);
        
        entity
    }

    /// Get all entities with sprite components
    pub fn get_sprite_entities(world: &World) -> Vec<EntityId> {
        world.entities()
            .into_iter()
            .filter(|entity_id| {
                world.component_registry.has::<Sprite>(entity_id) &&
                world.component_registry.has::<Transform2D>(entity_id)
            })
            .collect()
    }

    /// Get all camera entities
    pub fn get_camera_entities(world: &World) -> Vec<EntityId> {
        world.entities()
            .into_iter()
            .filter(|entity_id| world.component_registry.has::<Camera2D>(entity_id))
            .collect()
    }
}