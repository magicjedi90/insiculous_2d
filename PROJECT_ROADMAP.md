# Insiculous 2D - Project Roadmap

## Current Status (January 2026) - PRODUCTION READY ✅

**Engine Status:** Core systems complete, production-ready for 2D games

### Test Coverage
| System | Tests | Status |
|--------|-------|--------|
| ECS | 113 | ✅ 100% pass |
| Input | 56 | ✅ 100% pass |
| Engine Core | 68 | ✅ 100% pass |
| Physics | 28 | ✅ 100% pass |
| Renderer | 62 | ✅ 100% pass |
| Audio | 3 | ✅ 100% pass |
| UI | 53 | ✅ 100% pass |
| Editor | 136 | ✅ 100% pass |
| ECS Macros | 3 | ✅ 100% pass |
| Common | 26 | ✅ 100% pass |

**Total:** 549/549 tests passing (100% success rate)
**Code Quality:** 0 TODOs, 155+ assertions, 30 ignored (GPU/window only)

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
- [x] **Editor UI Framework** - Dockable panels, toolbar, menus ✅ COMPLETE
  - Based on existing immediate-mode UI system
  - Panel types: Scene view, Inspector, Hierarchy, Asset browser
  - 64 tests, DockArea/DockPanel, MenuBar, Toolbar components
- [ ] **Scene Viewport** - Render game world with editor overlay
  - Grid overlay and alignment guides
  - Camera pan/zoom controls
  - Selection rectangle and transform gizmos
- [x] **Entity Inspector** - Edit component properties ✅ COMPLETE
  - Automatic UI generation for component fields
  - Support for: Transform2D, Sprite, RigidBody, Collider, AudioSource
  - Editable field types: f32 slider, Vec2 input, bool checkbox, Vec4 color picker
  - 15 new tests for editable inspector and component editors
- [ ] **Hierarchy Panel** - Tree view of parent-child relationships
  - Drag-and-drop reparenting
  - Expand/collapse entity trees
- [ ] **Scene Saving/Loading** - Editor-specific format
  - Preserve editor state (camera position, selection)
  - Backward compatibility with runtime scenes

**Technical Implementation:** ✅ COMPLETE
- Created `crates/editor/` - Editor-specific code
- `EditorContext` - Extends `GameContext` with editor state
- `EditorTool` enum - Select, Move, Rotate, Scale
- `Selection` struct - Track selected entities
- `Gizmo` struct - Transform handles (translate, rotate, scale)

**Milestone:** Edit `hello_world.scene.ron` visually, move platforms, adjust physics

**Status Update (January 27, 2026):**
- Scene Viewport: ✅ Complete (45 tests - camera pan/zoom, grid overlay, coordinate conversion)
- Entity Inspector: ✅ Complete (15 new tests - editable fields for all component types)
- Next: Hierarchy Panel, Scene Saving/Loading

---

### Phase 2: Scripted Behaviors (Script Components) 🎯 PRIORITY

**Goal:** Unity/Godot-style script components - attach behavior scripts to entities with hot-reload support

**Philosophy:** Engine transparency - developers write Rust code that feels as natural as Unity C# or GDScript, with immediate iteration via hot-reload

