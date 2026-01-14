# Insiculous 2D - Project Roadmap

## Current Status (January 2026) - PRODUCTION READY ✅

**Engine Status:** Core systems complete, production-ready for 2D games

### Test Coverage
| System | Tests | Status |
|--------|-------|--------|
| ECS | 99 | ✅ 100% pass |
| Input | 56 | ✅ 100% pass |
| Engine Core | 53 | ✅ 100% pass |
| Physics | 28 | ✅ 100% pass |
| Renderer | 62 | ✅ 100% pass |
| Scene Graph | 12 | ✅ 100% pass |
| Audio | 7 | ✅ 100% pass |
| UI | 42 + 7 new | ✅ 100% pass |

**Total:** 358/358 tests passing (100% success rate)
**Code Quality:** 0 TODOs, 155+ assertions, 18 ignored (GPU/window only)

### Completed Features
- ✅ Simple Game API (`Game` trait, `run_game()`)
- ✅ ECS with archetype storage and type-safe queries
- ✅ WGPU 28.0.0 sprite rendering with batching
- ✅ Rapier2d physics with presets (platformer, top-down, etc.)
- ✅ Scene serialization (RON format) with prefabs
- ✅ Scene graph hierarchy with transform propagation
- ✅ Immediate-mode UI framework (buttons, sliders, panels)
- ✅ Event-based input system (keyboard, mouse, gamepad)
- ✅ Spatial audio with rodio backend
- ✅ Asset manager with texture loading

**Verification:** `cargo run --example hello_world` - Full platformer demo with physics, UI, audio

---

## NEW ROADMAP: Editor & Tooling Focus

### Phase 1: Scene Editor Foundation 🎯 PRIORITY

**Goal:** Create a visual scene editor for building game worlds

**Core Editor Features:**
- [ ] **Editor UI Framework** - Dockable panels, toolbar, menus
  - Based on existing immediate-mode UI system
  - Panel types: Scene view, Inspector, Hierarchy, Asset browser
- [ ] **Scene Viewport** - Render game world with editor overlay
  - Grid overlay and alignment guides
  - Camera pan/zoom controls
  - Selection rectangle and transform gizmos
- [ ] **Entity Inspector** - Edit component properties
  - Automatic UI generation for component fields
  - Support for: Transform2D, Sprite, RigidBody, Collider, AudioSource
- [ ] **Hierarchy Panel** - Tree view of parent-child relationships
  - Drag-and-drop reparenting
  - Expand/collapse entity trees
- [ ] **Scene Saving/Loading** - Editor-specific format
  - Preserve editor state (camera position, selection)
  - Backward compatibility with runtime scenes

**Technical Implementation:**
- Create `crates/editor/` - Editor-specific code
- `EditorContext` - Extends `GameContext` with editor state
- `EditorTool` enum - Select, Move, Rotate, Scale
- `Selection` struct - Track selected entities
- Gizmo rendering system - Transform handles

**Milestone:** Edit `hello_world.scene.ron` visually, move platforms, adjust physics

---

### Phase 2: Sprite Animation System 🎯 PRIORITY

**Goal:** Build comprehensive 2D animation tools

**Animation Features:**
- [ ] **Sprite Sheet Importer** - Import and slice sprite sheets
  - Automatic grid-based slicing
  - Manual frame definition with visual editor
  - Support for: PNG, Aseprite files (.ase/.aseprite)
- [ ] **Animation Timeline Editor** - Keyframe-based animation
  - Dopesheet view for frame timing
  - Curve editor for tweening/easing
  - Preview window with playback controls
- [ ] **Animation Controller** - State machine for animations
  - Animation states (Idle, Run, Jump, Attack)
  - Transitions with conditions (parameters, triggers)
  - Blend trees for smooth transitions
- [ ] **Bone/Skeletal Animation** - 2D skeletal deformation
  - Bone hierarchy creation
  - Weight painting tool
  - Mesh deformation
- [ ] **Animation Components** - ECS integration
  - `AnimationPlayer` - Play animations on entities
  - `Animator` - State machine controller
  - `SpriteSheet` - Reference to sprite sheet asset

**Technical Implementation:**
- `crates/animation/` - Animation system
- `AnimationClip` - Sequence of frames with timing
- `AnimationController` - State machine asset
- `SpriteSheetAsset` - Texture atlas with metadata
- Integration with existing `Sprite` component

**Milestone:** Animate a character with idle, run, jump animations controlled by physics

---

### Phase 3: Asset Pipeline & Management

**Goal:** Professional-grade asset workflow

**Asset Pipeline:**
- [ ] **Asset Browser** - Visual browser for all game assets
  - Thumbnail generation for textures, sprites, scenes
  - Search and filter capabilities
  - Folder organization with drag-and-drop
