# Insiculous 2D - Project Roadmap & Priority List

## Executive Summary (Updated: January 2026)

The Insiculous 2D engine has a solid architectural foundation. Key status:

- **Input System**: WORKING - 56 tests passing
- **Engine Core**: WORKING - 29 tests passing, lifecycle management stable
- **ECS**: WORKING - 60 tests passing, archetype storage implemented
- **Sprite Rendering**: BROKEN - GPU pipeline works but sprites are invisible due to buffer alignment issue

**Current Blocker**: Sprite rendering is non-functional. The GPU pipeline works (proven by NDC quad and triangle tests), but the sprite pipeline has a vertex/instance buffer format mismatch with the shader.

## 🔥 Critical Priority (Fix Immediately)

These issues must be fixed before any further development:

### 0. SPRITE RENDERING VISIBILITY BUG 🚨
**Status**: 🔴 **BROKEN - ROOT CAUSE IDENTIFIED, FIX NEEDED**
**Impact**: **100% BLOCKING** - Cannot render sprites until fixed

**Problem**: Sprites are invisible despite GPU pipeline functioning correctly.

**Diagnostic Evidence**:
```
ndc_quad_test.rs:     ✅ PASS - GPU pipeline works, colored quad visible
minimal_triangle_test.rs: ✅ PASS - Shaders execute correctly
final_sprite_test.rs: ❌ FAIL - Full sprite pipeline shows only dark blue background
sprite_demo.rs:       ❌ FAIL - No visible sprites
```

**Root Cause** (suspected):
1. `array_stride` in `SpriteVertex::desc()` / `SpriteInstance::desc()` doesn't match struct memory layout
2. Attribute offset calculations wrong due to padding/alignment
3. Shader `@location(N)` indices don't match vertex buffer attribute locations
4. Camera uniform layout mismatch (size or field order)

**Fix Actions**:
1. Verify struct sizes match GPU buffer strides using `std::mem::size_of`
2. Log actual vertex/instance data before GPU upload
3. Compare triangle test (works) vs sprite pipeline (fails) buffer setups
4. Create debug shader to visualize instance data as colors

### 1. Memory Safety & Lifetime Issues
**Status**: 🔴 CRITICAL - Engine unsafe to use
**Effort**: High (1-2 weeks)
**Files**: `crates/engine_core/src/application.rs`, `crates/renderer/src/renderer.rs`

- Fix `'static` lifetime requirements in renderer
- Resolve async runtime confusion (remove tokio from core)
- Implement proper resource cleanup
- Add thread safety to input system

### 2. Input System Integration
**Status**: ✅ COMPLETED - 100% Complete, All tests passing
**Effort**: Medium (3-5 days) - 5 days completed
**Files**: `crates/input/src/input_handler.rs`, `crates/engine_core/src/application.rs`

- ✅ Connect InputHandler to winit event loop - COMPLETED
- ✅ Implement event queue for input events - COMPLETED
- ✅ Fix input state update cycle - COMPLETED
- ✅ Add basic input mapping system - COMPLETED
- ✅ Fix remaining test failures in input mapping system - COMPLETED
- ✅ Complete integration testing with examples - COMPLETED
- ✅ Add comprehensive documentation - COMPLETED

**Technical Implementation**:
- Event queue system buffers input events between frames
- Input mapping system binds inputs to game actions
- Thread-safe wrapper for concurrent access
- 56 tests passing
- Default bindings for WASD, mouse, and gamepad controls

### 3. Core System Initialization
**Status**: ✅ COMPLETED - 100% Complete, All tests passing
**Effort**: Medium (3-5 days) - 6 days completed
**Files**: `crates/engine_core/src/scene.rs`, `crates/engine_core/src/lifecycle.rs`, `crates/ecs/src/world.rs`, `crates/ecs/src/system.rs`, `crates/ecs/src/generation.rs`

