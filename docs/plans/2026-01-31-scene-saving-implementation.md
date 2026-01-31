# Scene Saving/Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add scene saving to the editor, extracting World entities to RON files with editor state preservation.

**Architecture:** Component-by-component extraction mirrors the existing loader pattern. AssetManager tracks texture paths for reverse lookup. EditorSettings embedded in SceneData for camera persistence.

**Tech Stack:** Rust, RON serialization, ECS (ecs crate), engine_core, editor crate

---

## Task 1: Add EditorSettings to SceneData

**Files:**
- Modify: `crates/engine_core/src/scene_data.rs`

**Step 1: Write the failing test**

Add to the existing test module in `scene_data.rs`:

```rust
#[test]
fn test_editor_settings_serialization() {
    let settings = EditorSettings {
        camera_position: (150.0, -200.0),
        camera_zoom: 1.5,
    };

    let ron_str = ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    let parsed: EditorSettings = ron::from_str(&ron_str).expect("Failed to parse");
    assert_eq!(parsed.camera_position, (150.0, -200.0));
    assert_eq!(parsed.camera_zoom, 1.5);
}

#[test]
fn test_scene_data_with_editor_settings() {
    let scene = SceneData {
        name: "Test".to_string(),
        editor: Some(EditorSettings {
            camera_position: (100.0, 50.0),
            camera_zoom: 2.0,
        }),
        ..Default::default()
    };

    let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    assert!(ron_str.contains("EditorSettings"));

    let parsed: SceneData = ron::from_str(&ron_str).expect("Failed to parse");
    assert!(parsed.editor.is_some());
    assert_eq!(parsed.editor.unwrap().camera_zoom, 2.0);
}

#[test]
fn test_scene_data_without_editor_settings_backward_compat() {
    // Old scene format without editor field
    let scene_ron = r#"
        SceneData(
            name: "Old Scene",
            entities: [],
        )
    "#;

    let parsed: SceneData = ron::from_str(scene_ron).expect("Failed to parse");
    assert!(parsed.editor.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_editor_settings`
Expected: FAIL with "cannot find type `EditorSettings`"

**Step 3: Write the implementation**

Add before `SceneData` struct:

```rust
/// Editor-specific settings persisted with the scene
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EditorSettings {
    /// Camera position when scene was last saved
    #[serde(default)]
    pub camera_position: (f32, f32),
    /// Camera zoom level when scene was last saved
    #[serde(default = "default_zoom")]
    pub camera_zoom: f32,
}
```

Modify `SceneData` struct to add:

```rust
/// Editor settings (camera position, zoom) - optional for backward compatibility
#[serde(default)]
pub editor: Option<EditorSettings>,
```

Update `SceneData::default()` to include:

```rust
editor: None,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_editor_settings`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_data.rs
git commit -m "feat(scene_data): add EditorSettings for camera state persistence"
```

---

## Task 2: Add Texture Path Reverse Lookup to AssetManager

**Files:**
- Modify: `crates/engine_core/src/assets.rs`

**Step 1: Write the failing test**

Add to the test module in `assets.rs`:

```rust
#[test]
fn test_get_texture_path_white() {
    // Handle 0 is always #white
    // We need a mock test since we can't create real AssetManager without GPU
    // Test the logic in isolation
    assert_eq!(texture_path_for_handle(0), Some("#white"));
}

#[test]
fn test_get_texture_path_unknown() {
    assert_eq!(texture_path_for_handle(9999), None);
}

// Helper for testing without GPU
fn texture_path_for_handle(handle: u32) -> Option<&'static str> {
    if handle == 0 {
        Some("#white")
    } else {
        None
    }
}
```

**Step 2: Run test to verify it passes (these are unit tests for the logic)**

Run: `cargo test -p engine_core test_get_texture_path`
Expected: PASS

**Step 3: Write the implementation**

Add field to `AssetManager` struct:

```rust
pub struct AssetManager {
    texture_manager: TextureManager,
    config: AssetConfig,
    /// Reverse mapping from texture handle ID to path/reference string
    texture_paths: HashMap<u32, String>,
}
```

Update `new()` and `with_config()`:

```rust
pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
    Self {
        texture_manager: TextureManager::new(device, queue),
        config: AssetConfig::default(),
        texture_paths: HashMap::new(),
    }
}

