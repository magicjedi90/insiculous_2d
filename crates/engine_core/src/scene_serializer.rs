//! Scene serializer — converts a live World into SceneData for RON serialization.
//!
//! This is the inverse of `scene_loader.rs`. It walks the ECS world, extracts
//! known component types from each entity, and produces a `SceneData` structure
//! that can be written to a `.scene.ron` file.

use std::path::Path;

use ecs::sprite_components::{Camera, Name, Sprite, SpriteAnimation, Transform2D};
use ecs::{EntityId, World, WorldHierarchyExt};

use crate::scene_data::*;

/// Convert a World into SceneData suitable for RON serialization.
///
/// The `texture_path_fn` closure maps texture handle IDs back to their
/// original path strings (e.g., `"#white"`, `"player.png"`). Callers with
/// access to an `AssetManager` can use `assets.texture_path(handle)`;
/// tests can provide a simple default.
pub fn world_to_scene_data(
    world: &World,
    scene_name: &str,
    physics_settings: Option<PhysicsSettings>,
    texture_path_fn: &dyn Fn(u32) -> String,
) -> SceneData {
    let roots = world.get_root_entities();

    let entities: Vec<EntityData> = roots
        .iter()
        .map(|&root| entity_to_entity_data(world, root, texture_path_fn))
        .collect();

    SceneData {
        name: scene_name.to_string(),
        physics: physics_settings,
        prefabs: std::collections::HashMap::new(),
        entities,
    }
}

/// Convert a single entity to EntityData, recursively including children.
fn entity_to_entity_data(
    world: &World,
    entity: EntityId,
    texture_path_fn: &dyn Fn(u32) -> String,
) -> EntityData {
    let name = world.get::<Name>(entity).map(|n| n.as_str().to_string());
    let components = extract_components(world, entity, texture_path_fn);

    let children_ids = world.get_children(entity).unwrap_or(&[]);
    let children: Vec<EntityData> = children_ids
        .iter()
        .map(|&child| entity_to_entity_data(world, child, texture_path_fn))
        .collect();

    EntityData {
        name,
        prefab: None,
        parent: None,
        overrides: Vec::new(),
        components,
        children,
    }
}