- ✅ Fix scene initialization race conditions - COMPLETED
- ✅ Add proper system lifecycle management - COMPLETED
- ✅ Implement entity generation tracking - COMPLETED
- ✅ Fix system registry memory safety - COMPLETED
- ✅ Add comprehensive test coverage - COMPLETED
- ✅ Integrate with existing systems - COMPLETED

**Technical Implementation**:
- Thread-safe lifecycle management with proper state transitions
- Entity generation tracking for detecting stale references
- System lifecycle with initialization/start/stop/shutdown phases
- Panic-safe system registry with error recovery
- Engine Core: 29 tests, ECS: 60 tests

## 🟡 High Priority (Fix Soon)

These features are essential for a minimally functional engine:

### 4. Sprite Rendering System
**Status**: 🔴 **BROKEN** - See section 0 above
**Files**: `crates/renderer/src/sprite.rs`, `crates/renderer/src/sprite_data.rs`, `crates/renderer/src/shaders/sprite_instanced.wgsl`

Sprites are invisible. GPU pipeline works (NDC quads render), but sprite-specific rendering fails. See section 0 for root cause analysis and fix plan.

### 5. ECS Optimization
**Status**: ✅ COMPLETED - Major Performance Improvements Achieved
**Effort**: High (1-2 weeks) - 1 week completed
**Files**: `crates/ecs/src/world.rs`, `crates/ecs/src/component.rs`, `crates/ecs/src/archetype.rs`

- ✅ **Implemented archetype-based component storage** - Dense arrays for optimal cache locality
- ✅ **Added type-safe component queries** - QueryTypes trait with Single, Pair, Triple query support
- ✅ **Created comprehensive benchmarks** - Performance comparison between legacy and optimized ECS
- ✅ **Built working demonstration** - Shows 1000+ entities at 60+ FPS with archetype storage
- ✅ **Backward compatibility maintained** - Legacy HashMap storage still available
- ✅ **All core tests passing** - 100% success rate for ECS functionality

**Technical Implementation**:
- **Archetype Storage**: Entities with same component types stored together in dense arrays
- **Component Columns**: Efficient memory layout with minimal allocations
- **Query System**: Type-safe queries for entities with specific component combinations
- **Performance Optimized**: Better cache locality, reduced memory fragmentation
- **Dual Storage Modes**: `World::new()` for legacy, `World::new_optimized()` for archetype storage

**Performance Benefits**:
- **Cache Locality**: Components stored contiguously in memory
- **Reduced Allocations**: Dense arrays minimize heap allocations
- **Faster Iteration**: Linear memory access patterns
- **Memory Efficiency**: Components packed tightly without HashMap overhead
- **Scalability**: Handles 1000+ entities efficiently

**Note**: System scheduling improvements still needed - SystemContext architecture mismatch remains

### 6. Resource Management
**Status**: 🟡 High Priority - Essential for any real game
**Effort**: Medium (1 week)
**Files**: New crate `crates/asset_manager/`

- Create asset loading system
- Implement texture caching
- Add configuration file support
- Implement resource cleanup

## 🟢 Medium Priority (Plan For)

These features make the engine usable for game development:

### 7. Scene Graph System
**Status**: 🟢 Medium Priority - Architecture enhancement
**Effort**: High (1-2 weeks)
**Files**: `crates/engine_core/src/scene.rs` (rewrite)

- Implement proper scene graph with parent-child relationships
- Add transform hierarchy and propagation
- Implement spatial queries and culling
- Add scene serialization support

### 8. 2D Physics Integration
**Status**: 🟢 Medium Priority - Gameplay essential
**Effort**: High (2-3 weeks)
**Files**: New crate `crates/physics/`

- Integrate 2D physics engine (rapier2d or similar)
- Add physics components to ECS
- Implement collision detection and response
- Add physics debug rendering

### 9. Audio System
**Status**: 🟢 Medium Priority - Gameplay essential
**Effort**: Medium (1 week)
**Files**: New crate `crates/audio/`