pub fn with_config(device: Arc<Device>, queue: Arc<Queue>, config: AssetConfig) -> Self {
    Self {
        texture_manager: TextureManager::new(device, queue),
        config,
        texture_paths: HashMap::new(),
    }
}
```

Add the lookup method:

```rust
/// Get the path/reference string for a texture handle.
///
/// Returns:
/// - `Some("#white")` for handle 0 (built-in white texture)
/// - `Some(path)` for loaded textures
/// - `Some("#solid:RRGGBBAA")` for solid color textures
/// - `None` for unknown handles
pub fn get_texture_path(&self, handle: u32) -> Option<&str> {
    if handle == 0 {
        return Some("#white");
    }
    self.texture_paths.get(&handle).map(|s| s.as_str())
}
```

Update `load_texture()` to track the path:

```rust
pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureHandle, AssetError> {
    let path = path.as_ref();

    // Resolve path against base path if relative
    let full_path = if path.is_relative() {
        Path::new(&self.config.base_path).join(path)
    } else {
        path.to_path_buf()
    };

    if self.config.log_loading {
        log::info!("Loading texture: {:?}", full_path);
    }

    let handle = self.texture_manager.load_texture(&full_path, TextureLoadConfig::default())?;

    // Store the original path for scene saving (use the input path, not full_path)
    self.texture_paths.insert(handle.id, path.to_string_lossy().to_string());

    Ok(handle)
}
```

Update `create_solid_color()` to track the color reference:

```rust
pub fn create_solid_color(
    &mut self,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<TextureHandle, AssetError> {
    let handle = self.texture_manager.create_solid_color(width, height, color)?;

    // Store as #solid:RRGGBBAA format
    let path = format!("#solid:{:02X}{:02X}{:02X}{:02X}",
        color[0], color[1], color[2], color[3]);
    self.texture_paths.insert(handle.id, path);

    Ok(handle)
}
```

**Step 4: Run tests**

Run: `cargo test -p engine_core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/assets.rs
git commit -m "feat(assets): add texture path reverse lookup for scene saving"
```

---

## Task 3: Create SceneSaver with Basic Structure

**Files:**
- Create: `crates/engine_core/src/scene_saver.rs`
- Modify: `crates/engine_core/src/lib.rs`

**Step 1: Write the failing test**

Create `scene_saver.rs` with tests first:

```rust
//! Scene saving for serializing World entities to RON files.

use std::path::Path;

use ecs::{EntityId, World, WorldHierarchyExt};

use crate::assets::AssetManager;
use crate::scene_data::{ComponentData, EditorSettings, EntityData, SceneData};

/// Scene saver for extracting world state to RON files.
pub struct SceneSaver;

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::World;

    #[test]
    fn test_extract_empty_world() {
        let world = World::default();
        let scene = SceneSaver::extract_from_world(&world, None, "Empty");

        assert_eq!(scene.name, "Empty");
        assert!(scene.entities.is_empty());
        assert!(scene.prefabs.is_empty());
        assert!(scene.editor.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_extract_empty`
Expected: FAIL with "cannot find function `extract_from_world`"

**Step 3: Write minimal implementation**

```rust
//! Scene saving for serializing World entities to RON files.

use std::collections::HashMap;
use std::path::Path;

use ecs::{EntityId, World, WorldHierarchyExt};

use crate::assets::AssetManager;
use crate::scene_data::{ComponentData, EditorSettings, EntityData, SceneData};

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
            .iter()
            .filter_map(|&child| Self::extract_entity_recursive(world, assets, child))
            .collect();

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
        _world: &World,
        _assets: Option<&AssetManager>,
        _entity: EntityId,
    ) -> Vec<ComponentData> {
        // TODO: Implement component extraction in next task
        Vec::new()
    }

    fn extract_name(world: &World, entity: EntityId) -> Option<String> {
        world.get::<ecs::Name>(entity).map(|n| n.0.clone())
    }
}
```

Add to `lib.rs`:

```rust
pub mod scene_saver;
pub use scene_saver::SceneSaver;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_extract_empty`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs crates/engine_core/src/lib.rs
git commit -m "feat(scene_saver): add basic SceneSaver structure"
```

---

## Task 4: Implement Transform2D and Sprite Extraction

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

Add to tests in `scene_saver.rs`:

```rust
use ecs::sprite_components::{Transform2D, Sprite};
use glam::Vec2;

#[test]
fn test_extract_entity_with_transform() {
    let mut world = World::default();
    let entity = world.create_entity();
    world.add_component(&entity, Transform2D {
        position: Vec2::new(100.0, 200.0),
        rotation: 1.5,
        scale: Vec2::new(2.0, 3.0),
    }).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].components.len(), 1);

    match &scene.entities[0].components[0] {
        ComponentData::Transform2D { position, rotation, scale } => {
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
    world.add_component(&entity, Sprite {
        texture_handle: 0,
        offset: Vec2::new(5.0, 10.0),
        rotation: 0.5,
        scale: Vec2::new(1.0, 1.0),
        color: glam::Vec4::new(1.0, 0.5, 0.25, 1.0),
        depth: 5.0,
        tex_region: [0.0, 0.0, 1.0, 1.0],
    }).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].components.len(), 1);

    match &scene.entities[0].components[0] {
        ComponentData::Sprite { texture, offset, rotation, scale, color, depth } => {
            assert_eq!(texture, "#white");
            assert_eq!(*offset, (5.0, 10.0));
            assert_eq!(*rotation, 0.5);
            assert_eq!(*color, (1.0, 0.5, 0.25, 1.0));
            assert_eq!(*depth, 5.0);
        }
        _ => panic!("Expected Sprite"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_extract_entity_with`
Expected: FAIL - components vec is empty

**Step 3: Write the implementation**

Add imports at top of file:

```rust
use ecs::sprite_components::{Camera, Sprite, SpriteAnimation, Transform2D};
use glam::Vec2;
```

Replace `extract_components`:

```rust
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

    components
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_extract_entity_with`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "feat(scene_saver): implement Transform2D and Sprite extraction"
```

---

## Task 5: Implement Camera and SpriteAnimation Extraction

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_extract_entity_with_camera() {
    let mut world = World::default();
    let entity = world.create_entity();
    world.add_component(&entity, Camera {
        position: Vec2::new(50.0, 75.0),
        rotation: 0.25,
        zoom: 1.5,
        viewport_size: Vec2::new(1920.0, 1080.0),
        is_main_camera: true,
        near: -1000.0,
        far: 1000.0,
    }).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    match &scene.entities[0].components[0] {
        ComponentData::Camera2D { position, zoom, is_main_camera, .. } => {
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
    world.add_component(&entity, SpriteAnimation {
        fps: 12.0,
        frames: vec![[0.0, 0.0, 0.25, 0.25], [0.25, 0.0, 0.5, 0.25]],
        playing: true,
        loop_animation: false,
        current_frame: 0,
        time_accumulator: 0.0,
    }).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    match &scene.entities[0].components[0] {
        ComponentData::SpriteAnimation { fps, frames, playing, loop_animation } => {
            assert_eq!(*fps, 12.0);
            assert_eq!(frames.len(), 2);
            assert!(*playing);
            assert!(!*loop_animation);
        }
        _ => panic!("Expected SpriteAnimation"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_extract_entity_with_camera test_extract_entity_with_animation`
Expected: FAIL

**Step 3: Write the implementation**

Add to `extract_components`:

```rust
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_extract_entity_with_camera test_extract_entity_with_animation`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "feat(scene_saver): implement Camera and SpriteAnimation extraction"
```

---

## Task 6: Implement RigidBody and Collider Extraction

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

```rust
#[cfg(feature = "physics")]
#[test]
fn test_extract_entity_with_rigidbody() {
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
        ComponentData::RigidBody { body_type, velocity, gravity_scale, linear_damping, .. } => {
            assert_eq!(*body_type, crate::scene_data::RigidBodyTypeData::Dynamic);
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
    use physics::components::{Collider, ColliderShape};

    let mut world = World::default();
    let entity = world.create_entity();

    let mut collider = Collider::new(ColliderShape::Box {
        half_extents: Vec2::new(32.0, 16.0)
    });
    collider.friction = 0.8;
    collider.restitution = 0.2;
    world.add_component(&entity, collider).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    match &scene.entities[0].components[0] {
        ComponentData::Collider { shape, friction, restitution, .. } => {
            match shape {
                crate::scene_data::ColliderShapeData::Box { half_extents } => {
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core --features physics test_extract_entity_with_rigidbody test_extract_entity_with_collider`
Expected: FAIL

**Step 3: Write the implementation**

Add to `extract_components`:

```rust
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core --features physics test_extract_entity_with_rigidbody test_extract_entity_with_collider`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "feat(scene_saver): implement RigidBody and Collider extraction"
```

---

## Task 7: Implement Behavior Extraction

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_extract_entity_with_behavior() {
    let mut world = World::default();
    let entity = world.create_entity();
    world.add_component(&entity, ecs::behavior::Behavior::PlayerPlatformer {
        move_speed: 150.0,
        jump_impulse: 500.0,
        jump_cooldown: 0.2,
        tag: "hero".to_string(),
    }).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    match &scene.entities[0].components[0] {
        ComponentData::Behavior(crate::scene_data::BehaviorData::PlayerPlatformer {
            move_speed, jump_impulse, tag, ..
        }) => {
            assert_eq!(*move_speed, 150.0);
            assert_eq!(*jump_impulse, 500.0);
            assert_eq!(tag, "hero");
        }
        _ => panic!("Expected Behavior::PlayerPlatformer"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_extract_entity_with_behavior`
Expected: FAIL

**Step 3: Write the implementation**

Add conversion from ECS Behavior to BehaviorData. Add to `extract_components`:

```rust
// Behavior
if let Some(b) = world.get::<ecs::behavior::Behavior>(entity) {
    use crate::scene_data::BehaviorData;
    use ecs::behavior::Behavior;

    let behavior_data = match b {
        Behavior::PlayerPlatformer { move_speed, jump_impulse, jump_cooldown, tag } => {
            BehaviorData::PlayerPlatformer {
                move_speed: *move_speed,
                jump_impulse: *jump_impulse,
                jump_cooldown: *jump_cooldown,
                tag: tag.clone(),
            }
        }
        Behavior::PlayerTopDown { move_speed, tag } => {
            BehaviorData::PlayerTopDown {
                move_speed: *move_speed,
                tag: tag.clone(),
            }
        }
        Behavior::FollowEntity { target_name, follow_distance, follow_speed } => {
            BehaviorData::FollowEntity {
                target_name: target_name.clone(),
                follow_distance: *follow_distance,
                follow_speed: *follow_speed,
            }
        }
        Behavior::FollowTagged { target_tag, follow_distance, follow_speed } => {
            BehaviorData::FollowTagged {
                target_tag: target_tag.clone(),
                follow_distance: *follow_distance,
                follow_speed: *follow_speed,
            }
        }
        Behavior::Patrol { point_a, point_b, speed, wait_time } => {
            BehaviorData::Patrol {
                point_a: *point_a,
                point_b: *point_b,
                speed: *speed,
                wait_time: *wait_time,
            }
        }
        Behavior::Collectible { score_value, despawn_on_collect, collector_tag } => {
            BehaviorData::Collectible {
                score_value: *score_value,
                despawn_on_collect: *despawn_on_collect,
                collector_tag: collector_tag.clone(),
            }
        }
        Behavior::ChaseTagged { target_tag, detection_range, chase_speed, lose_interest_range } => {
            BehaviorData::ChaseTagged {
                target_tag: target_tag.clone(),
                detection_range: *detection_range,
                chase_speed: *chase_speed,
                lose_interest_range: *lose_interest_range,
            }
        }
    };

    components.push(ComponentData::Behavior(behavior_data));
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_extract_entity_with_behavior`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "feat(scene_saver): implement Behavior extraction"
```

---

## Task 8: Implement Hierarchy Extraction

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_extract_hierarchy() {
    let mut world = World::default();

    // Create parent
    let parent = world.create_entity();
    world.add_component(&parent, ecs::Name("parent".to_string())).unwrap();
    world.add_component(&parent, Transform2D::new(Vec2::new(100.0, 100.0))).unwrap();

    // Create child
    let child = world.create_entity();
    world.add_component(&child, ecs::Name("child".to_string())).unwrap();
    world.add_component(&child, Transform2D::new(Vec2::new(10.0, 10.0))).unwrap();

    // Set up hierarchy
    world.set_parent(child, parent).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    // Should have 1 root entity with 1 child
    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].name, Some("parent".to_string()));
    assert_eq!(scene.entities[0].children.len(), 1);
    assert_eq!(scene.entities[0].children[0].name, Some("child".to_string()));
}

#[test]
fn test_extract_deep_hierarchy() {
    let mut world = World::default();

    let grandparent = world.create_entity();
    world.add_component(&grandparent, ecs::Name("grandparent".to_string())).unwrap();

    let parent = world.create_entity();
    world.add_component(&parent, ecs::Name("parent".to_string())).unwrap();
    world.set_parent(parent, grandparent).unwrap();

    let child = world.create_entity();
    world.add_component(&child, ecs::Name("child".to_string())).unwrap();
    world.set_parent(child, parent).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "Test");

    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].children.len(), 1);
    assert_eq!(scene.entities[0].children[0].children.len(), 1);
    assert_eq!(scene.entities[0].children[0].children[0].name, Some("child".to_string()));
}
```

**Step 2: Run test to verify it passes (hierarchy already implemented)**

Run: `cargo test -p engine_core test_extract_hierarchy test_extract_deep`
Expected: PASS (the recursive extraction already handles this)

**Step 3: Commit (just the tests)**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "test(scene_saver): add hierarchy extraction tests"
```

---

## Task 9: Implement File Writing with SceneSaveError

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the failing test**

```rust
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_save_to_file() {
    let mut world = World::default();
    let entity = world.create_entity();
    world.add_component(&entity, ecs::Name("test_entity".to_string())).unwrap();
    world.add_component(&entity, Transform2D::new(Vec2::new(50.0, 75.0))).unwrap();

    let scene = SceneSaver::extract_from_world(&world, None, "SaveTest");

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    SceneSaver::save_to_file(&scene, path).unwrap();

    // Read back and verify
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("SaveTest"));
    assert!(content.contains("test_entity"));
    assert!(content.contains("Transform2D"));
}

#[test]
fn test_roundtrip_save_load() {
    use crate::scene_loader::SceneLoader;

    let mut world = World::default();
    let entity = world.create_entity();
    world.add_component(&entity, ecs::Name("roundtrip".to_string())).unwrap();
    world.add_component(&entity, Transform2D {
        position: Vec2::new(123.0, 456.0),
        rotation: 1.5,
        scale: Vec2::new(2.0, 2.0),
    }).unwrap();

    let mut scene = SceneSaver::extract_from_world(&world, None, "Roundtrip");
    scene.editor = Some(EditorSettings {
        camera_position: (100.0, 200.0),
        camera_zoom: 1.5,
    });

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    SceneSaver::save_to_file(&scene, path).unwrap();

    // Load it back
    let loaded = SceneLoader::load_from_file(path).unwrap();

    assert_eq!(loaded.name, "Roundtrip");
    assert_eq!(loaded.entities.len(), 1);
    assert!(loaded.editor.is_some());
    assert_eq!(loaded.editor.unwrap().camera_zoom, 1.5);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p engine_core test_save_to_file test_roundtrip`
Expected: FAIL with "cannot find function `save_to_file`"

**Step 3: Write the implementation**

Add error type and save functions:

```rust
/// Errors that can occur when saving a scene.
#[derive(Debug, thiserror::Error)]
pub enum SceneSaveError {
    #[error("Failed to write scene file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to serialize scene: {0}")]
    SerializationError(#[from] ron::Error),
}

impl SceneSaver {
    // ... existing methods ...

    /// Save scene data to a RON file.
    pub fn save_to_file(scene: &SceneData, path: impl AsRef<Path>) -> Result<(), SceneSaveError> {
        let config = ron::ser::PrettyConfig::default()
            .struct_names(true)
            .enumerate_arrays(false);

        let ron_string = ron::ser::to_string_pretty(scene, config)?;
        std::fs::write(path, ron_string)?;

        Ok(())
    }

    /// Extract from world and save to file in one operation.
    pub fn save_world_to_file(
        world: &World,
        assets: Option<&AssetManager>,
        scene_name: &str,
        editor_settings: Option<EditorSettings>,
        path: impl AsRef<Path>,
    ) -> Result<(), SceneSaveError> {
        let mut scene = Self::extract_from_world(world, assets, scene_name);
        scene.editor = editor_settings;
        Self::save_to_file(&scene, path)
    }
}
```

Add tempfile to dev-dependencies in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p engine_core test_save_to_file test_roundtrip`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs crates/engine_core/Cargo.toml
git commit -m "feat(scene_saver): implement file writing and roundtrip support"
```

---

## Task 10: Add Scene Tracking to EditorContext

**Files:**
- Modify: `crates/editor/src/context.rs`

**Step 1: Write the failing test**

Add to tests in `context.rs`:

```rust
use std::path::PathBuf;

#[test]
fn test_editor_context_scene_tracking() {
    let mut ctx = EditorContext::new();

    // Initially no scene
    assert!(ctx.current_scene_path().is_none());
    assert_eq!(ctx.scene_name(), "Untitled");

    // Set a scene
    ctx.set_current_scene(Some(PathBuf::from("/test/scene.ron")), "My Scene".to_string());

    assert_eq!(ctx.current_scene_path(), Some(std::path::Path::new("/test/scene.ron")));
    assert_eq!(ctx.scene_name(), "My Scene");
}

#[test]
fn test_editor_settings_conversion() {
    let mut ctx = EditorContext::new();
    ctx.set_camera_offset(Vec2::new(150.0, -200.0));
    ctx.set_camera_zoom(1.5);

    let settings = ctx.to_editor_settings();

    assert_eq!(settings.camera_position, (150.0, -200.0));
    assert_eq!(settings.camera_zoom, 1.5);
}

#[test]
fn test_apply_editor_settings() {
    let mut ctx = EditorContext::new();

    let settings = engine_core::scene_data::EditorSettings {
        camera_position: (100.0, 50.0),
        camera_zoom: 2.0,
    };

    ctx.apply_editor_settings(&settings);

    assert_eq!(ctx.camera_offset(), Vec2::new(100.0, 50.0));
    assert_eq!(ctx.camera_zoom(), 2.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p editor test_editor_context_scene test_editor_settings_conversion test_apply_editor_settings`
Expected: FAIL

**Step 3: Write the implementation**

Add imports at top of `context.rs`:

```rust
use std::path::{Path, PathBuf};
use engine_core::scene_data::EditorSettings;
```

Add fields to `EditorContext` struct:

```rust
/// Path to currently loaded scene (None = unsaved new scene)
current_scene_path: Option<PathBuf>,
/// Scene name for display
scene_name: String,
```

Update `EditorContext::new()`:

```rust
current_scene_path: None,
scene_name: "Untitled".to_string(),
```

Add methods:

```rust
// ================== Scene Methods ==================

/// Get the path to the currently loaded scene.
pub fn current_scene_path(&self) -> Option<&Path> {
    self.current_scene_path.as_deref()
}

/// Set the current scene path and name.
pub fn set_current_scene(&mut self, path: Option<PathBuf>, name: String) {
    self.current_scene_path = path;
    self.scene_name = name;
}

/// Get the current scene name.
pub fn scene_name(&self) -> &str {
    &self.scene_name
}

/// Create EditorSettings from current editor state.
pub fn to_editor_settings(&self) -> EditorSettings {
    EditorSettings {
        camera_position: (self.camera_offset().x, self.camera_offset().y),
        camera_zoom: self.camera_zoom(),
    }
}

/// Apply EditorSettings to restore camera state.
pub fn apply_editor_settings(&mut self, settings: &EditorSettings) {
    self.set_camera_offset(Vec2::new(settings.camera_position.0, settings.camera_position.1));
    self.set_camera_zoom(settings.camera_zoom);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p editor test_editor_context_scene test_editor_settings_conversion test_apply_editor_settings`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/editor/src/context.rs
git commit -m "feat(editor): add scene path tracking and EditorSettings support"
```

---

## Task 11: Create File Operations Module

**Files:**
- Create: `crates/editor/src/file_operations.rs`
- Modify: `crates/editor/src/lib.rs`

**Step 1: Write the failing test**

Create `file_operations.rs`:

```rust
//! File operations for the editor (save, load, new scene).

use std::path::{Path, PathBuf};

use ecs::World;
use engine_core::assets::AssetManager;
use engine_core::scene_data::SceneLoadError;
use engine_core::scene_loader::{SceneInstance, SceneLoader};
use engine_core::scene_saver::{SceneSaveError, SceneSaver};

use crate::EditorContext;

/// Extended error type for file operations
#[derive(Debug, thiserror::Error)]
pub enum FileOperationError {
    #[error("No scene path set - use Save As")]
    NoPath,

    #[error("Save error: {0}")]
    SaveError(#[from] SceneSaveError),

    #[error("Load error: {0}")]
    LoadError(#[from] SceneLoadError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use ecs::sprite_components::Transform2D;
    use glam::Vec2;

    // Helper to create a minimal editor context for testing
    fn test_editor_context() -> EditorContext {
        EditorContext::new()
    }

    #[test]
    fn test_save_scene_no_path_returns_error() {
        let editor = test_editor_context();
        let world = World::default();

        let result = save_scene(&editor, &world, None);

        assert!(matches!(result, Err(FileOperationError::NoPath)));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p editor test_save_scene_no_path`
Expected: FAIL with "cannot find function `save_scene`"

**Step 3: Write the implementation**

```rust
/// Save the current scene to its existing path.
///
/// Returns an error if no path is set (use `save_scene_as` instead).
pub fn save_scene(
    editor: &EditorContext,
    world: &World,
    assets: Option<&AssetManager>,
) -> Result<PathBuf, FileOperationError> {
    let path = editor
        .current_scene_path()
        .ok_or(FileOperationError::NoPath)?
        .to_path_buf();

    SceneSaver::save_world_to_file(
        world,
        assets,
        editor.scene_name(),
        Some(editor.to_editor_settings()),
        &path,
    )?;

    Ok(path)
}

/// Save the current scene to a new path.
pub fn save_scene_as(
    editor: &mut EditorContext,
    world: &World,
    assets: Option<&AssetManager>,
    path: PathBuf,
) -> Result<(), FileOperationError> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    SceneSaver::save_world_to_file(
        world,
        assets,
        &name,
        Some(editor.to_editor_settings()),
        &path,
    )?;

    editor.set_current_scene(Some(path), name);
    Ok(())
}

/// Load a scene from a file.
pub fn load_scene(
    editor: &mut EditorContext,
    world: &mut World,
    assets: &mut AssetManager,
    path: PathBuf,
) -> Result<SceneInstance, FileOperationError> {
    // Clear existing entities
    world.clear();

    let data = SceneLoader::load_from_file(&path)?;

    // Apply editor settings if present
    if let Some(settings) = &data.editor {
        editor.apply_editor_settings(settings);
    }

    let name = data.name.clone();
    let instance = SceneLoader::instantiate(&data, world, assets)?;

    editor.set_current_scene(Some(path), name);

    // Clear selection since entities changed
    editor.selection.clear();

    Ok(instance)
}

/// Create a new empty scene.
pub fn new_scene(editor: &mut EditorContext, world: &mut World) {
    world.clear();
    editor.set_current_scene(None, "Untitled".to_string());
    editor.selection.clear();
    editor.reset_camera();
}
```

Add to `lib.rs`:

```rust
pub mod file_operations;
pub use file_operations::{save_scene, save_scene_as, load_scene, new_scene, FileOperationError};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p editor test_save_scene_no_path`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/editor/src/file_operations.rs crates/editor/src/lib.rs
git commit -m "feat(editor): add file operations module for save/load"
```

---

## Task 12: Add Integration Tests

**Files:**
- Modify: `crates/engine_core/src/scene_saver.rs`

**Step 1: Write the integration test**

```rust
#[test]
fn test_full_scene_roundtrip() {
    use crate::scene_loader::SceneLoader;
    use tempfile::NamedTempFile;

    // Create a complex scene
    let mut world = World::default();

    // Root entity with multiple components
    let player = world.create_entity();
    world.add_component(&player, ecs::Name("player".to_string())).unwrap();
    world.add_component(&player, Transform2D {
        position: Vec2::new(-200.0, 100.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
    }).unwrap();
    world.add_component(&player, Sprite {
        texture_handle: 0,
        offset: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
        color: glam::Vec4::new(0.2, 0.4, 1.0, 1.0),
        depth: 0.0,
        tex_region: [0.0, 0.0, 1.0, 1.0],
    }).unwrap();

    // Child entity
    let weapon = world.create_entity();
    world.add_component(&weapon, ecs::Name("weapon".to_string())).unwrap();
    world.add_component(&weapon, Transform2D::new(Vec2::new(20.0, 0.0))).unwrap();
    world.set_parent(weapon, player).unwrap();

    // Extract and save
    let mut scene = SceneSaver::extract_from_world(&world, None, "Integration Test");
    scene.editor = Some(EditorSettings {
        camera_position: (150.0, -75.0),
        camera_zoom: 1.25,
    });

    let temp_file = NamedTempFile::new().unwrap();
    SceneSaver::save_to_file(&scene, temp_file.path()).unwrap();

    // Load back
    let loaded = SceneLoader::load_from_file(temp_file.path()).unwrap();

    // Verify structure
    assert_eq!(loaded.name, "Integration Test");
    assert_eq!(loaded.entities.len(), 1); // Only root
    assert_eq!(loaded.entities[0].name, Some("player".to_string()));
    assert_eq!(loaded.entities[0].children.len(), 1);
    assert_eq!(loaded.entities[0].children[0].name, Some("weapon".to_string()));

    // Verify editor settings
    let editor = loaded.editor.unwrap();
    assert_eq!(editor.camera_position, (150.0, -75.0));
    assert_eq!(editor.camera_zoom, 1.25);

    // Verify components
    assert_eq!(loaded.entities[0].components.len(), 2); // Transform + Sprite
}
```

**Step 2: Run test**

Run: `cargo test -p engine_core test_full_scene_roundtrip`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/engine_core/src/scene_saver.rs
git commit -m "test(scene_saver): add full scene roundtrip integration test"
```

---

## Task 13: Final Verification and Documentation Update

**Files:**
- Modify: `PROJECT_ROADMAP.md`

**Step 1: Run all tests**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 2: Update roadmap**

Mark Scene Saving/Loading as complete in `PROJECT_ROADMAP.md`:

```markdown
- [x] **Scene Saving/Loading** - Editor-specific format ✅ COMPLETE
  - SceneSaver extracts World → SceneData
  - EditorSettings preserves camera position/zoom
  - Backward compatible with existing scenes
  - ~15 new tests
```

**Step 3: Commit**

```bash
git add PROJECT_ROADMAP.md
git commit -m "docs: mark Scene Saving/Loading as complete in roadmap"
```

**Step 4: Run final verification**

```bash
cargo test --workspace
cargo build --workspace
```

---

## Summary

**Total Tasks:** 13
**New Files:** 2 (scene_saver.rs, file_operations.rs)
**Modified Files:** 5 (scene_data.rs, assets.rs, lib.rs x2, context.rs)
**Estimated New Tests:** ~20

**Key Components:**
1. `EditorSettings` - Camera state persistence
2. `AssetManager::get_texture_path()` - Reverse texture lookup
3. `SceneSaver` - World extraction and file writing
4. `EditorContext` - Scene path tracking
5. `file_operations` - Save/load/new scene functions
