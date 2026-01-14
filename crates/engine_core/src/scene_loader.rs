//! Scene loader for RON scene files
//!
//! This module provides functionality to load scenes from RON files,
//! resolve prefabs, and instantiate entities in the ECS world.

use std::collections::HashMap;
use std::path::Path;

use glam::Vec2;

use ecs::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
use ecs::{EntityId, World, WorldHierarchyExt};
use renderer::TextureHandle;

use crate::assets::AssetManager;
use crate::scene_data::{
    BehaviorData, ColliderShapeData, ComponentData, EntityData, PhysicsSettings, PrefabData,
    RigidBodyTypeData, SceneData, SceneLoadError,
};

/// Convert scene serialization data to ECS component
impl From<&BehaviorData> for ecs::behavior::Behavior {
    fn from(data: &BehaviorData) -> Self {
        match data {
            BehaviorData::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
                Self::PlayerPlatformer { move_speed: *move_speed, jump_impulse: *jump_impulse, jump_cooldown: *jump_cooldown, tag: tag.clone() }
            }
            BehaviorData::PlayerTopDown { move_speed, tag } => {
                Self::PlayerTopDown { move_speed: *move_speed, tag: tag.clone() }
            }
            BehaviorData::FollowEntity { target_name, follow_distance, follow_speed } => {
                Self::FollowEntity { target_name: target_name.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            BehaviorData::FollowTagged { target_tag, follow_distance, follow_speed } => {
                Self::FollowTagged { target_tag: target_tag.clone(), follow_distance: *follow_distance, follow_speed: *follow_speed }
            }
            BehaviorData::Patrol { point_a, point_b, speed, wait_time } => {
                Self::Patrol { point_a: *point_a, point_b: *point_b, speed: *speed, wait_time: *wait_time }
            }
            BehaviorData::Collectible { score_value, despawn_on_collect, collector_tag } => {
                Self::Collectible { score_value: *score_value, despawn_on_collect: *despawn_on_collect, collector_tag: collector_tag.clone() }
            }
            BehaviorData::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
                Self::ChaseTagged { target_tag: target_tag.clone(), detection_range: *detection_range, chase_speed: *chase_speed, lose_interest_range: *lose_interest_range }
            }
        }
    }
}

/// Result of loading a scene
#[derive(Debug)]
pub struct SceneInstance {
    /// Scene name
    pub name: String,
    /// Physics settings (if any)
    pub physics: Option<PhysicsSettings>,
    /// Mapping from entity names to EntityIds
    pub named_entities: HashMap<String, EntityId>,
    /// All created entity IDs
    pub entities: Vec<EntityId>,
    /// Number of entities created
    pub entity_count: usize,
}

impl SceneInstance {
    /// Get an entity by name
    pub fn get_entity(&self, name: &str) -> Option<EntityId> {
        self.named_entities.get(name).copied()
    }
}

/// Scene loader for parsing and instantiating RON scene files
pub struct SceneLoader;

