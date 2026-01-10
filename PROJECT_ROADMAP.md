# Insiculous 2D - Project Roadmap

## Current Status (January 2026)

| System | Status | Tests |
|--------|--------|-------|
| Input System | Working | 56 |
| Engine Core | Working | 29 |
| ECS | Working | 60 |
| Sprite Rendering | Working | 0 (visual) |

**Verification:** `cargo run --example hello_world` - Uses simplified Game API, ECS for game state, WASD to move player, wood texture on platform.

### Simple Game API - NEW
The engine now provides a `Game` trait that handles all winit/window boilerplate internally:
- `run_game(game, config)` - One function to start your game
- `GameConfig` - Configure window title, size, clear color
- `GameContext` - Access input, ECS world, delta time in update()
- Default sprite rendering from ECS entities

### ECS Integration - FIXED

All major ECS API issues have been resolved:

| Issue | Status |
|-------|--------|
| Typed component access | ✅ `world.get::<T>()` and `world.get_mut::<T>()` |
| Component trait downcasting | ✅ `as_any()` and `as_any_mut()` methods |
| Entity iteration | ✅ `world.entities()` returns `Vec<EntityId>` |
| Sprite systems | ✅ `sprite_system.rs` compiled and exported |

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

- ✅ Asset loading system (AssetManager in GameContext)
- ✅ Texture loading from files (PNG, JPEG, BMP, GIF)
- ✅ Programmatic texture creation (solid colors, checkerboard)
- ✅ Texture caching via TextureHandle system
- ✅ ECS texture handle integration (sprites use their assigned textures)
- Configuration file parsing (future)
- Proper GPU resource cleanup (future)

### 2D Physics Integration - TODO
**Priority:** Medium

- Integrate rapier2d or similar
- Physics components for ECS
- Collision detection and response
- Debug visualization

---

## Phase 3: Usability - PLANNED

**Goal:** Make the engine productive for developers.

### Scene Graph System
- Parent-child entity relationships
- Transform hierarchy propagation
- Spatial queries and frustum culling
- Scene serialization/deserialization

### Audio System
- Sound effect playback
- Background music streaming
- 2D positional audio
- Audio resource management

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

### Immediate
- [ ] Add structured logging (replace basic `log::` calls)
- [ ] Increase test coverage for renderer
- [ ] Profile and optimize hot paths

### Architecture
- [ ] Plugin system for extensibility
- [ ] Centralized event bus
- [ ] Configuration management system
- [ ] Reduce coupling between systems

---

## Success Metrics

| Phase | Criteria |
|-------|----------|
| Phase 2 | 1000+ sprites at 60 FPS, 10,000+ entities in ECS |
| Phase 3 | Complex scene hierarchies, spatial audio, functional UI |
| Phase 4 | Visual effects, productive editor, cross-platform builds |

---

## Risk Mitigation

1. **Incremental Refactoring** - Fix issues without large rewrites
2. **Comprehensive Testing** - Tests for every fix
3. **Code Reviews** - Thorough review for all changes
4. **Documentation** - Document architectural decisions

---

## Next Steps

1. ~~**Fix ECS Component API**~~ - ✅ DONE - Returns `&T` via `get::<T>()`
2. ~~**Add entity iteration**~~ - ✅ DONE - `entities()` method on World
3. ~~**Include sprite_system.rs**~~ - ✅ DONE - System trait fixed
4. ~~**Implement resource management**~~ - ✅ DONE - AssetManager with texture loading
5. **Add physics integration with rapier2d** - Next priority
6. **Add configuration file parsing** - TOML/JSON config support
7. **Scene graph system** - Parent-child entity relationships