/// Extract all serializable components from an entity as ComponentData variants.
///
/// Skips computed/internal components (GlobalTransform2D, Parent, Children)
/// and audio components (not yet in ComponentData enum). The `Name` component
/// is handled separately as `EntityData.name`.
fn extract_components(
    world: &World,
    entity: EntityId,
    texture_path_fn: &dyn Fn(u32) -> String,
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
        components.push(ComponentData::Sprite {
            texture: texture_path_fn(s.texture_handle),
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
        let frames: Vec<(f32, f32, f32, f32)> = a
            .frames
            .iter()
            .map(|f| (f[0], f[1], f[2], f[3]))
            .collect();
        components.push(ComponentData::SpriteAnimation {
            fps: a.fps,
            frames,
            playing: a.playing,
            loop_animation: a.loop_animation,
        });
    }

    // RigidBody (behind physics feature)
    #[cfg(feature = "physics")]
    if let Some(rb) = world.get::<physics::components::RigidBody>(entity) {
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

    // Collider (behind physics feature)
    #[cfg(feature = "physics")]
    if let Some(col) = world.get::<physics::components::Collider>(entity) {
        let shape = match &col.shape {
            physics::components::ColliderShape::Box { half_extents } => {
                ColliderShapeData::Box {
                    half_extents: (half_extents.x, half_extents.y),
                }
            }
            physics::components::ColliderShape::Circle { radius } => {
                ColliderShapeData::Circle { radius: *radius }
            }
            physics::components::ColliderShape::CapsuleY { half_height, radius } => {
                ColliderShapeData::CapsuleY {
                    half_height: *half_height,
                    radius: *radius,
                }
            }
            physics::components::ColliderShape::CapsuleX { half_height, radius } => {
                ColliderShapeData::CapsuleX {
                    half_height: *half_height,
                    radius: *radius,
                }
            }
        };
        components.push(ComponentData::Collider {
            shape,
            offset: (col.offset.x, col.offset.y),
            is_sensor: col.is_sensor,
            friction: col.friction,
            restitution: col.restitution,
        });
    }

    // Behavior
    if let Some(b) = world.get::<ecs::behavior::Behavior>(entity) {
        let behavior_data = behavior_to_data(b);
        components.push(ComponentData::Behavior(behavior_data));
    }

    components
}

/// Convert an ECS Behavior to its serialization counterpart BehaviorData.
fn behavior_to_data(behavior: &ecs::behavior::Behavior) -> BehaviorData {
    match behavior {
        ecs::behavior::Behavior::PlayerPlatformer {
            move_speed,
            jump_impulse,
            jump_cooldown,
            tag,
        } => BehaviorData::PlayerPlatformer {
            move_speed: *move_speed,
            jump_impulse: *jump_impulse,
            jump_cooldown: *jump_cooldown,
            tag: tag.clone(),
        },
        ecs::behavior::Behavior::PlayerTopDown { move_speed, tag } => {
            BehaviorData::PlayerTopDown {
                move_speed: *move_speed,
                tag: tag.clone(),
            }
        }
        ecs::behavior::Behavior::FollowEntity {
            target_name,
            follow_distance,
            follow_speed,
        } => BehaviorData::FollowEntity {
            target_name: target_name.clone(),
            follow_distance: *follow_distance,
            follow_speed: *follow_speed,
        },
        ecs::behavior::Behavior::FollowTagged {
            target_tag,
            follow_distance,
            follow_speed,
        } => BehaviorData::FollowTagged {
            target_tag: target_tag.clone(),
            follow_distance: *follow_distance,
            follow_speed: *follow_speed,
        },
        ecs::behavior::Behavior::Patrol {
            point_a,
            point_b,
            speed,
            wait_time,
        } => BehaviorData::Patrol {
            point_a: *point_a,
            point_b: *point_b,
            speed: *speed,
            wait_time: *wait_time,
        },
        ecs::behavior::Behavior::Collectible {
            score_value,
            despawn_on_collect,
            collector_tag,
        } => BehaviorData::Collectible {
            score_value: *score_value,
            despawn_on_collect: *despawn_on_collect,
            collector_tag: collector_tag.clone(),
        },
        ecs::behavior::Behavior::ChaseTagged {
            target_tag,
            detection_range,
            chase_speed,
            lose_interest_range,
        } => BehaviorData::ChaseTagged {
            target_tag: target_tag.clone(),
            detection_range: *detection_range,
            chase_speed: *chase_speed,
            lose_interest_range: *lose_interest_range,
        },
    }
}

/// Serialize SceneData to a pretty-printed RON string.
pub fn serialize_to_ron(scene: &SceneData) -> Result<String, String> {
    ron::ser::to_string_pretty(scene, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("RON serialization error: {}", e))
}

/// Write SceneData to a file as RON.
pub fn save_scene_to_file(scene: &SceneData, path: &Path) -> Result<(), String> {
    let ron_string = serialize_to_ron(scene)?;
    std::fs::write(path, ron_string).map_err(|e| format!("Failed to write scene file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec4};

    /// Default texture path function for tests: handle 0 → "#white", others → "#texture_{id}"
    fn test_texture_path(handle: u32) -> String {
        if handle == 0 {
            "#white".to_string()
        } else {
            format!("#texture_{}", handle)
        }
    }

    #[test]
    fn test_empty_world_produces_empty_scene() {
        let world = World::new();
        let scene = world_to_scene_data(&world, "Empty", None, &test_texture_path);

        assert_eq!(scene.name, "Empty");
        assert!(scene.entities.is_empty());
        assert!(scene.physics.is_none());
        assert!(scene.prefabs.is_empty());
    }

    #[test]
    fn test_single_entity_with_transform() {
        let mut world = World::new();
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
            .ok();

        let scene = world_to_scene_data(&world, "TransformTest", None, &test_texture_path);

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
            other => panic!("Expected Transform2D, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_name() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, Name::new("Player")).ok();

        let scene = world_to_scene_data(&world, "NameTest", None, &test_texture_path);

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, Some("Player".to_string()));
        // Name is not a component variant — it goes into EntityData.name
        assert!(scene.entities[0].components.is_empty());
    }

    #[test]
    fn test_entity_with_sprite() {
        let mut world = World::new();
        let entity = world.create_entity();
        let sprite = Sprite {
            texture_handle: 5,
            offset: Vec2::new(1.0, 2.0),
            rotation: 0.5,
            scale: Vec2::new(3.0, 4.0),
            color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            depth: 10.0,
            tex_region: [0.0, 0.0, 1.0, 1.0],
        };
        world.add_component(&entity, sprite).ok();

        let scene = world_to_scene_data(&world, "SpriteTest", None, &test_texture_path);

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Sprite {
                texture,
                offset,
                rotation,
                scale,
                color,
                depth,
            } => {
                assert_eq!(texture, "#texture_5");
                assert_eq!(*offset, (1.0, 2.0));
                assert_eq!(*rotation, 0.5);
                assert_eq!(*scale, (3.0, 4.0));
                assert_eq!(*color, (1.0, 0.0, 0.0, 1.0));
                assert_eq!(*depth, 10.0);
            }
            other => panic!("Expected Sprite, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_camera() {
        let mut world = World::new();
        let entity = world.create_entity();
        let camera = Camera {
            position: Vec2::new(50.0, 60.0),
            rotation: 0.1,
            zoom: 2.0,
            viewport_size: Vec2::new(1920.0, 1080.0),
            is_main_camera: true,
            near: -1000.0,
            far: 1000.0,
        };
        world.add_component(&entity, camera).ok();

        let scene = world_to_scene_data(&world, "CameraTest", None, &test_texture_path);

        assert_eq!(scene.entities[0].components.len(), 1);

        match &scene.entities[0].components[0] {
            ComponentData::Camera2D {
                position,
                rotation,
                zoom,
                viewport_size,
                is_main_camera,
            } => {
                assert_eq!(*position, (50.0, 60.0));
                assert_eq!(*rotation, 0.1);
                assert_eq!(*zoom, 2.0);
                assert_eq!(*viewport_size, (1920.0, 1080.0));
                assert!(*is_main_camera);
            }
            other => panic!("Expected Camera2D, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_sprite_animation() {
        let mut world = World::new();
        let entity = world.create_entity();
        let anim = SpriteAnimation {
            fps: 12.0,
            frames: vec![[0.0, 0.0, 0.25, 1.0], [0.25, 0.0, 0.25, 1.0]],
            playing: true,
            loop_animation: false,
            current_frame: 0,
            time_accumulator: 0.0,
        };
        world.add_component(&entity, anim).ok();

        let scene = world_to_scene_data(&world, "AnimTest", None, &test_texture_path);

        match &scene.entities[0].components[0] {
            ComponentData::SpriteAnimation {
                fps,
                frames,
                playing,
                loop_animation,
            } => {
                assert_eq!(*fps, 12.0);
                assert_eq!(frames.len(), 2);
                assert_eq!(frames[0], (0.0, 0.0, 0.25, 1.0));
                assert_eq!(frames[1], (0.25, 0.0, 0.25, 1.0));
                assert!(*playing);
                assert!(!*loop_animation);
            }
            other => panic!("Expected SpriteAnimation, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_rigid_body() {
        let mut world = World::new();
        let entity = world.create_entity();
        let rb = physics::components::RigidBody::new_dynamic()
            .with_velocity(Vec2::new(10.0, 20.0))
            .with_angular_velocity(0.5)
            .with_gravity_scale(0.8)
            .with_linear_damping(5.0)
            .with_angular_damping(1.0)
            .with_rotation_locked(true)
            .with_ccd(true);
        world.add_component(&entity, rb).ok();

        let scene = world_to_scene_data(&world, "RBTest", None, &test_texture_path);

        match &scene.entities[0].components[0] {
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
                assert_eq!(*body_type, RigidBodyTypeData::Dynamic);
                assert_eq!(*velocity, (10.0, 20.0));
                assert_eq!(*angular_velocity, 0.5);
                assert_eq!(*gravity_scale, 0.8);
                assert_eq!(*linear_damping, 5.0);
                assert_eq!(*angular_damping, 1.0);
                assert!(!*can_rotate);
                assert!(*ccd_enabled);
            }
            other => panic!("Expected RigidBody, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_collider() {
        let mut world = World::new();
        let entity = world.create_entity();
        let col = physics::components::Collider::new(
            physics::components::ColliderShape::Circle { radius: 25.0 },
        )
        .with_offset(Vec2::new(5.0, 10.0))
        .as_sensor()
        .with_friction(0.3)
        .with_restitution(0.7);
        world.add_component(&entity, col).ok();

        let scene = world_to_scene_data(&world, "ColTest", None, &test_texture_path);

        match &scene.entities[0].components[0] {
            ComponentData::Collider {
                shape,
                offset,
                is_sensor,
                friction,
                restitution,
            } => {
                match shape {
                    ColliderShapeData::Circle { radius } => assert_eq!(*radius, 25.0),
                    other => panic!("Expected Circle, got {:?}", other),
                }
                assert_eq!(*offset, (5.0, 10.0));
                assert!(*is_sensor);
                assert_eq!(*friction, 0.3);
                assert_eq!(*restitution, 0.7);
            }
            other => panic!("Expected Collider, got {:?}", other),
        }
    }

    #[test]
    fn test_entity_with_behavior() {
        let mut world = World::new();
        let entity = world.create_entity();
        let behavior = ecs::behavior::Behavior::PlayerPlatformer {
            move_speed: 200.0,
            jump_impulse: 500.0,
            jump_cooldown: 0.25,
            tag: "hero".to_string(),
        };
        world.add_component(&entity, behavior).ok();

        let scene = world_to_scene_data(&world, "BehaviorTest", None, &test_texture_path);

        match &scene.entities[0].components[0] {
            ComponentData::Behavior(BehaviorData::PlayerPlatformer {
                move_speed,
                jump_impulse,
                jump_cooldown,
                tag,
            }) => {
                assert_eq!(*move_speed, 200.0);
                assert_eq!(*jump_impulse, 500.0);
                assert_eq!(*jump_cooldown, 0.25);
                assert_eq!(tag, "hero");
            }
            other => panic!("Expected Behavior::PlayerPlatformer, got {:?}", other),
        }
    }

    #[test]
    fn test_hierarchy_preserved() {
        let mut world = World::new();
        let parent = world.create_entity();
        let child = world.create_entity();

        world.add_component(&parent, Name::new("Parent")).ok();
        world
            .add_component(&parent, Transform2D::new(Vec2::new(10.0, 20.0)))
            .ok();

        world.add_component(&child, Name::new("Child")).ok();
        world
            .add_component(&child, Transform2D::new(Vec2::new(30.0, 40.0)))
            .ok();

        world.set_parent(child, parent).unwrap();

        let scene = world_to_scene_data(&world, "HierarchyTest", None, &test_texture_path);

        // Only root entities at the top level
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, Some("Parent".to_string()));
        assert_eq!(scene.entities[0].children.len(), 1);
        assert_eq!(
            scene.entities[0].children[0].name,
            Some("Child".to_string())
        );

        // Verify child has its own components
        assert_eq!(scene.entities[0].children[0].components.len(), 1);
        match &scene.entities[0].children[0].components[0] {
            ComponentData::Transform2D { position, .. } => {
                assert_eq!(*position, (30.0, 40.0));
            }
            other => panic!("Expected Transform2D, got {:?}", other),
        }
    }

    #[test]
    fn test_physics_settings_included() {
        let world = World::new();
        let settings = PhysicsSettings {
            gravity: (0.0, -500.0),
            pixels_per_meter: 50.0,
            timestep: 1.0 / 120.0,
        };

        let scene =
            world_to_scene_data(&world, "PhysicsTest", Some(settings), &test_texture_path);

        let physics = scene.physics.unwrap();
        assert_eq!(physics.gravity, (0.0, -500.0));
        assert_eq!(physics.pixels_per_meter, 50.0);
    }

    #[test]
    fn test_global_transform_not_serialized() {
        let mut world = World::new();
        let entity = world.create_entity();
        world
            .add_component(
                &entity,
                ecs::hierarchy::GlobalTransform2D::default(),
            )
            .ok();

        let scene = world_to_scene_data(&world, "SkipTest", None, &test_texture_path);

        assert_eq!(scene.entities.len(), 1);
        // GlobalTransform2D should not appear as a ComponentData variant
        assert!(scene.entities[0].components.is_empty());
    }

    #[test]
    fn test_serialize_to_ron_valid() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(&entity, Name::new("TestEntity")).ok();
        world
            .add_component(&entity, Transform2D::new(Vec2::new(42.0, 99.0)))
            .ok();

        let scene = world_to_scene_data(&world, "RONTest", None, &test_texture_path);
        let ron_string = serialize_to_ron(&scene).expect("Serialization should succeed");

        // Verify the RON string can be parsed back
        let parsed: SceneData = ron::from_str(&ron_string).expect("Should parse back");
        assert_eq!(parsed.name, "RONTest");
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.entities[0].name, Some("TestEntity".to_string()));
    }

    #[test]
    fn test_save_to_file_creates_file() {
        let mut world = World::new();
        let entity = world.create_entity();
        world
            .add_component(&entity, Transform2D::new(Vec2::new(1.0, 2.0)))
            .ok();

        let scene = world_to_scene_data(&world, "FileTest", None, &test_texture_path);

        let dir = std::env::temp_dir().join("insiculous_test_scene_serializer");
        std::fs::create_dir_all(&dir).ok();
        let file_path = dir.join("test_scene.scene.ron");

        save_scene_to_file(&scene, &file_path).expect("Should write file");
        assert!(file_path.exists());

        // Read it back and verify
        let content = std::fs::read_to_string(&file_path).unwrap();
        let parsed: SceneData = ron::from_str(&content).expect("Should parse");
        assert_eq!(parsed.name, "FileTest");
        assert_eq!(parsed.entities.len(), 1);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_roundtrip_serialize_deserialize() {
        let mut world = World::new();

        // Create an entity with multiple components
        let entity = world.create_entity();
        world.add_component(&entity, Name::new("RoundTrip")).ok();
        world
            .add_component(
                &entity,
                Transform2D {
                    position: Vec2::new(100.0, 200.0),
                    rotation: 0.5,
                    scale: Vec2::new(2.0, 2.0),
                },
            )
            .ok();
        world
            .add_component(
                &entity,
                Sprite {
                    texture_handle: 0,
                    offset: Vec2::new(5.0, 10.0),
                    rotation: 0.1,
                    scale: Vec2::new(1.5, 1.5),
                    color: Vec4::new(0.5, 0.6, 0.7, 1.0),
                    depth: 5.0,
                    tex_region: [0.0, 0.0, 1.0, 1.0],
                },
            )
            .ok();

        let scene = world_to_scene_data(&world, "RoundTrip", None, &test_texture_path);
        let ron_string = serialize_to_ron(&scene).expect("Serialize should succeed");
        let parsed: SceneData = ron::from_str(&ron_string).expect("Parse should succeed");

        assert_eq!(parsed.name, "RoundTrip");
        assert_eq!(parsed.entities.len(), 1);
        assert_eq!(parsed.entities[0].name, Some("RoundTrip".to_string()));
        assert_eq!(parsed.entities[0].components.len(), 2);

        // Verify Transform2D
        match &parsed.entities[0].components[0] {
            ComponentData::Transform2D {
                position,
                rotation,
                scale,
            } => {
                assert_eq!(*position, (100.0, 200.0));
                assert_eq!(*rotation, 0.5);
                assert_eq!(*scale, (2.0, 2.0));
            }
            other => panic!("Expected Transform2D, got {:?}", other),
        }

        // Verify Sprite
        match &parsed.entities[0].components[1] {
            ComponentData::Sprite {
                texture,
                offset,
                color,
                depth,
                ..
            } => {
                assert_eq!(texture, "#white");
                assert_eq!(*offset, (5.0, 10.0));
                assert_eq!(*color, (0.5, 0.6, 0.7, 1.0));
                assert_eq!(*depth, 5.0);
            }
            other => panic!("Expected Sprite, got {:?}", other),
        }
    }

    #[test]
    fn test_default_texture_path() {
        assert_eq!(test_texture_path(0), "#white");
        assert_eq!(test_texture_path(1), "#texture_1");
        assert_eq!(test_texture_path(42), "#texture_42");
    }

    #[test]
    fn test_multiple_entities_ordering() {
        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(&e1, Name::new("First")).ok();
        world.add_component(&e2, Name::new("Second")).ok();
        world.add_component(&e3, Name::new("Third")).ok();

        let scene = world_to_scene_data(&world, "MultiTest", None, &test_texture_path);

        assert_eq!(scene.entities.len(), 3);

        // All three entities should be present (order depends on get_root_entities)
        let names: Vec<Option<String>> = scene.entities.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&Some("First".to_string())));
        assert!(names.contains(&Some("Second".to_string())));
        assert!(names.contains(&Some("Third".to_string())));
    }

    #[test]
    fn test_collider_box_shape() {
        let mut world = World::new();
        let entity = world.create_entity();
        let col = physics::components::Collider::new(
            physics::components::ColliderShape::Box {
                half_extents: Vec2::new(40.0, 20.0),
            },
        );
        world.add_component(&entity, col).ok();

        let scene = world_to_scene_data(&world, "BoxTest", None, &test_texture_path);

        match &scene.entities[0].components[0] {
            ComponentData::Collider { shape, .. } => match shape {
                ColliderShapeData::Box { half_extents } => {
                    assert_eq!(*half_extents, (40.0, 20.0));
                }
                other => panic!("Expected Box shape, got {:?}", other),
            },
            other => panic!("Expected Collider, got {:?}", other),
        }
    }

    #[test]
    fn test_rigid_body_static_and_kinematic() {
        let mut world = World::new();

        let static_e = world.create_entity();
        world
            .add_component(&static_e, physics::components::RigidBody::new_static())
            .ok();

        let kinematic_e = world.create_entity();
        world
            .add_component(
                &kinematic_e,
                physics::components::RigidBody::new_kinematic(),
            )
            .ok();

        let scene = world_to_scene_data(&world, "BodyTypes", None, &test_texture_path);

        // Collect body types from all entities
        let body_types: Vec<RigidBodyTypeData> = scene
            .entities
            .iter()
            .flat_map(|e| e.components.iter())
            .filter_map(|c| match c {
                ComponentData::RigidBody { body_type, .. } => Some(*body_type),
                _ => None,
            })
            .collect();

        assert!(body_types.contains(&RigidBodyTypeData::Static));
        assert!(body_types.contains(&RigidBodyTypeData::Kinematic));
    }

    #[test]
    fn test_deep_hierarchy_preserved() {
        let mut world = World::new();
        let grandparent = world.create_entity();
        let parent = world.create_entity();
        let child = world.create_entity();

        world.add_component(&grandparent, Name::new("GP")).ok();
        world.add_component(&parent, Name::new("P")).ok();
        world.add_component(&child, Name::new("C")).ok();

        world.set_parent(parent, grandparent).unwrap();
        world.set_parent(child, parent).unwrap();

        let scene = world_to_scene_data(&world, "DeepHierarchy", None, &test_texture_path);

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, Some("GP".to_string()));
        assert_eq!(scene.entities[0].children.len(), 1);
        assert_eq!(
            scene.entities[0].children[0].name,
            Some("P".to_string())
        );
        assert_eq!(scene.entities[0].children[0].children.len(), 1);
        assert_eq!(
            scene.entities[0].children[0].children[0].name,
            Some("C".to_string())
        );
    }
}