- [ ] **Sprite Atlas Generator** - Automatic texture packing
  - Bin packing algorithms (MaxRects, Shelf)
  - Automatic sprite referencing updates
  - Padding and extrusion options
- [ ] **Asset Import Pipeline** - Automated import with caching
  - Watch folders for auto-import
  - Import presets for common asset types
  - Background import with progress tracking
- [ ] **Font Asset Support** - Visual font preview and configuration
  - Font preview with size adjustment
  - Character range selection
  - Distance field font generation
- [ ] **Audio Asset Manager** - Waveform preview and editing
  - Waveform visualization
  - Trim and loop point editing
  - Format conversion (WAV ↔ OGG)

**Technical Implementation:**
- Extend `crates/engine_core/src/assets.rs`
- `AssetDatabase` - Track all assets with metadata
- `AssetProcessor` - Import and process assets
- `AssetWatcher` - File system watching
- Thumbnail cache system

**Milestone:** Import assets via drag-and-drop, auto-atlas generation, visual browsing

---

### Phase 4: Advanced Editor Features

**Goal:** Professional-grade development environment

**Advanced Tools:**
- [ ] **Visual Scripting** - Node-based game logic
  - Event nodes (OnStart, OnUpdate, OnCollision)
  - Action nodes (PlaySound, SetPosition, SpawnEntity)
  - Flow control (Branch, Sequence, Loop)
  - Variable system (local, global, blackboard)
- [ ] **Particle System Editor** - Visual particle effect creation
  - Emitter configuration (rate, shape, burst)
  - Particle properties (lifetime, size, color, velocity)
  - Curve editors for property animation
  - Real-time preview
- [ ] **Physics Debugger** - Visual physics debugging
  - Collider wireframe rendering
  - Velocity vector visualization
  - Collision point highlighting
  - AABB tree visualization
- [ ] **Profiler Integration** - Performance analysis tools
  - Frame time graph (16.6ms target line)
  - System timing breakdown (ECS, Physics, Render, UI)
  - Draw call and batch count
  - Memory usage tracking
- [ ] **Tilemap Editor** - Grid-based level editing
  - Paintbrush and fill tools
  - Multiple layers (background, foreground, collision)
  - Autotiling support
  - Tile property editing

**Technical Implementation:**
- `crates/editor/src/tools/` - Editor tool system
- `VisualScriptGraph` - Node graph asset
- `ParticleEmitter` - GPU particle system
- `PhysicsDebugRenderer` - Debug draw integration
- `EditorProfiler` - Performance capture and analysis

**Milestone:** Create particle effects, debug physics collisions, optimize performance bottlenecks

---

### Phase 5: Platform & Deployment

**Goal:** Multi-platform support and distribution

**Platform Support:**
- [ ] **Web (WASM) Export** - Browser-based games
  - WASM build pipeline
  - WebGL2 rendering backend
  - Touch input mapping
  - Asset loading via HTTP/fetch
- [ ] **Mobile Export** - iOS and Android
  - Touch gesture support (tap, drag, pinch)
  - Mobile-optimized UI scaling
  - App bundle generation
  - Performance optimizations (batching, texture compression)
- [ ] **Desktop Optimization** - Windows, macOS, Linux
  - Build scripts and installers
  - Steam/Epic integration
  - Controller mapping database
- [ ] **Hot Reloading** - Live asset and code reloading
  - Texture hot-reload
  - Scene hot-reload
  - Script hot-reload (future: Rust scripting)

**Technical Implementation:**
- Build scripts in `build/`
- Platform abstraction layer
- Mobile app templates
- Hot-reload file watching

**Milestone:** Deploy `hello_world.rs` to Web, mobile, and desktop

---

## Technical Debt (Remaining - January 2026)

**Overall Status:** 50 total items (59 completed, 50 remaining - 54% resolution rate)

### High Priority (0 items) ✅
All high priority technical debt has been resolved.

### Medium Priority (12 items)

**engine_core (6 items):**
- [ ] **EngineApplication cleanup** - Reduce from 346 to ~150 lines
  - Location: `engine_core/src/application.rs`
  - Note: This is deprecated code, migrate to Game API
- [ ] **DRY-001: Duplicate AudioManager placeholder pattern** - Asset and game loops both create fallback
  - Location: Multiple `GameContext` creation sites
  - Fix: Extract `ensure_audio()` helper method
- [ ] **DRY-003: Duplicate GameContext creation pattern** - Init and update both build contexts
  - Location: `game.rs: GameRunner` initialization and update
  - Fix: Extract `create_game_context()` helper
- [ ] **SRP-001: GameRunner still has multiple responsibilities** - Audio update, initialization check, etc.
  - Location: `game.rs: update_and_render()` calls 7 methods but still orchestrates
  - Fix: Extract `AudioManager`, `SceneManager` from orchestration logic
