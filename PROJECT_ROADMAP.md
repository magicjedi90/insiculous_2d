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

**Full Reports:** Each crate has a detailed `TECH_DEBT.md` with line numbers and suggested fixes.

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
- [x] **Font rendering visibility fix** - UI depth clipping (1000→900) + batch sorting
- [x] **Dead code suppressions documented** - Added explanatory comments to all remaining #[allow(dead_code)]
- [x] **Bind group caching** - Camera bind group created once, texture bind groups cached per handle
- [x] **Behavior system clone inefficiency** - Behaviors now accessed by reference, only small BehaviorState cloned
- [x] **renderer/KISS-002: Unsafe transmute removed** - Surface now uses proper `'static` lifetime from `Arc<Window>`
- [x] **physics/KISS-001: Collision start/stop detection** - Proper tracking with `CollisionPair` and frame-to-frame comparison
- [x] **renderer/ARCH-002: Time struct consolidated** - Re-exported from common crate
- [x] **renderer/ARCH-001: device/queue accessors documented** - Both versions kept with clear documentation
- [x] **renderer/DRY-001: Surface error handling extracted** - `acquire_frame()` helper method
- [x] **ecs/ARCH-002: Hierarchy cycle detection** - `set_parent()` now detects and rejects cycles

### High Priority (Safety/Functional Issues)
None - all high priority issues resolved! ✅

### Medium Priority

**Cross-Crate Issues:**
- [x] **common/ARCH-001: CameraUniform duplicated** - ✅ Already re-exported from common via renderer
- [ ] **ui/ARCH-001 + engine_core/KISS-001: Dual glyph caching** - Caches exist in both ui and engine_core
  - Location: `ui/src/font.rs`, `engine_core/src/contexts.rs`
  - Fix: Consolidate caching strategy (ui=CPU bitmaps, engine_core=GPU textures)
- [x] **renderer/ARCH-002: Time struct misplaced** - ✅ Time now defined in common, re-exported from renderer

**engine_core:**
- [ ] **Glyph texture key collision** - Color in cache key but textures grayscale
  - Location: `engine_core/src/contexts.rs:50-74`
- [ ] **Silent texture fallback** - Missing textures show white, no warning
  - Location: `engine_core/src/game.rs:262`
- [ ] **EngineApplication cleanup** - Reduce from 346 to ~150 lines
  - Location: `engine_core/src/application.rs`

**renderer:**
- [x] **ARCH-001: Redundant device/queue accessors** - ✅ Both versions kept with clear documentation for different use cases
- [x] **DRY-001: Duplicate surface error handling** - ✅ Extracted to `acquire_frame()` helper method
- [ ] **DRY-002: Duplicate sampler creation** - Nearly identical in 4 locations
  - Location: `sprite.rs:312-321`, `sprite.rs:647-656`, `sprite_data.rs:163-172`, `texture.rs:375-389`
- [ ] **SRP-001: SpritePipeline too large** - Manages 13 GPU resources
  - Location: `renderer/src/sprite.rs:225-254`
  - Fix: Split into PipelineResources, BufferManager, CameraManager, TextureBindGroupManager

**ecs:**
- [x] **ARCH-002: No cycle detection in hierarchy** - ✅ Cycle detection added in `set_parent()`, 2 tests
- [ ] **KISS-002: ComponentColumn uses unsafe** - Raw pointer dereferencing
  - Location: `ecs/src/component.rs:112-145`
  - Fix: Consider safe alternatives or add safety documentation

**input:**
- [ ] **SRP-001: Dual update methods confusion** - `update()` vs `end_frame()` unclear
  - Location: `input/src/input_handler.rs:238-258`
  - Fix: Rename to `process_and_end_frame()` vs `end_frame()` or document clearly
- [ ] **KISS-001: Multi-action binding asymmetry** - `get_action()` returns only first action
  - Location: `input/src/input_mapping.rs:122-139`
  - Fix: Document limitation or change to `HashMap<InputSource, Vec<GameAction>>`

**ui:**
- [ ] **DRY-001: Duplicate glyph-to-draw-data conversion** - Same pattern in two methods
  - Location: `ui/src/context.rs:215-235, 255-279`
- [ ] **SRP-001: FontManager too many responsibilities** - Loading, storage, rasterization, caching, layout
  - Location: `ui/src/font.rs:100-315`