impl SceneLoader {
    /// Load scene data from a RON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<SceneData, SceneLoadError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&content)
    }

    /// Parse scene data from a RON string
    pub fn parse(content: &str) -> Result<SceneData, SceneLoadError> {
        ron::from_str(content).map_err(SceneLoadError::RonError)
    }

    /// Instantiate a scene in the given world
    ///
    /// This creates all entities defined in the scene, resolving prefabs
    /// and applying overrides as specified. Parent-child relationships are
    /// established based on the `parent` field or inline `children`.
    pub fn instantiate(
        data: &SceneData,
        world: &mut World,
        assets: &mut AssetManager,
    ) -> Result<SceneInstance, SceneLoadError> {
        let mut named_entities = HashMap::new();
        let mut entities = Vec::new();

        // First pass: create all entities and collect their IDs
        for entity_data in &data.entities {
            Self::create_entity_recursive(
                entity_data,
                &data.prefabs,
                world,
                assets,
                &mut named_entities,
                &mut entities,
                None, // No parent for top-level entities initially
            )?;
        }

        // Second pass: establish parent relationships for entities using `parent` field
        for entity_data in &data.entities {
            if let Some(parent_name) = &entity_data.parent {
                if let Some(entity_name) = &entity_data.name {
                    if let (Some(&entity_id), Some(&parent_id)) = (
                        named_entities.get(entity_name),
                        named_entities.get(parent_name),
                    ) {
                        world.set_parent(entity_id, parent_id).ok();

                        // Add GlobalTransform2D component if the entity has Transform2D
                        if world.get::<Transform2D>(entity_id).is_some() {
                            let _ = world.add_component(
                                &entity_id,
                                ecs::hierarchy::GlobalTransform2D::default(),
                            );
                        }
                    }
                }
            }
        }

        let entity_count = entities.len();

        Ok(SceneInstance {
            name: data.name.clone(),
            physics: data.physics.clone(),
            named_entities,
            entities,
            entity_count,
        })
    }

    /// Recursively create an entity and its inline children
    fn create_entity_recursive(
        entity_data: &EntityData,
        prefabs: &HashMap<String, PrefabData>,
        world: &mut World,
        assets: &mut AssetManager,
        named_entities: &mut HashMap<String, EntityId>,
        entities: &mut Vec<EntityId>,
        parent_id: Option<EntityId>,
    ) -> Result<EntityId, SceneLoadError> {
        let entity_id = Self::create_entity(entity_data, prefabs, world, assets)?;
        entities.push(entity_id);

        if let Some(name) = &entity_data.name {
            named_entities.insert(name.clone(), entity_id);
        }

        // Set up parent relationship if specified
        if let Some(parent) = parent_id {
            world.set_parent(entity_id, parent).ok();

            // Add GlobalTransform2D component for hierarchical entities
            if world.get::<Transform2D>(entity_id).is_some() {
                let _ = world.add_component(&entity_id, ecs::hierarchy::GlobalTransform2D::default());
            }
        }

        // Create inline children recursively
        for child_data in &entity_data.children {
            Self::create_entity_recursive(
                child_data,
                prefabs,
                world,
                assets,
                named_entities,
                entities,
                Some(entity_id),
            )?;
        }

        Ok(entity_id)
    }

    /// Load a scene from file and instantiate it
    pub fn load_and_instantiate(
        path: impl AsRef<Path>,
        world: &mut World,
        assets: &mut AssetManager,
    ) -> Result<SceneInstance, SceneLoadError> {
        let data = Self::load_from_file(path)?;
        Self::instantiate(&data, world, assets)
    }

    /// Create a single entity from EntityData
    fn create_entity(
        entity_data: &EntityData,
        prefabs: &HashMap<String, PrefabData>,
        world: &mut World,
        assets: &mut AssetManager,
    ) -> Result<EntityId, SceneLoadError> {
        let entity_id = world.create_entity();

        // Get base components from prefab (if any)
        let base_components = if let Some(prefab_name) = &entity_data.prefab {
            let prefab = prefabs
                .get(prefab_name)
                .ok_or_else(|| SceneLoadError::PrefabNotFound(prefab_name.clone()))?;
            prefab.components.clone()
        } else {
            Vec::new()
        };

        // Merge base components with overrides and inline components
        let merged_components =
            Self::merge_components(&base_components, &entity_data.overrides, &entity_data.components);

        // Apply all components to the entity
        for component in &merged_components {
            Self::add_component_to_entity(entity_id, component, world, assets)?;
        }

        Ok(entity_id)
    }

    /// Merge prefab components with overrides and inline components
    fn merge_components(
        base: &[ComponentData],
        overrides: &[ComponentData],
        inline: &[ComponentData],
    ) -> Vec<ComponentData> {
        let mut result: Vec<ComponentData> = base.to_vec();

        // Apply overrides (replace matching component types)
        for override_comp in overrides {
            let component_type = Self::component_type_name(override_comp);
            if let Some(pos) = result
                .iter()
                .position(|c| Self::component_type_name(c) == component_type)
            {
                result[pos] = override_comp.clone();
            } else {
                result.push(override_comp.clone());
            }
        }

        // Add inline components (these take precedence over everything)
        for inline_comp in inline {
            let component_type = Self::component_type_name(inline_comp);
            if let Some(pos) = result
                .iter()
                .position(|c| Self::component_type_name(c) == component_type)
            {
                result[pos] = inline_comp.clone();
            } else {
                result.push(inline_comp.clone());
            }
        }

        result
    }

    /// Get a simple type name for component matching
    fn component_type_name(component: &ComponentData) -> &'static str {
        match component {
            ComponentData::Transform2D { .. } => "Transform2D",
            ComponentData::Sprite { .. } => "Sprite",
            ComponentData::Camera2D { .. } => "Camera2D",
            ComponentData::SpriteAnimation { .. } => "SpriteAnimation",
            ComponentData::RigidBody { .. } => "RigidBody",
            ComponentData::Collider { .. } => "Collider",
            ComponentData::Behavior(_) => "Behavior",
        }
    }

    /// Add a component to an entity based on ComponentData
    fn add_component_to_entity(
        entity_id: EntityId,
        component: &ComponentData,
        world: &mut World,
        assets: &mut AssetManager,
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
                let _ = world.add_component(&entity_id, transform);
            }

            ComponentData::Sprite {
                texture,
                offset,
                rotation,
                scale,
                color,
                depth,
            } => {
                let texture_handle = Self::resolve_texture(texture, assets)?;
                let sprite = Sprite {
                    texture_handle: texture_handle.id,
                    offset: Vec2::new(offset.0, offset.1),
                    rotation: *rotation,
                    scale: Vec2::new(scale.0, scale.1),
                    color: glam::Vec4::new(color.0, color.1, color.2, color.3),
                    depth: *depth,
                    tex_region: [0.0, 0.0, 1.0, 1.0],
                };
                let _ = world.add_component(&entity_id, sprite);
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
                let _ = world.add_component(&entity_id, camera);
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
                let _ = world.add_component(&entity_id, animation);
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

                    let _ = world.add_component(&entity_id, rigid_body);
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

                    let _ = world.add_component(&entity_id, collider);
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

            ComponentData::Behavior(behavior_data) => {
                let behavior: ecs::behavior::Behavior = behavior_data.into();
                let _ = world.add_component(&entity_id, behavior);
            }
        }

        Ok(())
    }

    /// Resolve a texture reference to a TextureHandle
    ///
    /// Texture references can be:
    /// - `#white` - Use the white texture (handle 0) for color tinting
    /// - `#solid:RRGGBB` - Create a solid color texture
    /// - Any other string - Load as a file path
    fn resolve_texture(
        texture_ref: &str,
        assets: &mut AssetManager,
    ) -> Result<TextureHandle, SceneLoadError> {
        if texture_ref == "#white" {
            // White texture is always handle 0
            return Ok(TextureHandle { id: 0 });
        }

        if let Some(hex) = texture_ref.strip_prefix("#solid:") {
            // Parse hex color and create solid color texture
            let color = Self::parse_hex_color(hex)?;
            assets
                .create_solid_color(1, 1, color)
                .map_err(|e| SceneLoadError::TextureLoadError(e.to_string()))
        } else {
            // Load as file path
            assets
                .load_texture(texture_ref)
                .map_err(|e| SceneLoadError::TextureLoadError(e.to_string()))
        }
    }

    /// Parse a hex color string (RRGGBB or RRGGBBAA) to [u8; 4]
    fn parse_hex_color(hex: &str) -> Result<[u8; 4], SceneLoadError> {
        let hex = hex.trim_start_matches('#');

        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            Ok([r, g, b, 255])
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?;
            Ok([r, g, b, a])
        } else {
            Err(SceneLoadError::InvalidTextureRef(format!(
                "Hex color must be 6 or 8 characters: {}",
                hex
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_6() {
        let color = SceneLoader::parse_hex_color("FF0000").unwrap();
        assert_eq!(color, [255, 0, 0, 255]);

        let color = SceneLoader::parse_hex_color("00FF00").unwrap();
        assert_eq!(color, [0, 255, 0, 255]);

        let color = SceneLoader::parse_hex_color("0000FF").unwrap();
        assert_eq!(color, [0, 0, 255, 255]);
    }

    #[test]
    fn test_parse_hex_color_8() {
        let color = SceneLoader::parse_hex_color("FF000080").unwrap();
        assert_eq!(color, [255, 0, 0, 128]);
    }

    #[test]
    fn test_parse_scene_basic() {
        let scene_ron = r#"
            SceneData(
                name: "Test Scene",
                entities: [
                    EntityData(
                        name: Some("player"),
                        components: [
                            Transform2D(position: (100.0, 200.0)),
                        ],
                    ),
                ],
            )
        "#;

        let scene = SceneLoader::parse(scene_ron).unwrap();
        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, Some("player".to_string()));
    }

    #[test]
    fn test_parse_scene_with_prefabs() {
        let scene_ron = r##"
            SceneData(
                name: "Prefab Test",
                prefabs: {
                    "Enemy": PrefabData(
                        components: [
                            Transform2D(position: (0.0, 0.0)),
                            Sprite(texture: "#white", color: (1.0, 0.0, 0.0, 1.0)),
                        ],
                    ),
                },
                entities: [
                    EntityData(
                        name: Some("enemy1"),
                        prefab: Some("Enemy"),
                        overrides: [
                            Transform2D(position: (500.0, 100.0)),
                        ],
                    ),
                ],
            )
        "##;

        let scene = SceneLoader::parse(scene_ron).unwrap();
        assert_eq!(scene.prefabs.len(), 1);
        assert!(scene.prefabs.contains_key("Enemy"));
    }

    #[test]
    fn test_merge_components() {
        let base = vec![ComponentData::Transform2D {
            position: (0.0, 0.0),
            rotation: 0.0,
            scale: (1.0, 1.0),
        }];

        let overrides = vec![ComponentData::Transform2D {
            position: (100.0, 200.0),
            rotation: 0.0,
            scale: (1.0, 1.0),
        }];

        let inline = vec![];

        let merged = SceneLoader::merge_components(&base, &overrides, &inline);
        assert_eq!(merged.len(), 1);

        if let ComponentData::Transform2D { position, .. } = &merged[0] {
            assert_eq!(*position, (100.0, 200.0));
        } else {
            panic!("Expected Transform2D");
        }
    }
}
