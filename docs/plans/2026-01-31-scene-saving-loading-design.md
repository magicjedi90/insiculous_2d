# Scene Saving/Loading Design

**Date:** 2026-01-31
**Status:** Approved
**Feature:** Phase 1 - Scene Editor Foundation (final item)

## Overview

Add the ability to save scenes from the editor, preserving both game entities and editor state (camera position/zoom). This completes Phase 1 of the editor roadmap.

## Requirements

1. **Save scenes** - Extract World entities → `SceneData` → write RON file
2. **Preserve editor state** - Camera position and zoom embedded in scene file
3. **Backward compatible** - Existing scenes load without changes
4. **File menu integration** - Save via File > Save / File > Save As (Ctrl+S)

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Save trigger | File menu only | Simple, matches editor maturity. Auto-save deferred. |
| Editor state scope | Camera position + zoom | Selection is transient, panel layout is global preference |
| State storage | Embedded `editor` field in SceneData | Single file, optional field ignored at runtime |
| Entity extraction | Component-by-component | Mirrors loader pattern, no ECS changes needed |
| Texture resolution | AssetManager reverse lookup | Keeps Sprite component lean, centralized mapping |
| Hierarchy format | Inline children | Visual hierarchy in file matches editor view |

## Architecture

### New Files

```
crates/engine_core/src/
├── scene_saver.rs      # NEW - World → SceneData extraction + file writing

crates/editor/src/
├── file_operations.rs  # NEW - Save/Load menu handlers
```

### Modified Files

```
crates/engine_core/src/
├── scene_data.rs       # Add EditorSettings struct
├── assets.rs           # Add get_texture_path() reverse lookup

crates/editor/src/
├── context.rs          # Add current_scene_path tracking
```

### Data Flow

**Save:**
```
User clicks File > Save
    ↓
EditorContext.current_scene_path (or prompt for path)
    ↓
SceneSaver::extract_from_world(world, assets) → SceneData
    ↓
SceneSaver::save_to_file(scene_data, path)
    ↓
RON file written
```

**Load (existing, with additions):**
```
User clicks File > Open
    ↓
File picker → path
    ↓
SceneLoader::load_from_file(path) → SceneData
    ↓
SceneLoader::instantiate(data, world, assets)
    ↓
EditorContext.apply_editor_settings(data.editor)  # NEW
    ↓
EditorContext.current_scene_path = Some(path)     # NEW
```

## Implementation Details

### 1. EditorSettings (scene_data.rs)

```rust
/// Editor-specific settings persisted with the scene
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorSettings {
    #[serde(default)]
    pub camera_position: (f32, f32),
    #[serde(default = "default_zoom")]
    pub camera_zoom: f32,
}

pub struct SceneData {
    pub name: String,
    #[serde(default)]
    pub physics: Option<PhysicsSettings>,
    #[serde(default)]
    pub editor: Option<EditorSettings>,  // NEW
    #[serde(default)]
    pub prefabs: HashMap<String, PrefabData>,
    #[serde(default)]
    pub entities: Vec<EntityData>,
}
```

### 2. AssetManager Texture Lookup (assets.rs)

```rust
pub struct AssetManager {
    textures: HashMap<String, TextureHandle>,
    texture_paths: HashMap<u32, String>,  // NEW - reverse mapping
}

impl AssetManager {
    pub fn get_texture_path(&self, handle: u32) -> Option<&str> {
        if handle == 0 {
            return Some("#white");
        }
        self.texture_paths.get(&handle).map(|s| s.as_str())
    }
}
```

Update `load_texture()` and `create_solid_color()` to populate `texture_paths`.

### 3. SceneSaver (scene_saver.rs)

```rust
pub struct SceneSaver;

impl SceneSaver {
    /// Extract all entities from world into SceneData
    pub fn extract_from_world(
        world: &World,
        assets: &AssetManager,
        scene_name: &str,
    ) -> SceneData {
        let mut entities = Vec::new();

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
        assets: &AssetManager,
        entity: EntityId,
    ) -> Option<EntityData> {
        let components = Self::extract_components(world, assets, entity);

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

    fn extract_components(world: &World, assets: &AssetManager, entity: EntityId) -> Vec<ComponentData> {
        let mut components = Vec::new();

        if let Some(t) = world.get::<Transform2D>(entity) {
            components.push(ComponentData::Transform2D {
                position: (t.position.x, t.position.y),
                rotation: t.rotation,
                scale: (t.scale.x, t.scale.y),
            });
        }

        if let Some(s) = world.get::<Sprite>(entity) {
            let texture = assets.get_texture_path(s.texture_handle)
                .unwrap_or("#white").to_string();
            components.push(ComponentData::Sprite {
                texture,
                offset: (s.offset.x, s.offset.y),
                rotation: s.rotation,
                scale: (s.scale.x, s.scale.y),
                color: (s.color.x, s.color.y, s.color.z, s.color.w),
                depth: s.depth,
            });
        }

        // Camera2D, SpriteAnimation, RigidBody, Collider, Behavior...
        // Same pattern for each component type

        components
    }

    fn extract_name(world: &World, entity: EntityId) -> Option<String> {
        world.get::<ecs::Name>(entity).map(|n| n.0.clone())
    }

    pub fn save_to_file(scene: &SceneData, path: impl AsRef<Path>) -> Result<(), SceneSaveError> {
        let config = ron::ser::PrettyConfig::default()
            .struct_names(true)
            .enumerate_arrays(false);

        let ron_string = ron::ser::to_string_pretty(scene, config)?;
        std::fs::write(path, ron_string)?;
        Ok(())
    }

    pub fn save_world_to_file(
        world: &World,
        assets: &AssetManager,
        scene_name: &str,
        editor_settings: Option<EditorSettings>,
        path: impl AsRef<Path>,
    ) -> Result<(), SceneSaveError> {
        let mut scene = Self::extract_from_world(world, assets, scene_name);
        scene.editor = editor_settings;
        Self::save_to_file(&scene, path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SceneSaveError {
    #[error("Failed to write scene file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to serialize scene: {0}")]
    SerializationError(#[from] ron::Error),
}
```