**physics:**
- [ ] **DRY-001: Repeated pixel-to-meter conversion** - Pattern repeated dozens of times
  - Location: `physics/src/physics_world.rs` (throughout)
  - Fix: Consider newtype wrappers `Pixels(f32)` and `Meters(f32)`

### Low Priority (Code Organization)

**DRY Violations (Working but verbose):**
- [ ] renderer/DRY-003: Duplicate render pass descriptor creation
- [ ] renderer/DRY-004: Duplicate texture descriptor creation
- [ ] ecs/DRY-001: Repeated component storage operations
- [ ] ecs/DRY-002: Duplicate error variants
- [ ] input/DRY-001: Repeated input state tracking pattern across device types
- [ ] input/DRY-002: Repeated action checking pattern
- [ ] ui/DRY-002: Duplicate checkbox drawing logic
- [ ] ui/DRY-003: Duplicate UIContext constructor logic
- [ ] physics/DRY-002: Repeated body builder pattern

**Architecture (Nice to Have):**
- [ ] ecs/SRP-001: World struct has many responsibilities
- [ ] ecs/ARCH-001: Entity ID format inconsistency (u32 vs usize)
- [ ] ecs/ARCH-004: Mixed visibility patterns
- [ ] renderer/SRP-002: Renderer handles init AND rendering
- [ ] renderer/ARCH-004: Inconsistent error types (RendererError vs TextureError)
- [ ] input/ARCH-001: Dual error types (InputError vs InputThreadError)
- [ ] input/ARCH-002: InputEvent uses winit types directly (acceptable coupling)
- [ ] ui/ARCH-002: rect.rs is essentially a re-export
- [ ] ui/ARCH-003: TextDrawData duplicates GlyphDrawData info
- [ ] common/KISS-001: Unused `with_prefixed_fields!` macro
- [ ] physics/ARCH-001: PhysicsSystem has pass-through methods
- [ ] physics/ARCH-002: Single collision callback limitation

### Architecture (Future Features)
- [ ] Plugin system for extensibility
- [ ] Centralized event bus
- [ ] Configuration management system
- [ ] Reduce coupling between systems

---

## Code Quality Summary

### Issues by Crate (from TECH_DEBT.md reports)

| Crate | High | Medium | Low | Total | Overall Assessment |
|-------|------|--------|-----|-------|-------------------|
| engine_core | 0 | 3 | 10 | 13 | SRP complete ✅, behavior optimized ✅ |
| renderer | 0 | 5 | 7 | 12 | Bind groups cached ✅, unsafe fixed ✅ |
| ecs | 0 | 1 | 11 | 12 | Tests complete ✅, cycle detection ✅ |
| ui | 0 | 2 | 8 | 10 | Well-structured immediate-mode UI |
| input | 0 | 2 | 5 | 7 | Production-ready, minor API confusion |
| physics | 0 | 0 | 5 | 5 | Clean rapier2d integration, collision detection ✅ |
| common | 0 | 1 | 3 | 4 | Minimal debt, well-designed foundation |
| **Total** | **0** | **10** | **49** | **59** | |

### High Priority Summary
All high priority issues resolved! ✅

### Cross-Crate Dependencies to Address
1. ~~CameraUniform: common ↔ renderer (duplication)~~ ✅ Already properly shared
2. Glyph caching: ui ↔ engine_core (dual caches)
3. ~~Time struct: renderer → common (misplaced)~~ ✅ Consolidated in common

### Documented Scaffolding (Intentional Dead Code)
All `#[allow(dead_code)]` suppressions are now documented with explanatory comments:
1. `renderer/sprite.rs` - Reserved fields for batch splitting, pipeline recreation, sampler fallback
2. `renderer/sprite_data.rs` - Buffer usage stored for potential recreation
3. `renderer/texture.rs` - Builder pattern field stored for future use
4. `ecs/world.rs` - Scaffolding for future full query implementation
5. `ecs/archetype.rs` - Scaffolding for query types
6. `audio/manager.rs` - Reserved for future "stop by handle" API

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

**Technical Debt Audit (January 2026):**
- Created TECH_DEBT.md reports for all 7 crates
- Identified 66 total issues (2 high, 15 medium, 49 low priority)
- Categorized by DRY (23), SRP (13), KISS (10), Architecture (20)
- High priority: unsafe transmute in renderer, incomplete collision events in physics
- Cross-crate issues documented: CameraUniform duplication, dual glyph caching
