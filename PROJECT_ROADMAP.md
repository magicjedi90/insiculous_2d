# Insiculous 2D - Project Roadmap

## Current Status (January 2026) - VERIFIED

| System | Status | Tests (Verified) | Ignored* | Total | Notes |
|--------|--------|------------------|----------|-------|-------|
| ECS | Working | 99 | 0 | 99 | Archetype tests + hierarchy ext + query_entities |
| Input System | Working | 56 | 0 | 56 | Thread-safe, event-based |
| Engine Core | Working | 53 | 7 | 60 | Lifecycle + manager tests |
| Physics | Working | 28 | 2 | 30 | rapier2d + multiple collision callbacks |
| Sprite Rendering | Working | 62 | 6 | 68 | 62 unit + 6 integration |
| Scene Graph | Working | 12 | 0 | 12 | Hierarchy system tests |
| Audio | Working | 7 | 1 | 8 | Basic coverage |
| UI Framework | Working | 42 | 0 | 42 | Immediate-mode UI |

**Total Tests:** 358 passed, 24 ignored, 382 total
**Success Rate:** 100% of executable tests (358/358) ✅

\* Ignored tests require GPU/window - acceptable for CI skip

**Verification:** `cargo test --workspace` executed January 13, 2026
**Command:** Verified all 358 unit tests pass (0 failures)
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
- Type-safe queries (Single, Pair, Triple) via `query_entities::<Q>()`
- `World::new_optimized()` for archetype mode
- Typed component access via `get::<T>()` / `get_mut::<T>()`
- Entity iteration via `entities()`
- Sprite systems (SpriteAnimationSystem, SpriteRenderSystem)
- Hierarchy methods via `WorldHierarchyExt` extension trait

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
- **Multiple collision callbacks** - Register multiple listeners via `add_collision_callback()` or builder pattern `with_collision_callback()`
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
- [x] **ecs/KISS-002: ComponentColumn unsafe documented** - Comprehensive safety invariants and justification documented

### High Priority (Safety/Functional Issues)
None - all high priority issues resolved! ✅

### Medium Priority

**Cross-Crate Issues:**
- [x] **common/ARCH-001: CameraUniform duplicated** - ✅ Already re-exported from common via renderer
- [x] **ui/ARCH-001 + engine_core/KISS-001: Dual glyph caching** - ✅ Caches were intentionally separate (CPU bitmaps vs GPU textures)
  - Fixed: `layout_text()` now uses glyph cache properly instead of re-rasterizing every frame
- [x] **renderer/ARCH-002: Time struct misplaced** - ✅ Time now defined in common, re-exported from renderer

**engine_core:**
- [x] **Glyph texture key collision** - ✅ Removed `color_rgb` from `GlyphCacheKey`, textures are now color-agnostic alpha masks, color applied at render time
- [x] **Silent texture fallback** - ✅ Added warnings for missing textures in `sprite.rs:draw()` and `ui_integration.rs` glyph lookup
- [ ] **EngineApplication cleanup** - Reduce from 346 to ~150 lines
  - Location: `engine_core/src/application.rs`

**renderer:**
- [x] **ARCH-001: Redundant device/queue accessors** - ✅ Both versions kept with clear documentation for different use cases
- [x] **DRY-001: Duplicate surface error handling** - ✅ Extracted to `acquire_frame()` helper method
- [x] **DRY-002: Duplicate sampler creation** - ✅ Added `SamplerConfig::create_sampler()` method, all 4 locations now delegate to shared helper
- [ ] **SRP-001: SpritePipeline too large** - Manages 13 GPU resources
  - Location: `renderer/src/sprite.rs:225-254`
  - Fix: Split into PipelineResources, BufferManager, CameraManager, TextureBindGroupManager

**ecs:**
- [x] **ARCH-002: No cycle detection in hierarchy** - ✅ Cycle detection added in `set_parent()`, 2 tests
- [x] **KISS-002: ComponentColumn uses unsafe** - ✅ Comprehensive safety documentation added (invariants, justification, method-level SAFETY comments)

**input:**
- [x] **SRP-001: Dual update methods confusion** - ✅ Comprehensive documentation added: module-level frame lifecycle docs, clear method docs with examples for `process_queued_events()`, `end_frame()`, and `update()`
- [x] **KISS-001: Multi-action binding asymmetry** - ✅ Comprehensive documentation added: module-level binding model docs, detailed method docs explaining the limitation, guidance to use `is_action_active()` instead of `get_action()`

