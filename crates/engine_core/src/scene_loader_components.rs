//! Scene-component construction: `ComponentData` -> ECS components.
//!
//! Split from `scene_loader.rs` (file size); this is the loader half of the
//! scene schema. The inverse (World -> `ComponentData`) lives in
//! `scene_serializer.rs` — new component types need an arm in BOTH.

use glam::Vec2;

use ecs::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
use ecs::{EntityId, World};

use crate::scene_data::{ColliderShapeData, ComponentData, RigidBodyTypeData, SceneLoadError};
use crate::texture_ref::TextureResolver;

use crate::scene_loader::SceneLoader;

impl SceneLoader {
    /// Get a simple type name for component matching
    pub(crate) fn component_type_name(component: &ComponentData) -> &str {
        match component {
            ComponentData::Transform2D { .. } => "Transform2D",
            ComponentData::Sprite { .. } => "Sprite",
            ComponentData::Camera2D { .. } => "Camera2D",
            ComponentData::Tilemap { .. } => "Tilemap",
            ComponentData::SpriteAnimation { .. } => "SpriteAnimation",
            ComponentData::RigidBody { .. } => "RigidBody",
            ComponentData::Collider { .. } => "Collider",
            ComponentData::UiLabel { .. } => "UiLabel",
            ComponentData::UiPanel { .. } => "UiPanel",
            ComponentData::UiButton { .. } => "UiButton",
            ComponentData::Behavior(_) => "Behavior",
            ComponentData::EntityTag { .. } => "EntityTag",
            ComponentData::Dynamic { component_type, .. } => component_type.as_str(),
        }
    }