**Core Features:**
- [ ] **Script Trait System** - Rust-native script components
  - `Script` trait with lifecycle hooks (`on_start`, `on_update`, `on_physics`, `on_destroy`)
  - Clear separation: `on_update` for visual/frame logic, `on_physics` for physics/movement (avoids Unity's confusing update/fixed_update overlap)
  - Automatic component registration with ECS
  - Access to `ScriptContext` (entity, world queries, delta time, input)
- [ ] **Hot-Reload Support** - Iterate without recompiling the game
  - Scripts compiled as dynamic libraries (.so/.dll)
  - File watcher detects changes and reloads automatically
  - State preservation across reloads (optional serialization)
  - Graceful error handling - script errors don't crash the game
- [ ] **Inspector Integration** - Automatic UI for script fields
  - `#[inspectable]` attribute for field customization
  - Auto-generated editors for: f32 (sliders), Vec2/Vec3, bool, enums, String
  - Live editing - changes reflect immediately in running game
  - Component ordering in inspector (priority attribute)
- [ ] **Script API** - Developer-friendly interfaces
  - `self.entity()` - Get owning entity ID
  - `self.get_component::<T>()` / `self.set_component()` - Component access
  - `self.spawn_entity()` / `self.destroy_entity()` - Entity lifecycle
  - `self.query::<T>()` - World queries from scripts
  - Event system: `self.on_collision_enter()`, `self.on_trigger_stay()`, etc.
- [ ] **Editor Scripting Workflow** - First-class IDE experience
  - New script template generation (via editor or CLI)
  - Script debugging support (breakpoints in hot-reloaded code)
  - Script dependency management (use other scripts as fields)
  - Built-in script documentation from doc comments

**Technical Implementation:**
- `crates/scripting/` - Scripting system crate (NEW)
  - `Script` trait - Core interface for all scripts
  - `ScriptManager` - Hot-reload, compilation, lifecycle
  - `ScriptContext` - Safe API for scripts to interact with world
  - `ScriptHost` - Dynamic library loading via `libloading`
- `engine_core/src/localization/` - Localization system (integrated)
  - `LocalizationManager` - String lookup, locale management
  - `FontMapping` - Language-to-font configuration
- `editor/src/inspector/script_inspector.rs` - Auto-generated UI for script fields
- **Dynamic library loading** via `libloading` crate

**Behavior Migration (ECS Cleanup):**
- **Current Issue:** Hard-coded behaviors (`PlayerPlatformer`, `ChaseTagged`, etc.) in `ecs` crate are inflexible and overlap with scripting
- **Migration Path:**
  1. Move existing behaviors to `scripting/src/builtins/` as reference script implementations
  2. `ecs` crate keeps only the `Behavior` marker trait (minimal footprint)
  3. `ecs` crate deprecates `behavior.rs` module (moved to `scripting`)
  4. Built-in behaviors become pre-compiled script examples
- **Result:** Single source of truth for behavior logic - everything is a script

**Example Developer Experience:**
```rust
// hello_scripts.rs
#[derive(Script, Default)]
pub struct PlayerController {
    #[inspectable(slider(0.0..200.0))]
    speed: f32,
    jump_force: f32,
}

impl Script for PlayerController {
    fn on_start(&mut self, ctx: &mut ScriptContext) {
        ctx.log("Player ready!");
    }
    
    fn on_update(&mut self, ctx: &mut ScriptContext) {
        // Visual/frame logic (animations, input reading)
        if ctx.input.is_key_pressed(KeyCode::Space) {
            self.jump(ctx);
        }
    }
    
    fn on_physics(&mut self, ctx: &mut ScriptContext) {
        // Physics logic (forces, velocities)
        let move_input = ctx.input.axis(Axis::Horizontal);
        ctx.physics.apply_force(Vec2::new(move_input * self.speed, 0.0));
    }
}
```

**Milestone:** Create a script, attach to entity, edit fields in inspector, modify code and see changes immediately without restart

---

### Phase 3: Localization System (i18n) 🎯 PRIORITY

**Goal:** Support multiple languages for gameplay and editor UI

**Supported Languages:**
- **English (US)** - `Futureworld-AZwJ.ttf` font
- **Pirate** - `BlackSamsGold-ej5e.ttf` font

**Localization Features:**
- [ ] **String Table System** - Key-value localization storage
  - JSON/RON format for translation files
  - Namespaced keys (e.g., `gameplay.ui.start`, `editor.menu.file`)
  - Hot-reload support for translation files
  - Fallback to English when translation missing
- [ ] **Font Mapping** - Language-to-font configuration
  - Per-language font asset assignment
  - Font fallback chain for missing glyphs
  - Dynamic font loading based on active language
- [ ] **Runtime Language Switching** - Change language without restart
  - Editor language preference persistence
  - In-game language selection UI
  - Immediate UI text refresh on language change
- [ ] **Text Rendering Integration** - UI and renderer support
  - UILabel auto-translation from keys
  - Text component locale-aware rendering
  - Right-to-left (RTL) text support (future)
- [ ] **Editor Localization** - Full editor UI translation
  - Menu labels, tooltips, panel headers
  - Inspector property names and descriptions
  - Error messages and notifications

**Technical Implementation:**
- `engine_core/src/localization/` - Integrated localization (KISS/DRY)
  - `LocalizationManager` - Active locale and string lookup
  - `string_table.rs` - Key-value storage with hot-reload
  - `font_mapping.rs` - Language-to-font asset resolution
- `ui/src/components/localized_label.rs` - `LocalizedLabel` component for auto-translation
- Translation files: `assets/i18n/en-US.json`, `assets/i18n/pirate.json`
- **Why integrated?** Localization is configuration/data, similar to scenes/assets which `engine_core` already manages. Avoids a separate crate just for JSON loading.

**Milestone:** Toggle between English and Pirate in editor; same text renders with different fonts

---

### Phase 4: Sprite Animation System 🎯 PRIORITY

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

### Phase 4: Asset Pipeline & Management

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

### Phase 5: Advanced Editor Features

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

### Phase 6: Platform & Deployment

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

**Overall Status:** 53 total items (3 completed, 50 remaining - 6% resolution rate this update)

**Priority Order:** Address the biggest risks first (stability, architecture, and data loss) before lower-impact improvements.

### High Priority (2 items) ⚠️

**ecs (2 items):**
- [ ] **PATTERN-001: ECS archetype storage uses trait-object interface** - Violates archetype pattern principles
  - Location: `ecs/src/component.rs:240`, `ecs/src/archetype.rs:235`
  - Issue: Components boxed as `Box<dyn Component>` before storage, requiring runtime downcasting via `as_any()`. Negates cache locality benefits of archetype storage.
  - Fix: Store components as raw bytes directly from concrete types; remove `Box<dyn Component>` from archetype interface
  - See: `crates/ecs/TECH_DEBT.md` Pattern Drift section
  
- [ ] **PATTERN-002: ECS defaults to Legacy storage despite archetype claims** - API contract violation
  - Location: `ecs/src/world.rs:27`, `ecs/src/component.rs:390`
  - Issue: `World::default()` uses `LegacyComponentStorage` (HashMap) despite docs claiming "archetype-based ECS". Users must explicitly call `World::new_optimized()`.
  - Fix: Make `use_archetype_storage: true` the default, or rename methods to reflect actual behavior
  - See: `crates/ecs/TECH_DEBT.md` Pattern Drift section

### Medium Priority (10 items)

**engine_core (3 items remaining, 3 completed):**
- [ ] **EngineApplication cleanup** - Reduce from 346 to ~150 lines
  - Location: `engine_core/src/application.rs`
  - Note: This is deprecated code, migrate to Game API
- [x] **DRY-001: Duplicate AudioManager placeholder pattern** - ✅ COMPLETED
  - Fixed: AudioManager properly integrated through GameContext
- [x] **DRY-003: Duplicate GameContext creation pattern** - ✅ COMPLETED  
  - Fixed: GameContext creation consolidated in GameRunner
- [x] **SRP-001: GameRunner still has multiple responsibilities** - ✅ COMPLETED
  - Fixed: Extracted 5 managers (GameLoopManager, UIManager, RenderManager, WindowManager, SceneManager)
  - game.rs reduced from 862 → 555 lines (-36%)
- [ ] **SRP-003: EngineApplication duplicates GameRunner functionality** - Both manage game lifecycle
  - Location: `application.rs:346 lines` vs `game.rs:555 lines`
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

**ecs (3 items):**
- [ ] **ARCH-004: Hard-coded behaviors should move to scripting crate** - Behaviors overlap with Script system
  - Location: `ecs/src/behavior.rs` (PlayerPlatformer, ChaseTagged, etc.)
  - Issue: Hard-coded behaviors are inflexible; Scripting system replaces them
  - Fix: Migrate behaviors to `scripting/src/builtins/`, keep only marker trait in ECS
  - See: Phase 2 Scripted Behaviors in roadmap
- [ ] **SRP-002: ComponentStorage enum handles both storage types** - Single enum for two storage strategies
  - Location: `ecs/src/component.rs`
  - Fix: Consider trait objects or separate types for Legacy vs Archetype
- [ ] **SRP-003: TransformHierarchySystem does double iteration** - Separate root and child passes
  - Location: `ecs/src/hierarchy_system.rs:87-118`
  - Fix: Reorganize to single pass with better data flow

**common (1 item):**
- [ ] **ARCH-001: CameraUniform duplicated in renderer crate** - Exists in both common and renderer
  - Fix: Use `common::CameraUniform` everywhere, remove renderer copy

**audio (1 item):**
- [ ] **ARCH-001: No streaming for large music assets** - All audio loaded eagerly into memory
  - Fix: Add optional streaming path for long tracks (keep cache for SFX)

**input (1 item):**
- [ ] **TEST-001: Missing input timing + dead zone tests** - Gamepad dead zone and frame timing unvalidated
  - Fix: Add tests for dead zone normalization and frame-accurate event timing

**physics (1 item):**
- [ ] **TEST-001: Missing friction/kinematic/sensor validation** - Core physics materials and triggers untested
  - Fix: Add coverage for friction/restitution, kinematic bodies, and sensors

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

**audio (2 items):**
- [ ] **DOC-001: Spatial audio limitations undocumented** - Attenuation-only behavior not explicit in API docs
- [ ] **FEATURE-001: Music mixing/bus support** - Track need for multi-track music or mixing buses

**physics (2 items):**
- [ ] **DRY-002: Repeated body builder pattern in add_rigid_body** - Similar builder code for body types
- [ ] **SRP-001: PhysicsWorld handles too many rapier types** - Direct exposure of rapier internals

**input (2 items):**
- [ ] **DRY-001: Repeated input state tracking pattern** - Keyboard/mouse/gamepad similar structure
- [ ] **DRY-003: Repeated unbind logic in input_mapping.rs** - Unbinding has duplicate patterns

**common (1 item):**
- [ ] **DOC-001: Macros module usage is undocumented** - Clarify intended consumers and examples

**common (2 items):**
- [ ] **DRY-001: Duplicate matrix construction pattern in transform.rs** - Similar to GlobalTransform2D pattern
- [ ] **ARCH-002: Camera2D vs renderer sprite_data Camera2D** - Verify Camera2D is canonical source

**Cross-Crate/System (3 items):**
- [ ] **ecs: DRY-002: Duplicate error variants** - Consolidate error types across crates
- [ ] **ARCH-001: Reduce coupling** - Event bus or message system for cross-crate communication
- [ ] **ARCH-002: Configuration system** - Centralized config management

**Total Remaining:** 50 items

---

## Success Metrics

### Phase 1 (Scene Editor)
- Create a complete level in 30 minutes (vs 2 hours manually)
- Zero manual RON editing required
- Visual feedback for all operations

### Phase 2 (Scripted Behaviors)
- Script hot-reload in < 500ms on change
- Zero boilerplate for simple scripts (just impl Script)
- All public fields show in inspector automatically
- Script errors are caught and logged, game continues running
- Can prototype a simple game without recompiling the engine

### Phase 3 (Localization)
- Switch languages in < 100ms without restart
- 100% of editor UI strings externalized
- Pirate translation coverage for all core features
- Font fallback works for missing glyphs

### Phase 4 (Animation)
- Support 100+ frame animations at 60 FPS
- < 16ms frame time with animated sprites
- Seamless animation transitions

### Phase 4 (Asset Pipeline)
- Import 50+ assets in < 5 minutes
- Automatic atlas packing with < 10% wasted space
- Real-time thumbnail generation

### Phase 5 (Advanced Tools)
- Visual scripts run at 95% of native code speed
- Particle systems with 1000+ particles at 60 FPS
- < 5ms overhead for physics debug rendering

### Phase 6 (Platform Support)
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

### Scripted Behaviors
1. **Scripts are just Rust** - No DSL, no magic - pure Rust structs implementing a trait
2. **Hot-reload is essential** - Developer iteration speed > all other concerns
3. **Clear lifecycle naming** - `on_update` vs `on_physics` (not confusing update/fixed_update)
4. **Zero-cost abstractions** - Script API compiles to direct ECS calls, no overhead
5. **Inspector as documentation** - Script fields document themselves via attributes
6. **Fail gracefully** - Script errors are caught and logged, never crash the game

### Localization
1. **Key-based strings** - Never hardcode display text, always use keys
2. **Context comments** - Include translator context in source files
3. **Font per locale** - Each language defines its primary font
4. **Fallback chain** - Missing translation → English → key name
5. **Editor-first** - Editor UI must be fully localizable

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

## New Feature Crate Structure

### Phase 2: Scripted Behaviors
```
crates/scripting/              (NEW - separate crate)
├── src/
│   ├── lib.rs                 # Script trait, exports
│   ├── script_manager.rs      # Hot-reload, compilation
│   ├── script_context.rs      # Safe world API for scripts
│   ├── script_host.rs         # Dynamic lib loading
│   ├── inspector.rs           # Script field UI generation
│   └── builtins/              # Reference implementations
│       ├── player_platformer.rs
│       ├── chase_tagged.rs
│       └── ...
└── Cargo.toml

crates/ecs/src/                (CLEANUP - remove behaviors)
├── lib.rs                     # Remove behavior module re-export
├── component.rs               # Keep: Behavior marker trait only
└── behavior.rs                # DEPRECATED - migrate to scripting crate

crates/editor/src/
├── inspector/
│   ├── mod.rs
│   └── script_inspector.rs    # Script field rendering
```

### Phase 3: Localization
```
crates/engine_core/src/        (INTEGRATED - not separate crate)
├── localization/
│   ├── mod.rs                 # LocalizationManager
│   ├── string_table.rs        # Key-value storage
│   └── font_mapping.rs        # Language -> Font
├── lib.rs                     # Re-export LocalizationManager

assets/i18n/
├── en-US.json                 # English strings
└── pirate.json                # Pirate strings

crates/ui/src/
└── components/
    └── localized_label.rs     # LocalizedLabel component
```

**Why this structure?**
- **Scripting = New Crate**: Hot-reload is complex (dynamic libs, file watching). Clean separation, games opt-in.
- **Localization = Integrated**: It's configuration/data (like scenes). `engine_core` already manages assets/scenes.
- **Behaviors = Migrated**: Hard-coded behaviors move from `ecs` to `scripting/builtins/`. Single source of truth.

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
- Managers extracted: GameLoopManager, UIManager, RenderManager, WindowManager, SceneManager
- Files extracted: game_config.rs, contexts.rs, ui_integration.rs, scene_manager.rs, behavior_runner.rs
- SRP refactoring: game.rs reduced from 862 → 555 lines (-36%)
- Test quality: 0 TODOs, 155+ assertions

</details>