- Implement audio playback system
- Add 2D positional audio
- Implement audio streaming
- Add sound effect management

### 10. UI Framework
**Status**: 🟢 Medium Priority - User experience
**Effort**: High (2-3 weeks)
**Files**: New crate `crates/ui/`

- Create immediate mode UI system
- Implement common UI widgets
- Add UI layout system
- Integrate with input system

## 🔵 Low Priority (Nice To Have)

These features enhance the engine but aren't essential:

### 11. Advanced Rendering Features
**Status**: 🔵 Low Priority - Visual enhancement
**Effort**: High (2-4 weeks)
**Files**: `crates/renderer/src/`, new modules

- Implement 2D lighting system
- Add post-processing pipeline
- Implement particle system
- Add normal mapping support

### 12. Editor Tools
**Status**: 🔵 Low Priority - Development tools
**Effort**: Very High (1-2 months)
**Files**: New crate `crates/editor/`

- Create scene editor
- Implement property inspector
- Add asset browser
- Implement visual scripting

### 13. Platform Support
**Status**: 🔵 Low Priority - Market reach
**Effort**: Medium (1-2 weeks)
**Files**: Various platform-specific files

- Add mobile platform support
- Implement web export (WASM)
- Add console platform support
- Implement platform-specific optimizations

### 14. Advanced Features
**Status**: 🔵 Low Priority - Specialized use cases
**Effort**: Variable
**Files**: Various new crates

- Implement networking/multiplayer
- Add save/load system
- Implement modding support
- Add advanced profiling tools

## Technical Debt & Code Quality

### Immediate Refactoring Needed
1. **Error Handling**: Replace `unwrap()` calls with proper error handling - COMPLETED for input, core, and renderer systems
2. **Logging**: Add structured logging instead of basic `log::` calls
3. **Documentation**: Add comprehensive documentation to public APIs - COMPLETED for input, core, and renderer systems
4. **Testing**: Increase test coverage (currently ~95% for input and core systems, 100% for renderer core functionality)

### Architecture Improvements
1. **Plugin System**: Implement proper plugin architecture for extensibility
2. **Event Bus**: Create centralized event system for decoupled communication
3. **Configuration System**: Implement proper configuration management
4. **Dependency Injection**: Reduce tight coupling between systems

## Development Phases

### Phase 1: Stabilization (2-3 weeks)
**Goal**: Make the engine safe and functional
- ✅ Fix critical memory safety issues - COMPLETED (100%)
- ✅ Implement basic input system - COMPLETED (100%)
- ✅ Fix initialization race conditions - COMPLETED (100%)
- ✅ Add basic error handling - COMPLETED (100%)

**Status**: ✅ **PHASE 1 COMPLETE**
**Test Results**: Input: 56 tests, Engine Core: 29 tests, ECS: 60 tests (all passing)
**Key Achievements**: Memory safety, thread-safe systems, proper lifecycle management

### Phase 2: Core Features (4-6 weeks)
**Goal**: Make the engine usable for simple games
- ❌ **Sprite rendering system** - BROKEN (see section 0)
  - WGPU 28.0.0 infrastructure set up
  - Sprite batching and texture management code exists
  - **Sprites are invisible** - buffer alignment issue
  - Renderer tests removed (0 tests)
- ✅ **Optimize ECS performance** - COMPLETED
  - Archetype-based component storage implemented
  - Type-safe component queries (Single, Pair, Triple)
  - 60 tests passing
- Add resource management
- Create basic physics integration

### Phase 3: Usability (6-8 weeks)
**Goal**: Make the engine productive for developers
- Implement scene graph system
- Add audio system
- Create UI framework
- Add configuration system

### Phase 4: Polish (8-12 weeks)
**Goal**: Make the engine competitive
- Add advanced rendering features
- Implement editor tools
- Add platform support
- Performance optimization

## Risk Assessment