    /// Add a component to an entity based on ComponentData
    pub(crate) fn add_component_to_entity(
        entity_id: EntityId,
        component: &ComponentData,
        world: &mut World,
        assets: &mut impl TextureResolver,
    ) -> Result<(), SceneLoadError> {
        match component {
            ComponentData::Transform2D {
                position,
                rotation,
                scale,
            } => {
                let transform = Transform2D {
                    position: Vec2::new(position.0, position.1),
                    rotation: *rotation,
                    scale: Vec2::new(scale.0, scale.1),
                };
                Self::add_component_logged(world, entity_id, transform);
            }

            ComponentData::Sprite {
                texture,
                offset,
                rotation,
                scale,
                color,
                depth,
                emissive,
            } => {
                let texture_handle = assets.resolve_texture(texture)?;
                let sprite = Sprite {
                    texture_handle: texture_handle.id,
                    offset: Vec2::new(offset.0, offset.1),
                    rotation: *rotation,
                    scale: Vec2::new(scale.0, scale.1),
                    color: glam::Vec4::new(color.0, color.1, color.2, color.3),
                    depth: *depth,
                    visible: true,
                    emissive: *emissive,
                    tex_region: [0.0, 0.0, 1.0, 1.0],
                };
                Self::add_component_logged(world, entity_id, sprite);
            }

            ComponentData::Camera2D {
                position,
                rotation,
                zoom,
                viewport_size,
                is_main_camera,
            } => {
                let camera = Camera {
                    position: Vec2::new(position.0, position.1),
                    rotation: *rotation,
                    zoom: *zoom,
                    viewport_size: Vec2::new(viewport_size.0, viewport_size.1),
                    is_main_camera: *is_main_camera,
                    near: -1000.0,
                    far: 1000.0,
                };
                Self::add_component_logged(world, entity_id, camera);
            }

            ComponentData::Tilemap {
                tileset,
                width,
                height,
                tile_size,
                tiles,
                tile_uv_size,
                depth,
            } => {
                let texture_handle = assets.resolve_texture(tileset)?;
                let tilemap = ecs::Tilemap {
                    width: *width,
                    height: *height,
                    tile_size: *tile_size,
                    tileset: texture_handle.id,
                    tiles: tiles.clone(),
                    tile_uv_size: Vec2::new(tile_uv_size.0, tile_uv_size.1),
                    depth: *depth,
                };
                Self::add_component_logged(world, entity_id, tilemap);
            }

            ComponentData::SpriteAnimation {
                fps,
                frames,
                playing,
                loop_animation,
            } => {
                let animation = SpriteAnimation {
                    fps: *fps,
                    frames: frames
                        .iter()
                        .map(|f| [f.0, f.1, f.2, f.3])
                        .collect(),
                    playing: *playing,
                    loop_animation: *loop_animation,
                    current_frame: 0,
                    time_accumulator: 0.0,
                };
                Self::add_component_logged(world, entity_id, animation);
            }

            ComponentData::RigidBody {
                body_type,
                velocity,
                angular_velocity,
                gravity_scale,
                linear_damping,
                angular_damping,
                can_rotate,
                ccd_enabled,
            } => {
                #[cfg(feature = "physics")]
                {
                    use physics::components::RigidBody;

                    let mut rigid_body = match body_type {
                        RigidBodyTypeData::Dynamic => RigidBody::new_dynamic(),
                        RigidBodyTypeData::Static => RigidBody::new_static(),
                        RigidBodyTypeData::Kinematic => RigidBody::new_kinematic(),
                    };

                    rigid_body.velocity = Vec2::new(velocity.0, velocity.1);
                    rigid_body.angular_velocity = *angular_velocity;
                    rigid_body.gravity_scale = *gravity_scale;
                    rigid_body.linear_damping = *linear_damping;
                    rigid_body.angular_damping = *angular_damping;
                    rigid_body.can_rotate = *can_rotate;
                    rigid_body.ccd_enabled = *ccd_enabled;

                    Self::add_component_logged(world, entity_id, rigid_body);
                }

                #[cfg(not(feature = "physics"))]
                {
                    log::warn!(
                        "RigidBody component in scene but physics feature is disabled"
                    );
                    // Suppress unused variable warnings
                    let _ = (body_type, velocity, angular_velocity, gravity_scale,
                             linear_damping, angular_damping, can_rotate, ccd_enabled);
                }
            }

            ComponentData::Collider {
                shape,
                offset,
                is_sensor,
                friction,
                restitution,
            } => {
                #[cfg(feature = "physics")]
                {
                    use physics::components::{Collider, ColliderShape};

                    let collider_shape = match shape {
                        ColliderShapeData::Box { half_extents } => ColliderShape::Box {
                            half_extents: Vec2::new(half_extents.0, half_extents.1),
                        },
                        ColliderShapeData::Circle { radius } => {
                            ColliderShape::Circle { radius: *radius }
                        }
                        ColliderShapeData::CapsuleY { half_height, radius } => {
                            ColliderShape::CapsuleY {
                                half_height: *half_height,
                                radius: *radius,
                            }
                        }
                        ColliderShapeData::CapsuleX { half_height, radius } => {
                            ColliderShape::CapsuleX {
                                half_height: *half_height,
                                radius: *radius,
                            }
                        }
                    };

                    let mut collider = Collider::new(collider_shape);
                    collider.offset = Vec2::new(offset.0, offset.1);
                    collider.is_sensor = *is_sensor;
                    collider.friction = *friction;
                    collider.restitution = *restitution;

                    Self::add_component_logged(world, entity_id, collider);
                }

                #[cfg(not(feature = "physics"))]
                {
                    log::warn!(
                        "Collider component in scene but physics feature is disabled"
                    );
                    // Suppress unused variable warnings
                    let _ = (shape, offset, is_sensor, friction, restitution);
                }
            }

            ComponentData::UiLabel { text, anchor, offset, font_size, color, visible } => {
                let label = ecs::UiLabel {
                    text: text.clone(),
                    anchor: *anchor,
                    offset: Vec2::new(offset.0, offset.1),
                    font_size: *font_size,
                    color: glam::Vec4::new(color.0, color.1, color.2, color.3),
                    visible: *visible,
                };
                Self::add_component_logged(world, entity_id, label);
            }

            ComponentData::UiPanel {
                anchor,
                offset,
                size,
                background,
                border,
                border_width,
                visible,
            } => {
                let panel = ecs::UiPanel {
                    anchor: *anchor,
                    offset: Vec2::new(offset.0, offset.1),
                    size: Vec2::new(size.0, size.1),
                    background: glam::Vec4::new(background.0, background.1, background.2, background.3),
                    border: glam::Vec4::new(border.0, border.1, border.2, border.3),
                    border_width: *border_width,
                    visible: *visible,
                };
                Self::add_component_logged(world, entity_id, panel);
            }

            ComponentData::UiButton { text, id, anchor, offset, size, visible } => {
                let button = ecs::UiButton {
                    text: text.clone(),
                    id: id.clone(),
                    anchor: *anchor,
                    offset: Vec2::new(offset.0, offset.1),
                    size: Vec2::new(size.0, size.1),
                    visible: *visible,
                };
                Self::add_component_logged(world, entity_id, button);
            }

            ComponentData::Behavior(behavior_data) => {
                let behavior: ecs::behavior::Behavior = behavior_data.into();
                Self::add_component_logged(world, entity_id, behavior);
            }

            ComponentData::EntityTag { tag } => {
                Self::add_component_logged(world, entity_id, ecs::behavior::EntityTag::new(tag.clone()));
            }

            ComponentData::Dynamic { component_type, data } => {
                // Use the component registry to create the component
                let registry = ecs::component_registry::global_registry();

                if !registry.is_registered(component_type) {
                    return Err(SceneLoadError::ComponentError(format!(
                        "Unknown component type '{}' - not registered in ComponentRegistry",
                        component_type
                    )));
                }

                // Create the component via factory
                match registry.create_component(component_type, data.clone()) {
                    Ok(_boxed_component) => {
                        // TODO: World needs type-erased component addition to fully support this.
                        // For now, we validate the component can be created but log a warning.
                        log::warn!(
                            "Dynamic component '{}' created but World lacks type-erased storage. \
                             Use explicit ComponentData variants for now.",
                            component_type
                        );
                    }
                    Err(e) => {
                        return Err(SceneLoadError::ComponentError(format!(
                            "Failed to create component '{}': {}",
                            component_type, e
                        )));
                    }
                }
            }
        }

        Ok(())
    }

}
