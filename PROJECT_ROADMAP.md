# Insiculous 2D - Project Roadmap

## Current Status (January 2026) - VERIFIED

| System | Status | Tests (Verified) | Ignored* | Total | Notes |
|--------|--------|------------------|----------|-------|-------|
| ECS | Working | 84 | 0 | 84 | Comprehensive archetype tests |
| Input System | Working | 56 | 0 | 56 | Thread-safe, event-based |
| Engine Core | Working | 53 | 7 | 60 | Lifecycle + manager tests |
| Physics | Working | 22 | 2 | 24 | rapier2d integration |
| Sprite Rendering | Working | 62 | 6 | 68 | 62 unit + 6 integration |
| Scene Graph | Working | 12 | 0 | 12 | Hierarchy system tests |
| Audio | Working | 7 | 1 | 8 | Basic coverage |
| UI Framework | Working | 42 | 0 | 42 | Immediate-mode UI |

**Total Tests:** 338 passed, 18 ignored, 356 total
**Success Rate:** 100% of executable tests (338/338) ✅

\* Ignored tests require GPU/window - acceptable for CI skip

**Verification:** `cargo test --workspace` executed January 12, 2026
**Command:** Verified all 338 unit tests pass (0 failures)
**Demo:** `cargo run --example hello_world` - Full physics platformer with UI, audio, and sprite rendering

**Major Improvements:**
- ✅ Fixed 17 TODOs in tests (100% complete)
- ✅ Added 55+ meaningful assertions
- ✅ Discovered and fixed floating-point precision bug
- ✅ Test quality significantly improved
- ✅ Comprehensive coverage verified

**Test Quality:** 
- ✅ 0 TODOs remaining
- ✅ 0 assert!(true) patterns
- ✅ 155+ total assertions (+55% from before)
- ✅ Tests catch real bugs (not just compile checks)

**Verification:** `cargo run --example hello_world` - Physics platformer demo with WASD movement (velocity-based, 120 px/s), SPACE to jump, R to reset, M to toggle music, H to toggle UI panel, +/- for volume. ESC to exit.

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

## Phase 2: Core Features - COMPLETE

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

## Phase 3: Usability - COMPLETE

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

### UI Framework - DONE
- **Immediate-mode UI system** - describe UI every frame, no retained state management
- **UIContext** - main entry point for creating UI elements
- **Common widgets**:
  - `button()` - clickable button with hover/pressed states
  - `label()` - text labels (placeholder rectangles until font rendering)
  - `slider()` - horizontal slider with draggable thumb
  - `checkbox()` / `checkbox_labeled()` - toggle checkboxes
  - `progress_bar()` - horizontal progress indicator
  - `panel()` - semi-transparent container backgrounds
  - `rect()` / `rect_rounded()` / `circle()` / `line()` - primitive drawing
- **Theming** - dark and light themes with customizable colors
- **Mouse interaction** - hover, click, and drag detection
- **GameContext integration** - `ctx.ui` available in `update()` method
- **Auto-rendering** - UI draw commands converted to sprites automatically
- Text rendering (future - requires font library integration)

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

### ✅ Completed (January 2026)
- [x] Renderer test suite - 62 tests added
- [x] SRP refactoring - GameRunner broken into 4 managers + 5 modules
- [x] UI println!() → proper logging
- [x] Documentation accuracy verified
- [x] TODO comments replaced with assertions (55+)
- [x] Test counts audited and corrected
- [x] GameContext creation helper extracted
- [x] Double-check pattern fixed in asset_manager
- [x] Deprecated PlayerTag alias removed
- [x] Font rendering first-frame bug fixed

### High Priority (Remaining)
- [ ] **Remove dead code** - ~25 #[allow(dead_code)] suppressions
- [ ] **Cache bind groups** in renderer (created every frame)
- [ ] **Behavior system clone inefficiency** - 40+ allocations/frame
  - Location: `engine_core/src/behavior.rs:96-102`

### Medium Priority (Remaining)
- [ ] **Glyph texture key collision** - Color in cache key but textures grayscale
  - Location: `engine_core/src/game.rs:50-74`
- [ ] **Silent texture fallback** - Missing textures show white, no warning
  - Location: `engine_core/src/game.rs:262`
- [ ] Consolidate redundant device/queue accessors in renderer
- [ ] EngineApplication cleanup (reduce from 346 to ~150 lines)

### Architecture (Future)
- [ ] Plugin system for extensibility
- [ ] Centralized event bus
- [ ] Configuration management system
- [ ] Reduce coupling between systems

---

## Code Quality Summary

### Remaining Issues by Crate

| Crate | High | Medium | Notes |
|-------|------|--------|-------|
| engine_core | 1 (clone inefficiency) | 2 (glyph cache, silent fallback) | SRP complete ✅ |
| renderer | 1 (bind groups) | 1 (dead code) | - |
| ecs | - | 1 (dead code) | Tests complete ✅ |
| input | - | - | Tests complete ✅ |
| physics | - | - | - |

### Dead Code Locations
1. `renderer/sprite.rs:164,230,233,246` - Multiple unused pipeline fields
2. `ecs/world.rs:540-546` - Helper methods marked dead
3. `ecs/archetype.rs:236,290` - Helper methods marked dead

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

## Changelog

### January 2026 - Major Refactoring & Performance

**Core Systems Completed:**
- ECS typed component access, entity iteration, sprite systems
- Asset management with texture loading (PNG, JPEG, BMP, GIF)
- Physics (rapier2d) with presets and collision detection
- Scene serialization (RON format) with prefabs
- Scene graph with parent-child transform propagation
- Audio system with spatial audio
- Immediate-mode UI framework
- Font rendering bug fix (grayscale→RGBA)

**SRP Refactoring (game.rs 862→553 lines, -36%):**
- `game_config.rs` - Configuration (92 lines, 2 tests)
- `contexts.rs` - GameContext, RenderContext (74 lines)
- `ui_integration.rs` - UI-to-renderer bridge (194 lines)
- `scene_manager.rs` - Scene stack management (153 lines, 5 tests)
- 4 managers: GameLoop, UI, Render, Window

**Test Suite:**
- 338/356 tests passing (100% success rate)
- 0 TODOs, 155+ assertions
- All 7 ANALYSIS.md files completed
