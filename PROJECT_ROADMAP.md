# Insiculous 2D - Project Roadmap

## Current Status (January 2026)

| System | Status | Tests |
|--------|--------|-------|
| ECS | Working | 89 |
| Input System | Working | 60 |
| Engine Core | Working | 42 |
| Physics | Working | 22 |
| Sprite Rendering | Working | 62 |
| Scene Graph | Working | 12 |
| Audio | Working | 10 |

**Total Tests**: 280 (all passing)

**Verification:** `cargo run --example hello_world` - Physics platformer demo with WASD movement (velocity-based, 120 px/s), SPACE to jump, R to reset, M to toggle music, +/- for volume. ESC to exit.

### Simple Game API - NEW
The engine now provides a `Game` trait that handles all winit/window boilerplate internally:
- `run_game(game, config)` - One function to start your game
- `GameConfig` - Configure window title, size, clear color
- `GameContext` - Access input, ECS world, audio, assets, delta time in update()
- Default sprite rendering from ECS entities

### ECS Integration - FIXED

All major ECS API issues have been resolved:

| Issue | Status |
|-------|--------|
| Typed component access | `world.get::<T>()` and `world.get_mut::<T>()` |
| Component trait downcasting | `as_any()` and `as_any_mut()` methods |
| Entity iteration | `world.entities()` returns `Vec<EntityId>` |
| Sprite systems | `sprite_system.rs` compiled and exported |

**Note:** All ECS tests pass. Previous math bugs in transform/animation have been fixed.

---

## Phase 1: Stabilization - COMPLETE

**Goal:** Make the engine safe and functional.

| Item | Status |
|------|--------|
| Memory Safety & Lifetime Issues | Done |
| Input System Integration | Done |
| Core System Initialization | Done |

**Achievements:**
- Fixed `'static` lifetime requirements in renderer
- Implemented thread-safe input handling with event queue
- Added entity generation tracking for stale reference detection
- Proper lifecycle management with state transitions
- Panic-safe system registry with error recovery

---

## Phase 2: Core Features - IN PROGRESS

**Goal:** Make the engine usable for simple 2D games.

### Sprite Rendering System - DONE
- WGPU 28.0.0 instanced rendering
- Automatic sprite batching by texture
- Camera with orthographic projection
- Color tinting via white texture multiplication

### ECS Optimization - DONE
- Archetype-based component storage
- Dense arrays for cache locality
- Type-safe queries (Single, Pair, Triple)
- `World::new_optimized()` for archetype mode
- Typed component access via `get::<T>()` / `get_mut::<T>()`
- Entity iteration via `entities()`
- Sprite systems (SpriteAnimationSystem, SpriteRenderSystem)

### Resource Management - DONE
**Priority:** High

- Asset loading system (AssetManager in GameContext)
- Texture loading from files (PNG, JPEG, BMP, GIF)
- Programmatic texture creation (solid colors, checkerboard)
- Texture caching via TextureHandle system
- ECS texture handle integration (sprites use their assigned textures)
- **Scene Serialization (RON format)** - Unity/Godot-style scene files
  - `SceneData` - Root structure with physics settings, prefabs, and entities
  - `PrefabData` - Reusable entity templates with components
  - `EntityData` - Entity instances with prefab references and overrides
  - `SceneLoader::load_and_instantiate()` - Load scenes into ECS world
  - `Scene::load_from_file()` - Convenience method on Scene struct
  - Texture references: `#white`, `#solid:RRGGBB`, or file paths
  - Example scene file: `examples/assets/scenes/hello_world.scene.ron`
- Proper GPU resource cleanup (future)

### 2D Physics Integration - DONE
**Priority:** Medium

- Integrated rapier2d 0.23 for 2D physics simulation
- Physics components for ECS (RigidBody, Collider)
- Collision detection and response with proper collider sizing
- Fixed timestep physics simulation (1/60s default)
- Continuous Collision Detection (CCD) for fast-moving objects
- Raycasting support
- Velocity-based movement for precise platformer controls
- Force/impulse-based movement for physics-driven gameplay
- **Physics presets** for common game types:
  - `PhysicsConfig::platformer()` / `top_down()` / `low_gravity()` / `space()`
  - `RigidBody::player_platformer()` / `player_top_down()` / `pushable()`
  - `Collider::player_box()` / `platform()` / `pushable_box()` / `bouncy()` / `slippery()`
  - `MovementConfig::platformer()` / `top_down()` / `floaty()`
- Debug visualization (future)

---

## Phase 3: Usability - IN PROGRESS

**Goal:** Make the engine productive for developers.

### Scene Graph System - DONE
- Parent-child entity relationships (`Parent`, `Children` components)
- Transform hierarchy propagation (`TransformHierarchySystem`, `GlobalTransform2D`)
- Hierarchy queries (`get_children()`, `get_parent()`, `get_root_entities()`, `get_descendants()`, `get_ancestors()`)
- Scene serialization with hierarchy (`parent` field and inline `children` in RON)
- Spatial queries and frustum culling (future)