### 4. EditorContext Changes (context.rs)

```rust
pub struct EditorContext {
    // ... existing fields ...
    current_scene_path: Option<PathBuf>,
    scene_name: String,
}

impl EditorContext {
    pub fn current_scene_path(&self) -> Option<&Path> {
        self.current_scene_path.as_deref()
    }

    pub fn set_current_scene(&mut self, path: Option<PathBuf>, name: String) {
        self.current_scene_path = path;
        self.scene_name = name;
    }

    pub fn scene_name(&self) -> &str {
        &self.scene_name
    }

    pub fn to_editor_settings(&self) -> EditorSettings {
        EditorSettings {
            camera_position: (self.camera_offset().x, self.camera_offset().y),
            camera_zoom: self.camera_zoom(),
        }
    }

    pub fn apply_editor_settings(&mut self, settings: &EditorSettings) {
        self.set_camera_offset(Vec2::new(settings.camera_position.0, settings.camera_position.1));
        self.set_camera_zoom(settings.camera_zoom);
    }
}
```

### 5. File Operations (file_operations.rs)

```rust
pub fn save_scene(
    editor: &EditorContext,
    world: &World,
    assets: &AssetManager,
) -> Result<PathBuf, SceneSaveError> {
    let path = match editor.current_scene_path() {
        Some(p) => p.to_path_buf(),
        None => return Err(SceneSaveError::NoPath),
    };

    SceneSaver::save_world_to_file(
        world,
        assets,
        editor.scene_name(),
        Some(editor.to_editor_settings()),
        &path,
    )?;

    Ok(path)
}

pub fn save_scene_as(
    editor: &mut EditorContext,
    world: &World,
    assets: &AssetManager,
    path: PathBuf,
) -> Result<(), SceneSaveError> {
    let name = path.file_stem()
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

pub fn load_scene(
    editor: &mut EditorContext,
    world: &mut World,
    assets: &mut AssetManager,
    path: PathBuf,
) -> Result<SceneInstance, SceneLoadError> {
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

    Ok(instance)
}
```

## Testing Strategy

### Unit Tests

1. **Empty world** - Extract produces empty entities list
2. **Single entity** - Each component type extracts correctly
3. **Hierarchy** - Parent-child relationships become inline children
4. **Texture paths** - #white, #solid:RRGGBB, file paths all resolve
5. **Editor settings** - Camera position/zoom round-trip correctly
6. **Backward compat** - Load scene without `editor` field

### Integration Test

Load `hello_world.scene.ron` → save to temp file → reload → verify entity count and positions match.

### Test Count Estimate

- scene_saver.rs: ~10 tests
- assets.rs additions: ~3 tests
- context.rs additions: ~4 tests
- file_operations.rs: ~5 tests

**Total: ~22 new tests**

## Future Considerations (Not In Scope)

- **Auto-save with dirty tracking** - Track changes, prompt on close
- **Prefab preservation** - Currently flattened on save
- **Undo/redo integration** - Would need command pattern
- **Physics settings extraction** - Requires PhysicsSystem access

## Example Output

```ron
SceneData(
    name: "My Level",
    editor: Some(EditorSettings(
        camera_position: (150.0, -200.0),
        camera_zoom: 1.5,
    )),
    physics: Some(PhysicsSettings(
        gravity: (0.0, -980.0),
        pixels_per_meter: 100.0,
    )),
    entities: [
        EntityData(
            name: Some("player"),
            components: [
                Transform2D(
                    position: (100.0, 50.0),
                    rotation: 0.0,
                    scale: (1.0, 1.0),
                ),
                Sprite(
                    texture: "#white",
                    color: (0.2, 0.4, 1.0, 1.0),
                ),
                RigidBody(
                    body_type: Dynamic,
                    linear_damping: 5.0,
                ),
            ],
            children: [
                EntityData(
                    name: Some("weapon"),
                    components: [
                        Transform2D(position: (20.0, 0.0)),
                        Sprite(texture: "assets/sword.png"),
                    ],
                ),
            ],
        ),
    ],
)
```

## Implementation Order

1. Add `EditorSettings` to scene_data.rs
2. Add `texture_paths` reverse lookup to AssetManager
3. Create scene_saver.rs with extraction logic
4. Add scene tracking to EditorContext
5. Create file_operations.rs with save/load handlers
6. Wire up File menu actions
7. Add tests throughout
