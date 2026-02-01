//! Scene saving for serializing World entities to RON files.

use std::collections::HashMap;

use ecs::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
use ecs::{EntityId, Name, World, WorldHierarchyExt};

use crate::assets::AssetManager;
use crate::scene_data::{ComponentData, EntityData, SceneData};

/// Scene saver for extracting world state to RON files.
pub struct SceneSaver;

impl SceneSaver {
    /// Extract all entities from world into SceneData.
    ///
    /// If `assets` is None, texture paths will default to "#white".
    pub fn extract_from_world(
        world: &World,
        assets: Option<&AssetManager>,
        scene_name: &str,
    ) -> SceneData {
        let mut entities = Vec::new();

        // Get root entities (those without parents)
        for root_id in world.get_root_entities() {
            if let Some(entity_data) = Self::extract_entity_recursive(world, assets, root_id) {
                entities.push(entity_data);
            }
        }

        SceneData {
            name: scene_name.to_string(),
            physics: None,
            editor: None,
            prefabs: HashMap::new(),
            entities,
        }
    }

    fn extract_entity_recursive(
        world: &World,
        assets: Option<&AssetManager>,
        entity: EntityId,
    ) -> Option<EntityData> {
        let components = Self::extract_components(world, assets, entity);

        // Recursively extract children
        let children: Vec<EntityData> = world
            .get_children(entity)
            .map(|child_ids| {
                child_ids
                    .iter()
                    .filter_map(|&child| Self::extract_entity_recursive(world, assets, child))
                    .collect()
            })
            .unwrap_or_default();

        Some(EntityData {
            name: Self::extract_name(world, entity),
            prefab: None,
            parent: None,
            overrides: Vec::new(),
            components,
            children,
        })
    }

    fn extract_components(
        world: &World,
        assets: Option<&AssetManager>,
        entity: EntityId,
    ) -> Vec<ComponentData> {
        let mut components = Vec::new();

        // Transform2D
        if let Some(t) = world.get::<Transform2D>(entity) {
            components.push(ComponentData::Transform2D {
                position: (t.position.x, t.position.y),
                rotation: t.rotation,
                scale: (t.scale.x, t.scale.y),
            });
        }

        // Sprite
        if let Some(s) = world.get::<Sprite>(entity) {
            let texture = assets
                .and_then(|a| a.get_texture_path(s.texture_handle))
                .unwrap_or("#white")
                .to_string();

            components.push(ComponentData::Sprite {
                texture,
                offset: (s.offset.x, s.offset.y),
                rotation: s.rotation,
                scale: (s.scale.x, s.scale.y),
                color: (s.color.x, s.color.y, s.color.z, s.color.w),
                depth: s.depth,
            });
        }

        // Camera
        if let Some(c) = world.get::<Camera>(entity) {
            components.push(ComponentData::Camera2D {
                position: (c.position.x, c.position.y),
                rotation: c.rotation,
                zoom: c.zoom,
                viewport_size: (c.viewport_size.x, c.viewport_size.y),
                is_main_camera: c.is_main_camera,
            });
        }

        // SpriteAnimation
        if let Some(a) = world.get::<SpriteAnimation>(entity) {
            components.push(ComponentData::SpriteAnimation {
                fps: a.fps,
                frames: a.frames.iter().map(|f| (f[0], f[1], f[2], f[3])).collect(),
                playing: a.playing,
                loop_animation: a.loop_animation,
            });
        }

        // RigidBody (physics feature)
        #[cfg(feature = "physics")]
        if let Some(rb) = world.get::<physics::components::RigidBody>(entity) {
            use crate::scene_data::RigidBodyTypeData;

            let body_type = match rb.body_type {
                physics::components::RigidBodyType::Dynamic => RigidBodyTypeData::Dynamic,
                physics::components::RigidBodyType::Static => RigidBodyTypeData::Static,
                physics::components::RigidBodyType::Kinematic => RigidBodyTypeData::Kinematic,
            };

            components.push(ComponentData::RigidBody {
                body_type,
                velocity: (rb.velocity.x, rb.velocity.y),
                angular_velocity: rb.angular_velocity,
                gravity_scale: rb.gravity_scale,
                linear_damping: rb.linear_damping,
                angular_damping: rb.angular_damping,
                can_rotate: rb.can_rotate,
                ccd_enabled: rb.ccd_enabled,
            });
        }

        // Collider (physics feature)
        #[cfg(feature = "physics")]
        if let Some(col) = world.get::<physics::components::Collider>(entity) {
            use crate::scene_data::ColliderShapeData;
            use physics::components::ColliderShape;

            let shape = match &col.shape {
                ColliderShape::Box { half_extents } => ColliderShapeData::Box {
                    half_extents: (half_extents.x, half_extents.y),
                },
                ColliderShape::Circle { radius } => ColliderShapeData::Circle { radius: *radius },
                ColliderShape::CapsuleY { half_height, radius } => ColliderShapeData::CapsuleY {
                    half_height: *half_height,
                    radius: *radius,
                },
                ColliderShape::CapsuleX { half_height, radius } => ColliderShapeData::CapsuleX {
                    half_height: *half_height,
                    radius: *radius,
                },
            };

            components.push(ComponentData::Collider {
                shape,
                offset: (col.offset.x, col.offset.y),
                is_sensor: col.is_sensor,
                friction: col.friction,
                restitution: col.restitution,
            });
        }

        components
    }