### Audio System - DONE
- Sound effect playback via `AudioManager`
- Background music with pause/resume
- Volume control (master, SFX, music)
- ECS components (`AudioSource`, `AudioListener`, `PlaySoundEffect`)
- Spatial audio attenuation (distance-based volume falloff)
- Format support: WAV, MP3, OGG, FLAC, AAC

### UI Framework
- Immediate mode UI rendering
- Common widgets (button, text, slider)
- Layout system
- Input integration

---

## Phase 4: Polish - PLANNED

**Goal:** Make the engine competitive.

### Advanced Rendering
- 2D lighting system
- Post-processing pipeline
- Particle system
- Normal mapping support

### Editor Tools
- Scene editor
- Property inspector
- Asset browser
- Hot reloading

### Platform Support
- Mobile (iOS/Android)
- Web (WASM)
- Consoles
- Platform-specific optimizations

---

## Technical Debt

### Critical (Must Fix)
- [x] ~~**Add renderer test suite**~~ - 62 tests added (COMPLETED Jan 2026)
- [x] ~~**Fix SRP violations** in GameRunner and EngineApplication~~ - FIXED (Jan 2026)
  - Created `RenderManager` - encapsulates renderer, sprite pipeline, camera
  - Created `WindowManager` - encapsulates window creation and size tracking
  - Refactored both `GameRunner` and `EngineApplication` to use managers
- [x] ~~**Replace println!()** with log::info!() in behavior.rs:243~~ - FIXED (Jan 2026)

### High Priority
- [ ] **Remove dead code** - ~25 #[allow(dead_code)] suppressions remaining
- [ ] **Reduce clone() calls** in behavior update loop (40+ per frame)
- [ ] **Cache bind groups** in renderer (currently created every frame)
- [ ] **Replace TODO comments** in tests with actual assertions

### Medium Priority
- [ ] Consolidate redundant device/queue accessors in renderer
- [x] ~~Extract GameContext creation helper (duplicated 3 times)~~ - FIXED (Jan 2026)
- [x] ~~Fix double-check pattern in asset_manager access~~ - FIXED (Jan 2026)
- [x] ~~Remove deprecated PlayerTag alias from ECS~~ - FIXED (Jan 2026)

### Architecture
- [ ] Plugin system for extensibility
- [ ] Centralized event bus
- [ ] Configuration management system
- [ ] Reduce coupling between systems

---

## Code Quality Summary

### Issues by Crate

| Crate | Critical | High | Medium | Low |
|-------|----------|------|--------|-----|
| engine_core | 2 (SRP) | 2 (clone, println) | 2 (redundant code) | - |
| renderer | - | 2 (bind groups, dead code) | 1 (accessors) | - |
| ecs | - | - | 3 (deprecated, visibility, bloat) | - |
| input | - | - | 2 (TODO tests, dead zones) | - |
| physics | - | - | 2 (dead code, test gaps) | - |

### Dead Code Locations
1. ~~`engine_core/application.rs:253` - `extract_sprite_data()` returns empty data~~ - REMOVED
2. `renderer/sprite.rs:164,230,233,246` - Multiple unused pipeline fields
3. `ecs/world.rs:540-546` - Helper methods marked dead
4. `ecs/archetype.rs:236,290` - Helper methods marked dead
5. ~~`physics/world.rs:166,172,178` - Three consecutive dead methods~~ - NOW PUBLIC API

---

## Risk Mitigation

1. **Incremental Refactoring** - Fix issues without large rewrites
2. **Comprehensive Testing** - Tests for every fix (especially renderer)
3. **Code Reviews** - Thorough review for all changes
4. **Documentation** - Document architectural decisions
5. **Quality Gates** - No new dead code, no new SRP violations

---

## Success Metrics

| Phase | Criteria |
|-------|----------|
| Phase 2 | 1000+ sprites at 60 FPS, 10,000+ entities in ECS |
| Phase 3 | Complex scene hierarchies, spatial audio, functional UI |
| Phase 4 | Visual effects, productive editor, cross-platform builds |

---

## Next Steps

1. ~~**Fix ECS Component API**~~ - DONE - Returns `&T` via `get::<T>()`
2. ~~**Add entity iteration**~~ - DONE - `entities()` method on World
3. ~~**Include sprite_system.rs**~~ - DONE - System trait fixed
4. ~~**Implement resource management**~~ - DONE - AssetManager with texture loading
5. ~~**Add physics integration with rapier2d**~~ - DONE - Full 2D physics with collisions
6. ~~**Add scene serialization**~~ - DONE - RON-based scene files with prefabs
7. ~~**Scene graph system**~~ - DONE - Parent-child relationships with transform propagation
8. ~~**Add renderer test suite**~~ - DONE - 62 tests added
9. ~~**Fix SRP violations**~~ - DONE - Created RenderManager and WindowManager, refactored GameRunner and EngineApplication
10. ~~**Audio system**~~ - DONE - Sound effects, background music, volume control, ECS integration
11. **UI Framework** - Immediate mode UI rendering with common widgets