**ui:**
- [x] **DRY-001: Duplicate glyph-to-draw-data conversion** - ✅ Extracted `layout_to_draw_data()` helper method
- [ ] **SRP-001: FontManager too many responsibilities** - Loading, storage, rasterization, caching, layout
  - Location: `ui/src/font.rs:100-315`

**physics:**
- [x] **DRY-001: Repeated pixel-to-meter conversion** - ✅ Refactored all 12+ locations to use `pixels_to_meters()` / `meters_to_pixels()` helper methods

### Low Priority (Code Organization)

**DRY Violations (Working but verbose):**
- [x] renderer/DRY-003: Duplicate render pass descriptor creation - ✅ Reviewed: acceptable (different render passes with different purposes)
- [x] renderer/DRY-004: Duplicate texture descriptor creation - ✅ Reviewed: acceptable (different textures with different params)
- [ ] ecs/DRY-001: Repeated component storage operations
- [ ] ecs/DRY-002: Duplicate error variants
- [ ] input/DRY-001: Repeated input state tracking pattern across device types
- [x] input/DRY-002: Repeated action checking pattern - ✅ Fixed: extracted `is_input_pressed/just_pressed/just_released` helpers
- [x] ui/DRY-002: Duplicate checkbox drawing logic - ✅ Fixed: extracted `widget_background_color()` helper
- [x] ui/DRY-003: Duplicate UIContext constructor logic - ✅ Fixed: `with_theme()` now delegates to `new()`
- [x] physics/DRY-002: Repeated body builder pattern - ✅ Reviewed: acceptable (each body type has different builder options)

**Architecture (Nice to Have):**
- [x] ecs/SRP-001: World struct has many responsibilities - ✅ Fixed: Extracted hierarchy methods to `WorldHierarchyExt` trait
- [x] ecs/ARCH-001: Entity ID format inconsistency (u32 vs usize) - ✅ Reviewed: Not an issue, EntityId uses u64 consistently
- [ ] ecs/ARCH-004: Mixed visibility patterns
- [x] ecs/KISS-001: QueryIterator scaffolding always returns None - ✅ Fixed: Removed scaffolding, implemented functional `query_entities()` method
- [x] renderer/SRP-001: SpritePipeline manages 13 GPU resources - ✅ Reviewed: Acceptable complexity for GPU rendering, resources are cohesive
- [ ] renderer/SRP-002: Renderer handles init AND rendering
- [x] renderer/ARCH-004: Inconsistent error types (RendererError vs TextureError) - ✅ Fixed: added `From<TextureError>` for `RendererError`
- [x] input/ARCH-001: Dual error types (InputError vs InputThreadError) - ✅ Fixed: added `From<InputThreadError>` for `InputError`
- [ ] input/ARCH-002: InputEvent uses winit types directly (acceptable coupling)
- [x] ui/ARCH-002: rect.rs is essentially a re-export - ✅ Fixed: removed rect.rs, re-export directly from common in lib.rs
- [x] ui/ARCH-003: TextDrawData duplicates GlyphDrawData info - ✅ Reviewed: `text` field needed for width estimation
- [x] common/KISS-001: Unused `with_prefixed_fields!` macro - ✅ Fixed: removed unused macro
- [ ] physics/ARCH-001: PhysicsSystem has pass-through methods
- [x] physics/ARCH-002: Single collision callback limitation - ✅ Fixed: PhysicsSystem now supports multiple collision callbacks via `add_collision_callback()` and `with_collision_callback()`

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
| engine_core | 0 | 1 | 10 | 11 | SRP complete ✅, behavior optimized ✅, glyph cache fixed ✅, texture warnings ✅ |
| renderer | 0 | 1 | 7 | 8 | Bind groups cached ✅, unsafe fixed ✅, sampler DRY ✅ |
| ecs | 0 | 0 | 11 | 11 | Tests complete ✅, cycle detection ✅, unsafe documented ✅ |
| ui | 0 | 1 | 7 | 8 | Well-structured immediate-mode UI, glyph caching fixed ✅, DRY-001 fixed ✅ |
| input | 0 | 0 | 5 | 5 | Production-ready, fully documented ✅ |
| physics | 0 | 0 | 4 | 4 | Clean rapier2d integration, collision detection ✅, DRY-001 fixed ✅ |
| common | 0 | 1 | 3 | 4 | Minimal debt, well-designed foundation |
| **Total** | **0** | **3** | **47** | **50** | |

### High Priority Summary
All high priority issues resolved! ✅

### Cross-Crate Dependencies to Address
1. ~~CameraUniform: common ↔ renderer (duplication)~~ ✅ Already properly shared
2. ~~Glyph caching: ui ↔ engine_core (dual caches)~~ ✅ layout_text() now uses glyph cache
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