    fn extract_name(world: &World, entity: EntityId) -> Option<String> {
        world.get::<Name>(entity).map(|n| n.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
    use glam::{Vec2, Vec4};

    #[test]
    fn test_extract_empty_world() {
        let world = World::default();
        let scene = SceneSaver::extract_from_world(&world, None, "Empty");

        assert_eq!(scene.name, "Empty");
        assert!(scene.entities.is_empty());
        assert!(scene.prefabs.is_empty());
        assert!(scene.editor.is_none());
    }

    #[test]
    fn test_extract_entity_with_transform() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                Transform2D {
                    position: Vec2::new(100.0, 200.0),
                    rotation: 1.5,
                    scale: Vec2::new(2.0, 3.0),
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Transform2D {
                position,
                rotation,
                scale,
            } => {
                assert_eq!(*position, (100.0, 200.0));
                assert_eq!(*rotation, 1.5);
                assert_eq!(*scale, (2.0, 3.0));
            }
            _ => panic!("Expected Transform2D"),
        }
    }

    #[test]
    fn test_extract_entity_with_sprite() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                Sprite {
                    texture_handle: 0,
                    offset: Vec2::new(5.0, 10.0),
                    rotation: 0.5,
                    scale: Vec2::new(1.0, 1.0),
                    color: Vec4::new(1.0, 0.5, 0.25, 1.0),
                    depth: 5.0,
                    tex_region: [0.0, 0.0, 1.0, 1.0],
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Sprite {
                texture,
                offset,
                rotation,
                scale: _,
                color,
                depth,
            } => {
                assert_eq!(texture, "#white");
                assert_eq!(*offset, (5.0, 10.0));
                assert_eq!(*rotation, 0.5);
                assert_eq!(*color, (1.0, 0.5, 0.25, 1.0));
                assert_eq!(*depth, 5.0);
            }
            _ => panic!("Expected Sprite"),
        }
    }

    #[test]
    fn test_extract_entity_with_camera() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                Camera {
                    position: Vec2::new(50.0, 75.0),
                    rotation: 0.25,
                    zoom: 1.5,
                    viewport_size: Vec2::new(1920.0, 1080.0),
                    is_main_camera: true,
                    near: -1000.0,
                    far: 1000.0,
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        match &scene.entities[0].components[0] {
            ComponentData::Camera2D {
                position,
                zoom,
                is_main_camera,
                ..
            } => {
                assert_eq!(*position, (50.0, 75.0));
                assert_eq!(*zoom, 1.5);
                assert!(*is_main_camera);
            }
            _ => panic!("Expected Camera2D"),
        }
    }

    #[test]
    fn test_extract_entity_with_animation() {
        let mut world = World::default();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                SpriteAnimation {
                    fps: 12.0,
                    frames: vec![[0.0, 0.0, 0.25, 0.25], [0.25, 0.0, 0.5, 0.25]],
                    playing: true,
                    loop_animation: false,
                    current_frame: 0,
                    time_accumulator: 0.0,
                },
            )
            .unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        match &scene.entities[0].components[0] {
            ComponentData::SpriteAnimation {
                fps,
                frames,
                playing,
                loop_animation,
            } => {
                assert_eq!(*fps, 12.0);
                assert_eq!(frames.len(), 2);
                assert!(*playing);
                assert!(!*loop_animation);
            }
            _ => panic!("Expected SpriteAnimation"),
        }
    }

    #[cfg(feature = "physics")]
    #[test]
    fn test_extract_entity_with_rigidbody() {
        use crate::scene_data::RigidBodyTypeData;
        use physics::components::RigidBody;

        let mut world = World::default();
        let entity = world.create_entity();

        let mut rb = RigidBody::new_dynamic();
        rb.velocity = Vec2::new(10.0, 20.0);
        rb.gravity_scale = 0.5;
        rb.linear_damping = 2.0;
        world.add_component(&entity, rb).unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        match &scene.entities[0].components[0] {
            ComponentData::RigidBody {
                body_type,
                velocity,
                gravity_scale,
                linear_damping,
                ..
            } => {
                assert_eq!(*body_type, RigidBodyTypeData::Dynamic);
                assert_eq!(*velocity, (10.0, 20.0));
                assert_eq!(*gravity_scale, 0.5);
                assert_eq!(*linear_damping, 2.0);
            }
            _ => panic!("Expected RigidBody"),
        }
    }

    #[cfg(feature = "physics")]
    #[test]
    fn test_extract_entity_with_collider() {
        use crate::scene_data::ColliderShapeData;
        use physics::components::{Collider, ColliderShape};

        let mut world = World::default();
        let entity = world.create_entity();

        let mut collider = Collider::new(ColliderShape::Box {
            half_extents: Vec2::new(32.0, 16.0),
        });
        collider.friction = 0.8;
        collider.restitution = 0.2;
        world.add_component(&entity, collider).unwrap();

        let scene = SceneSaver::extract_from_world(&world, None, "Test");

        assert_eq!(scene.entities.len(), 1);
        match &scene.entities[0].components[0] {
            ComponentData::Collider {
                shape,
                friction,
                restitution,
                ..
            } => {
                match shape {
                    ColliderShapeData::Box { half_extents } => {
                        assert_eq!(*half_extents, (32.0, 16.0));
                    }
                    _ => panic!("Expected Box shape"),
                }
                assert_eq!(*friction, 0.8);
                assert_eq!(*restitution, 0.2);
            }
            _ => panic!("Expected Collider"),
        }
    }
}