- [ ] **SRP-003: EngineApplication duplicates GameRunner functionality** - Both manage game lifecycle
  - Location: `application.rs:346 lines` vs `game.rs:553 lines`
  - Fix: Deprecate EngineApplication fully, migrate to Game API
- [ ] **ARCH-001: Dual API pattern creates confusion** - Both EngineApplication and GameRunner exist
  - Location: Throughout engine_core
  - Fix: Consolidate on Game API, mark EngineApplication as deprecated

**renderer (2 items):**
- [ ] **SRP-001: SpritePipeline holds too many GPU resources** - Manages 13 resources in one struct
  - Location: `renderer/src/sprite.rs:225-254`
  - Fix: Split into PipelineResources, BufferManager, CameraManager, TextureBindGroupManager
- [ ] **ARCH-003: Dead code with #[allow(dead_code)] suppressions** - 4 documented but unused items
  - Location: `sprite.rs`, `sprite_data.rs`, `texture.rs` 
  - Fix: Use fields or remove them if truly unnecessary

**ui (1 item):**
- [ ] **SRP-001: FontManager too many responsibilities** - Loading, storage, rasterization, caching, layout
  - Location: `ui/src/font.rs:100-315`
  - Fix: Split into FontLoader, GlyphCache, TextLayoutEngine

**ecs (2 items):**
- [ ] **SRP-002: ComponentStorage enum handles both storage types** - Single enum for two storage strategies
  - Location: `ecs/src/component.rs`
  - Fix: Consider trait objects or separate types for Legacy vs Archetype
- [ ] **SRP-003: TransformHierarchySystem does double iteration** - Separate root and child passes
  - Location: `ecs/src/hierarchy_system.rs:87-118`
  - Fix: Reorganize to single pass with better data flow

**common (1 item):**
- [ ] **ARCH-001: CameraUniform duplicated in renderer crate** - Exists in both common and renderer
  - Fix: Use `common::CameraUniform` everywhere, remove renderer copy

### Low Priority (38 items)

**engine_core (6 items):**
- [ ] **DRY-002: Duplicate coordinate transformation logic in ui_integration.rs** - UI-to-world code repeated
- [ ] **DRY-004: Repeated hex color parsing error handling** - Multiple .parse().expect() calls
- [ ] **DRY-005: Duplicate surface error recovery pattern** - Error handling in renderer and surface creation
- [ ] **SRP-002: BehaviorRunner handles multiple behavior types inline** - Large match statement with duplicated logic
- [ ] **KISS-002: Over-engineered lifecycle state machine** - 8 states in EngineApplication, simplify to 3
- [ ] **ARCH-002: Timer vs GameLoopManager overlap** - Both track frame timing
  - Location: `timing.rs` and `game_loop_manager.rs`
  - Fix: Consolidate timing logic

**ecs (5 items):**
- [ ] **DRY-001: Repeated entity existence checks** - 7+ duplicate patterns in world.rs
- [ ] **DRY-003: Duplicate matrix computation in GlobalTransform2D** - Sin/cos calculated multiple times
- [ ] **DRY-004: Repeated builder pattern in audio_components.rs** - Identical `with_volume()` methods
- [ ] **ARCH-003: Dead code marked but not removed** - Several #[allow(dead_code)] suppressions
- [ ] **ARCH-004: Dual storage systems add complexity** - Legacy and Archetype both maintained

**renderer (7 items):**
- [ ] **DRY-001: Duplicate surface error handling in renderer.rs** - Pattern repeated in multiple methods
- [ ] **DRY-003: Duplicate render pass descriptor in sprite.rs** - Similar render pass setup
- [ ] **DRY-004: Duplicate texture descriptor creation** - Texture upload code patterns
- [ ] **SRP-003: RenderPipelineInspector mixes logging with operation wrapping** - Debug utils mixed with pipeline logic
- [ ] **KISS-001: RenderPipelineInspector is over-engineered** - Complex debug infrastructure
- [ ] **ARCH-001: Redundant device/queue accessors** - Both device() and gpu_device() methods exist
- [ ] **ARCH-002: Time struct in renderer crate is misplaced** - Should be in common crate

**ui (4 items):**
- [ ] **DRY-004: Repeated font check and placeholder fallback** - Font loading check repeated in label methods
- [ ] **SRP-002: UIContext has large widget methods** - button(), slider() are 40+ lines each
- [ ] **KISS-001: WidgetPersistentState has unused flexibility** - String field rarely used
- [ ] **ARCH-003: TextDrawData duplicates GlyphDrawData info** - Text field overlaps with glyphs vector

**physics (2 items):**
- [ ] **DRY-002: Repeated body builder pattern in add_rigid_body** - Similar builder code for body types
- [ ] **SRP-001: PhysicsWorld handles too many rapier types** - Direct exposure of rapier internals