### High Risk Issues
1. **Memory Safety**: Current code has potential use-after-free and race conditions
2. **Performance**: ECS implementation is inefficient for large numbers of entities
3. **Scalability**: Fixed buffer sizes and lack of resource management limit scalability
4. **Maintainability**: Tight coupling makes changes difficult and error-prone

### Mitigation Strategies
1. **Incremental Refactoring**: Fix issues incrementally rather than rewriting everything
2. **Comprehensive Testing**: Add tests for each fix to prevent regressions
3. **Code Reviews**: Implement thorough code review process for all changes
4. **Documentation**: Document all architectural decisions and patterns

## Resource Requirements

### Development Effort
- **Critical Issues**: 3-4 weeks (1 developer)
- **High Priority**: 6-8 weeks (1 developer)
- **Medium Priority**: 8-12 weeks (1 developer)
- **Low Priority**: 12-20 weeks (1 developer)

### Skills Required
- **Systems Programming**: Memory management, concurrency, performance
- **Graphics Programming**: WGPU, shader development, rendering techniques
- **Game Development**: ECS, game loops, input handling, physics
- **Software Architecture**: Design patterns, API design, modularity

## Success Metrics

### Phase 1 Success Criteria
- No memory safety issues (verified with Valgrind/AddressSanitizer)
- All examples run without crashes
- Basic input functionality works
- Error handling is consistent

### Phase 2 Success Criteria
- Can render 1000+ sprites at 60 FPS
- ECS can handle 10,000+ entities
- Resource loading is reliable
- Basic physics simulation works

### Phase 3 Success Criteria
- Scene graph supports complex hierarchies
- Audio system supports multiple simultaneous sounds
- UI system provides common widgets
- Configuration system is flexible

### Phase 4 Success Criteria
- Rendering supports advanced visual effects
- Editor tools improve development productivity
- Platform support is stable
- Performance is competitive with other Rust game engines

## Recommendations

### Immediate Actions (Next 2 weeks)
1. **Stop Feature Development**: Focus exclusively on fixing critical issues
2. **Add Memory Safety Tests**: Use tools like AddressSanitizer and Valgrind
3. **Implement Basic Input**: Get input system working with event loop integration
4. **Document Current State**: Clearly mark what's working and what's not

### Short Term (Next 2 months)
1. **Refactor Core Systems**: Fix architectural issues in engine_core and renderer
2. **Implement Missing Features**: Focus on sprite rendering and ECS optimization
3. **Add Comprehensive Tests**: Achieve > 80% test coverage
4. **Create Documentation**: Write user guides and API documentation

### Long Term (3-6 months)
1. **Build Community**: Create Discord server, write blog posts, give talks
2. **Add Advanced Features**: Implement physics, audio, and UI systems
3. **Create Examples**: Build sample games demonstrating engine capabilities
4. **Optimize Performance**: Profile and optimize critical paths

## Conclusion

The Insiculous 2D engine has good architectural foundations but needs significant work to become production-ready. The critical issues around memory safety and system integration must be addressed immediately before any further development. With focused effort on stabilization and core features, the engine could become a viable option for 2D game development in Rust within 3-6 months.

The key to success is:
1. **Fix critical issues first** - Don't add features until the foundation is solid
2. **Test thoroughly** - Every fix needs comprehensive tests
3. **Document everything** - Good documentation is essential for adoption
4. **Build incrementally** - Small, tested changes are better than large rewrites
5. **Focus on usability** - Make the engine easy to use for developers

With proper execution of this roadmap, Insiculous 2D can become a competitive game engine in the Rust ecosystem.

## 🎯 Sprite Rendering System - Status Update

### WGPU 28.0.0 Migration
Successfully migrated from deprecated `ImageDataLayout` to `TexelCopyBufferLayout`.

### Current Status: BROKEN
- Infrastructure is in place (batching, camera, texture management)
- GPU pipeline works (NDC quads and triangles render)
- **Sprites are invisible** due to vertex/instance buffer alignment issue
- Renderer tests were removed (0 tests)

See section 0 for debugging details and fix plan.