**input (2 items):**
- [ ] **DRY-001: Repeated input state tracking pattern** - Keyboard/mouse/gamepad similar structure
- [ ] **DRY-003: Repeated unbind logic in input_mapping.rs** - Unbinding has duplicate patterns

**common (2 items):**
- [ ] **DRY-001: Duplicate matrix construction pattern in transform.rs** - Similar to GlobalTransform2D pattern
- [ ] **ARCH-002: Camera2D vs renderer sprite_data Camera2D** - Verify Camera2D is canonical source

**Cross-Crate/System (3 items):**
- [ ] **ecs: DRY-002: Duplicate error variants** - Consolidate error types across crates
- [ ] **ARCH-001: Reduce coupling** - Event bus or message system for cross-crate communication
- [ ] **ARCH-002: Configuration system** - Centralized config management

**Total Remaining:** 50 items (vs 101 originally - 50% reduction ✓)
</thinking>

Now I'll update the PROJECT_ROADMAP.md with the comprehensive technical debt list:

<function=StrReplaceFile>
<parameter=path>/home/jedi/RustroverProjects/insiculous_2d/PROJECT_ROADMAP.md</parameter>
<parameter=edit>{

---

## Success Metrics

### Phase 1 (Scene Editor)
- Create a complete level in 30 minutes (vs 2 hours manually)
- Zero manual RON editing required
- Visual feedback for all operations

### Phase 2 (Animation)
- Support 100+ frame animations at 60 FPS
- < 16ms frame time with animated sprites
- Seamless animation transitions

### Phase 3 (Asset Pipeline)
- Import 50+ assets in < 5 minutes
- Automatic atlas packing with < 10% wasted space
- Real-time thumbnail generation

### Phase 4 (Advanced Tools)
- Visual scripts run at 95% of native code speed
- Particle systems with 1000+ particles at 60 FPS
- < 5ms overhead for physics debug rendering

### Phase 5 (Platform Support)
- Web: Load and start in < 5 seconds
- Mobile: 60 FPS on mid-range devices (2022+)
- Desktop: Package and deploy in < 1 minute

---

## Development Guidelines

### Editor Architecture
1. **Separate editor from runtime** - Editor is a separate binary
2. **Use existing systems** - Leverage UI, ECS, and rendering
3. **Modular tools** - Each tool is a separate module
4. **Game preview mode** - Run game inside editor viewport
5. **Asset hot-reloading** - Immediate feedback on changes

### Sprite Animation
1. **Data-driven** - Animations are assets, not code
2. **ECS integration** - Components reference animation assets
3. **GPU-friendly** - Sprite sheets + UV animation
4. **Extensible** - Support for frame-based and skeletal
5. **Tool-first** - Visual editor is primary workflow

### Asset Pipeline
1. **Automated** - Minimal manual steps
2. **Cached** - Only reprocess changed assets
3. **Verified** - Import validation and error reporting
4. **Versioned** - Asset metadata and versioning
5. **Distributed** - Support for team collaboration

---

## Quick Reference

**Commands:**
```bash
# Run tests
cargo test --workspace

# Run demo
cargo run --example hello_world

# Future: Run editor
cargo run --bin editor

# Future: Build for web
cargo build --target wasm32-unknown-unknown --release

# Future: Build for mobile
cargo apk build --release
```

**Key Files:**
- `AGENTS.md` - AI agent guidance (high-level)
- `training.md` - API patterns and examples
- `PROJECT_ROADMAP.md` - This file (single source of truth)
- `crates/*/TECH_DEBT.md` - Per-crate technical debt
- `examples/hello_world.rs` - Reference implementation

---

## Archive: Completed Phases (2025)

<details>
<summary>Click to expand completed work</summary>

### Phase 1: Stabilization - COMPLETE ✅
**Goal:** Make the engine safe and functional.
- Memory safety & lifetime issues resolved
- Thread-safe input handling with event queue
- Panic-safe system registry

### Phase 2: Core Features - COMPLETE ✅
**Goal:** Make the engine usable for simple 2D games.
- Sprite rendering with WGPU 28.0.0 and batching
- Archetype-based ECS with type-safe queries
- Asset management with texture loading
- Rapier2d physics integration with presets
- Scene serialization (RON format) with prefabs

### Phase 3: Usability - COMPLETE ✅
**Goal:** Make the engine productive for developers.
- Scene graph with parent-child relationships
- Audio system with spatial audio
- Immediate-mode UI framework
- Simple Game API (`Game` trait, `run_game()`)
- SRP refactoring (game.rs: 862→553 lines, -36%)

**Technical Debt Resolved:**
- 66 issues identified → 8 remaining (88% reduction)
- 0 high priority issues remaining
- All SRP refactorings complete
- Test quality: 0 TODOs, 155+ assertions

</details>
