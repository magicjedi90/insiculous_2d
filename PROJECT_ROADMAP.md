# Insiculous 2D - Project Roadmap

## Current Status (February 2026) - PRODUCTION READY

**Engine Status:** Core systems complete, production-ready for 2D games
**Editor Status:** Foundation complete (UI framework, viewport, read-only inspector, hierarchy), needs functional integration

### Test Coverage
| System | Tests | Status |
|--------|-------|--------|
| ECS | 110 | 100% pass |
| Input | 56 | 100% pass |
| Engine Core | 67 | 100% pass |
| Physics | 28 | 100% pass |
| Renderer | 62 | 100% pass |
| Audio | 3 | 100% pass |
| UI | 60 | 100% pass |
| Editor | 148 | 100% pass |
| Editor Integration | 14 | 100% pass |
| ECS Macros | 3 | 100% pass |
| Common | 26 | 100% pass |

**Total:** 578/578 tests passing (100% success rate)
**Code Quality:** 0 TODOs, 155+ assertions, 30 ignored (GPU/window only)

### Completed Engine Features
- Simple Game API (`Game` trait, `run_game()`)
- ECS with HashMap-based per-type storage and type-safe queries
- WGPU 28.0.0 sprite rendering with batching
- Rapier2d physics with presets (platformer, top-down, etc.)
- Scene serialization (RON format) with prefabs
- Scene graph hierarchy with transform propagation
- Immediate-mode UI framework (buttons, sliders, panels)
- Event-based input system (keyboard, mouse, gamepad)
- Spatial audio with rodio backend
- Asset manager with texture loading

### Completed Editor Foundation
- Dockable panel system (Scene View, Hierarchy, Inspector, Asset Browser, Console)
- Menu bar with dropdowns (File, Edit, View, Entity)
- Toolbar with tool selection (Select, Move, Rotate, Scale + keyboard shortcuts)
- Scene viewport with camera pan/zoom and grid overlay
- Entity picking (click-to-select, rectangle selection, depth sorting)
- Transform gizmos (translate mode with world-space delta)
- Read-only component inspector (generic serde-based field display)
- Editable field widgets (sliders, Vec2 inputs, checkboxes, color pickers) - **not yet wired up**
- Component editor result types (TransformEditResult, SpriteEditResult, etc.) - **not yet wired up**
- Hierarchy panel with tree view, expand/collapse, name resolution
- Selection system (single, multi, toggle, primary entity)

**Verification:** `cargo run --example hello_world` (platformer demo), `cargo run --example editor_demo --features editor` (editor UI)

---

## Roadmap: Editor-First Development

**Philosophy:** A game engine is only as good as its editor. The priority is making the editor a functional, productive tool for building games - on par with what developers expect from Unity and Godot. Scripting, animation, and platform support come after the editor is usable.

---

### Phase 1: Functional Editor 🎯 CURRENT PRIORITY

**Goal:** An editor you can actually build a game scene in. Select entities, edit their properties, see changes live, save your work.

#### 1A. Dev Mode Integration ✅ COMPLETE (February 2026)
The editor is a mode of the engine, not a separate example binary. One function call (`run_game_with_editor(game, config)`) overlays the full editor UI on any game.

- [x] **Editor cargo feature** - `cargo run --example editor_demo --features editor` activates editor mode
  - `editor_integration` crate wraps any `Game` with `EditorGame<G>` transparent wrapper
  - Feature flag `editor` on root crate gates `editor_integration` dependency
  - Editor panels render on top of the game viewport
  - Editor-only code compiles out without the `editor` feature
- [x] **EditorGame wrapper** - Editor lifecycle hooks via `Game` trait delegation
  - `EditorGame<G>` implements `Game`, intercepting init/update/on_key_pressed
  - Editor gets its own update pass (before inner game update)
  - Font loading, transform hierarchy, menu bar, toolbar, dock panels, gizmo, tool shortcuts all automatic
  - Minimum window size enforced (1024x720) for editor usability
- [x] **Simplified editor_demo.rs** - 351 lines → 66 lines, just entity setup + `run_game_with_editor()`
- [x] **Removed hard-coded Escape exit** - Escape key now flows to `Game::on_key_pressed()` like any other key
- [x] **Removed unused engine_core dep from editor crate** - Cleaner dependency graph

#### 1B. Property Editing with Writeback
The editable inspector widgets exist but aren't connected. Inspector changes must actually modify ECS components.

- [x] **Wire up component editors** - Connect existing `edit_transform2d()`, `edit_sprite()`, etc. to ECS writeback
  - Inspector panel calls component-specific editors instead of read-only `inspect_component()`
  - Edit results (`TransformEditResult`, `SpriteEditResult`, etc.) applied via `world.get_mut::<T>()`
  - Changes visible immediately in viewport (live preview)
  - Also wired up: RigidBody, Collider, AudioSource editing with writeback
- [x] **RigidBody and Collider editing** - Wire up `edit_rigid_body()` and `edit_collider()` with physics sync
  - Inspector writeback updates ECS components; `PhysicsSystem::update()` auto-syncs to rapier
  - Body type and shape editing are read-only (enum dropdown/shape editor not yet available)
- [x] **AudioSource editing** - Wire up `edit_audio_source()` for volume, pitch, spatial settings
- [x] **Rotate and Scale gizmos** - Gizmo interaction writes back to `Transform2D`
  - Rotate gizmo: circular handle, angle delta applied to `Transform2D.rotation`
  - Scale gizmo: corner handles, scale delta applied to `Transform2D.scale` (clamped to min 0.01)

#### 1C. Play / Pause / Stop
Run the game inside the editor, pause to inspect state, stop to reset to the saved scene.

- [ ] **Editor play states** - `EditorPlayState` enum: `Editing`, `Playing`, `Paused`
  - Play button: Snapshot current scene state, begin running game logic (behaviors, physics, scripts)
  - Pause button: Freeze game logic, allow inspection and property editing
  - Stop button: Restore scene to pre-play snapshot, return to Editing state
- [ ] **Scene snapshot/restore** - Serialize entire world state before play, deserialize on stop
  - Must capture all component data, hierarchy relationships, physics state
  - Fast path: clone world rather than serialize/deserialize if possible
- [ ] **Visual play state indicator** - Clear UI showing current mode
  - Tinted viewport border (blue = editing, green = playing, yellow = paused)
  - Play/Pause/Stop buttons in toolbar
  - Keyboard shortcuts: Ctrl+P (play/pause toggle), Ctrl+Shift+P (stop)
- [ ] **Input routing** - Game receives input only during Playing state
  - Editing/Paused: Input goes to editor (pan, select, gizmo, etc.)
  - Playing: Input goes to game, editor hotkeys still work (Ctrl+P to pause)

#### 1D. Entity Operations
Create, delete, duplicate, and reparent entities from the editor UI.

- [ ] **Create entity** - Wire up Entity menu items
  - Create Empty: new entity with Transform2D at viewport center
  - Create with Sprite: entity + Transform2D + Sprite
  - Create physics bodies: entity + Transform2D + RigidBody + Collider (with presets)
  - New entity selected automatically after creation
- [ ] **Delete entity** - Delete key or menu action
  - Remove entity and all components from world
  - Remove from physics world if has RigidBody/Collider
  - Reparent children to deleted entity's parent (or make them roots)
  - Clear from selection
- [ ] **Duplicate entity** - Ctrl+D
  - Deep copy all components
  - Offset position slightly so duplicate is visible
  - Duplicate children recursively (preserve hierarchy)
  - Select the duplicate
- [ ] **Hierarchy drag-and-drop reparenting** - Drag entities in hierarchy to change parent
  - Visual drop target indicator (above, below, as child)
  - Reparent updates transform hierarchy (recalculate local transform from global)
  - Prevent circular reparenting (can't parent to own descendant)
  - Drop at root level to unparent

#### 1E. Component Add/Remove
Attach and detach components on selected entities.

- [ ] **Add Component button** - In inspector panel below existing components
  - Dropdown/searchable list of available component types
  - Component added with default values
  - Uses ComponentRegistry for available types
- [ ] **Remove Component button** - Per-component header with remove (X) button
  - Confirmation for components with dependencies (e.g., removing RigidBody also removes Collider)
  - Physics bodies cleaned up when physics components removed

#### 1F. Undo/Redo
Every editor operation must be reversible.

- [ ] **Command pattern** - `EditorCommand` trait with `execute()` and `undo()` methods
  - `CommandHistory` struct with undo/redo stacks
  - Commands: `SetComponentField`, `CreateEntity`, `DeleteEntity`, `DuplicateEntity`, `ReparentEntity`, `AddComponent`, `RemoveComponent`, `TransformGizmo`
- [ ] **Command merging** - Continuous gizmo drags merge into single command
  - Slider drags merge while mouse is held
  - Gizmo movements merge while actively dragging
- [ ] **Undo/Redo UI** - Ctrl+Z / Ctrl+Shift+Z, Edit menu items
  - Display current command name in status bar

#### 1G. Scene Save/Load
Persist scene changes to disk and reload them.

- [ ] **Save scene** - Ctrl+S serializes world to RON file
  - Serialize all entities with components
  - Preserve hierarchy relationships
  - Compatible with existing `SceneData` format from `scene_loader.rs`
  - Dirty flag tracking (unsaved changes indicator in title bar)
- [ ] **Load scene** - File > Open loads RON scene into editor
  - Clear current world, load new scene
  - Reset editor state (selection, camera position)
  - Recent files list
- [ ] **Editor state persistence** - Save/restore editor preferences
  - Camera position and zoom
  - Panel layout and sizes
  - Last opened scene path
  - Stored separately from scene data (e.g., `.editor_state.json`)

**Phase 1 Milestone:** Open `hello_world.scene.ron` in the editor, move platforms with gizmos, edit physics properties in inspector, press Play to test, Stop to reset, Save changes to disk.

**Success Metrics:**
- Build a complete level in 30 minutes (vs 2 hours editing RON manually)
- Zero manual RON editing required for scene authoring
- All property changes visible immediately in viewport
- Undo any operation with Ctrl+Z
- Play/Stop cycle in < 500ms

---

### Phase 2: Productive Editor

**Goal:** Quality-of-life features that make the editor efficient for daily use. Asset management, multi-editing, prefabs, and a console.

#### 2A. Asset Browser
- [ ] **Asset panel implementation** - Visual browser for project assets
  - Grid/list view toggle for asset display
  - Thumbnail generation for textures and sprites
  - Search bar with filtering by type (textures, scenes, audio, fonts)
  - Folder navigation with breadcrumb path
- [ ] **Drag-and-drop asset assignment** - Drag texture onto Sprite component in inspector
  - Drag texture onto entity in viewport to assign sprite
  - Drag scene file to load
  - Drag audio file onto AudioSource component
- [ ] **Asset import** - Drop files into asset browser to import
  - Copy to project assets directory
  - Auto-detect type and apply default import settings
  - Watch for external changes and re-import

#### 2B. Multi-Object Editing
- [ ] **Shared property editing** - Select multiple entities, edit common properties
  - Inspector shows fields shared across all selected entities
  - Mixed values shown as "--" or indeterminate state
  - Edits applied to all selected entities simultaneously
- [ ] **Multi-transform** - Gizmo operates on all selected entities
  - Move: all selected entities translate by same delta
  - Rotate: all selected entities rotate around selection center
  - Scale: all selected entities scale relative to selection center

#### 2C. Copy/Paste
- [ ] **Copy/Cut/Paste entities** - Ctrl+C, Ctrl+X, Ctrl+V
  - Clipboard holds serialized entity data
  - Paste at mouse position or viewport center
  - Paste preserves component values and hierarchy
  - Cut = Copy + Delete

#### 2D. Prefab System
- [ ] **Prefab creation** - Save entity (with children) as reusable template
  - Right-click entity > Create Prefab
  - Prefab stored as RON file in assets
  - Prefab instances track overrides (like Unity prefab variants)
- [ ] **Prefab instantiation** - Drag prefab from asset browser to create instance
  - Instance linked to source prefab
  - Override individual properties per instance
  - Apply instance changes back to prefab
- [ ] **Prefab updates** - Modify prefab, all instances update
  - Only non-overridden fields update
  - Visual indicator for overridden properties in inspector

#### 2E. Console Panel
- [ ] **Log output** - Display engine log messages in Console panel
  - Color-coded by level: Info (white), Warn (yellow), Error (red), Debug (gray)
  - Scrollable with auto-scroll to bottom
  - Clear button
  - Log level filter dropdown
- [ ] **Search and filter** - Filter log messages by text or source
  - Regex support for advanced filtering
  - Collapse repeated messages with count

#### 2F. Hierarchy Improvements
- [ ] **Search/filter** - Search bar in hierarchy panel
  - Filter entities by name, component type, or tag
  - Highlight matches, collapse non-matching branches
- [ ] **Context menu** - Right-click entity for actions
  - Create Child, Duplicate, Delete, Rename
  - Copy/Cut/Paste
  - Create Prefab
  - Focus in Viewport

#### 2G. Localization System (i18n)
- [ ] **String table system** - Key-value localization storage
  - JSON/RON format for translation files
  - Namespaced keys (e.g., `gameplay.ui.start`, `editor.menu.file`)
  - Hot-reload support for translation files
  - Fallback to English when translation missing
- [ ] **Font mapping** - Per-language font assignment
  - Supported: English (Futureworld-AZwJ.ttf), Pirate (BlackSamsGold-ej5e.ttf)
  - Font fallback chain for missing glyphs
- [ ] **Runtime language switching** - Change language without restart
  - Editor language preference persistence
  - Immediate UI text refresh on language change
- [ ] **Editor UI localization** - All editor strings externalized
  - Menu labels, tooltips, panel headers
  - Inspector property names and descriptions

**Phase 2 Milestone:** Import sprites through asset browser, create prefab templates, edit multiple entities at once, view logs in console, switch editor language to Pirate.

**Success Metrics:**
- Import 50+ assets in < 5 minutes with drag-and-drop
- Edit shared properties across 10 selected entities simultaneously
- Prefab changes propagate to all instances instantly
- Switch languages in < 100ms without restart

---

### Phase 3: Scripting & Animation

**Goal:** Runtime behaviors via scripts and 2D animation tooling. These depend on a functional editor for inspector integration and workflow.

#### 3A. Scripted Behaviors (Script Components)
Unity/Godot-style script components - attach behavior scripts to entities with hot-reload support.

- [ ] **Script trait system** - Rust-native script components
  - `Script` trait with lifecycle hooks (`on_start`, `on_update`, `on_physics`, `on_destroy`)
  - Clear separation: `on_update` for visual/frame logic, `on_physics` for physics/movement
  - Automatic component registration with ECS
  - Access to `ScriptContext` (entity, world queries, delta time, input)
- [ ] **Hot-reload support** - Iterate without recompiling the game
  - Scripts compiled as dynamic libraries (.so/.dll)
  - File watcher detects changes and reloads automatically
  - State preservation across reloads (optional serialization)
  - Graceful error handling - script errors don't crash the game
- [ ] **Inspector integration** - Automatic UI for script fields
  - `#[inspectable]` attribute for field customization
  - Auto-generated editors for: f32 (sliders), Vec2/Vec3, bool, enums, String
  - Live editing - changes reflect immediately in running game
- [ ] **Script API** - Developer-friendly interfaces
  - `self.entity()`, `self.get_component::<T>()`, `self.set_component()`
  - `self.spawn_entity()`, `self.destroy_entity()`
  - `self.query::<T>()`, event system (`on_collision_enter`, etc.)

**Behavior migration:** Hard-coded behaviors (`PlayerPlatformer`, `ChaseTagged`, etc.) move from `ecs` crate to `scripting/src/builtins/` as reference script implementations. ECS crate keeps only the `Behavior` marker trait.

```rust
#[derive(Script, Default)]
pub struct PlayerController {
    #[inspectable(slider(0.0..200.0))]
    speed: f32,
    jump_force: f32,
}

impl Script for PlayerController {
    fn on_update(&mut self, ctx: &mut ScriptContext) {
        if ctx.input.is_key_pressed(KeyCode::Space) {
            self.jump(ctx);
        }
    }

    fn on_physics(&mut self, ctx: &mut ScriptContext) {
        let move_input = ctx.input.axis(Axis::Horizontal);
        ctx.physics.apply_force(Vec2::new(move_input * self.speed, 0.0));
    }
}
```

#### 3B. Sprite Animation System
- [ ] **Sprite sheet importer** - Import and slice sprite sheets
  - Automatic grid-based slicing
  - Manual frame definition with visual editor
  - Support for: PNG, Aseprite files (.ase/.aseprite)
- [ ] **Animation timeline editor** - Keyframe-based animation
  - Dopesheet view for frame timing
  - Curve editor for tweening/easing
  - Preview window with playback controls
- [ ] **Animation controller** - State machine for animations
  - Animation states (Idle, Run, Jump, Attack)
  - Transitions with conditions (parameters, triggers)
  - Blend trees for smooth transitions
- [ ] **Animation components** - ECS integration
  - `AnimationPlayer` - Play animations on entities
  - `Animator` - State machine controller
  - `SpriteSheet` - Reference to sprite sheet asset

**Technical implementation:**
```
crates/scripting/              (NEW - separate crate)
├── src/
│   ├── lib.rs                 # Script trait, exports
│   ├── script_manager.rs      # Hot-reload, compilation
│   ├── script_context.rs      # Safe world API for scripts
│   ├── script_host.rs         # Dynamic lib loading
│   └── builtins/              # Migrated behaviors
│       ├── player_platformer.rs
│       └── chase_tagged.rs

crates/animation/              (NEW - separate crate)
├── src/
│   ├── lib.rs                 # Animation system
│   ├── clip.rs                # AnimationClip
│   ├── controller.rs          # State machine
│   └── sprite_sheet.rs        # Sprite sheet asset
```

**Phase 3 Milestone:** Create a script, attach to entity, edit fields in inspector, hot-reload on save. Animate a character with idle/run/jump controlled by physics.

**Success Metrics:**
- Script hot-reload in < 500ms on change
- Zero boilerplate for simple scripts (just impl Script)
- Script errors caught and logged, game continues running
- 100+ frame animations at 60 FPS
- Seamless animation transitions

---

### Phase 4: Advanced Editor Tools

**Goal:** Professional-grade development tools built on top of the functional editor.

#### 4A. Physics Debugger
- [ ] **Collider wireframe rendering** - Overlay physics shapes on viewport
  - Box, circle, capsule outlines
  - Color-coded by body type (dynamic = blue, static = green, kinematic = yellow)
- [ ] **Velocity vector visualization** - Arrow showing body velocity
- [ ] **Collision point highlighting** - Flash on contact points
- [ ] **Toggle overlay** - Quick on/off for physics debug rendering

#### 4B. Profiler Integration
- [ ] **Frame time graph** - Real-time graph with 16.6ms target line
- [ ] **System timing breakdown** - ECS, Physics, Render, UI time per frame
- [ ] **Draw call and batch count** - Rendering statistics
- [ ] **Memory usage tracking** - Per-system allocation tracking

#### 4C. Tilemap Editor
- [ ] **Paintbrush and fill tools** - Paint tiles in viewport
- [ ] **Multiple layers** - Background, foreground, collision layers
- [ ] **Autotiling support** - Automatic tile neighbor matching
- [ ] **Tile property editing** - Per-tile collision, metadata

#### 4D. Particle System Editor
- [ ] **Emitter configuration** - Rate, shape, burst settings
- [ ] **Particle properties** - Lifetime, size, color, velocity curves
- [ ] **Curve editors** - Visual curve editing for property animation
- [ ] **Real-time preview** - Live particle preview in viewport

#### 4E. Visual Scripting
- [ ] **Node graph editor** - Visual programming interface
  - Event nodes (OnStart, OnUpdate, OnCollision)
  - Action nodes (PlaySound, SetPosition, SpawnEntity)
  - Flow control (Branch, Sequence, Loop)
  - Variable system (local, global, blackboard)

#### 4F. Asset Pipeline
- [ ] **Sprite atlas generator** - Automatic texture packing
  - Bin packing algorithms (MaxRects, Shelf)
  - Automatic sprite referencing updates
- [ ] **Asset import pipeline** - Automated import with caching
  - Watch folders for auto-import
  - Import presets for common asset types
  - Background import with progress tracking
- [ ] **Audio asset manager** - Waveform preview and editing

**Phase 4 Milestone:** Debug physics collisions visually, profile frame time, create tilemaps and particle effects.

**Success Metrics:**
- < 5ms overhead for physics debug rendering
- Particle systems with 1000+ particles at 60 FPS
- Automatic atlas packing with < 10% wasted space

---

### Phase 5: Platform & Deployment

**Goal:** Ship games to multiple platforms.

- [ ] **Web (WASM) export** - Browser-based games
  - WASM build pipeline
  - WebGL2 rendering backend
  - Touch input mapping
  - Asset loading via HTTP/fetch
- [ ] **Mobile export** - iOS and Android
  - Touch gesture support (tap, drag, pinch)
  - Mobile-optimized UI scaling
  - App bundle generation
- [ ] **Desktop optimization** - Windows, macOS, Linux
  - Build scripts and installers
  - Steam/Epic integration
  - Controller mapping database
- [ ] **Hot reloading** - Live asset and code reloading
  - Texture hot-reload
  - Scene hot-reload
  - Script hot-reload

**Phase 5 Milestone:** Deploy `hello_world.rs` to Web, mobile, and desktop.

**Success Metrics:**
- Web: Load and start in < 5 seconds
- Mobile: 60 FPS on mid-range devices (2022+)
- Desktop: Package and deploy in < 1 minute

---

## Technical Debt (Remaining)

**Overall Status:** 53 total items (8 completed, 45 remaining)

**Priority Order:** Address the biggest risks first (stability, architecture, and data loss) before lower-impact improvements.

### High Priority (0 items — all resolved)

**ecs (2 items resolved):**
- [x] **PATTERN-001: ECS archetype storage uses trait-object interface** - RESOLVED (February 2026)
  - Resolution: Broken archetype storage code removed entirely. ECS now uses single HashMap-based per-type storage (`ComponentStore`). Proper archetype storage is deferred as a future ground-up rewrite.
- [x] **PATTERN-002: ECS defaults to Legacy storage despite archetype claims** - RESOLVED (February 2026)
  - Resolution: Dual-storage system removed. `World::new_optimized()` and `ComponentStorage` enum deleted. Single storage path via `ComponentRegistry` → `ComponentStore` (HashMap-based). Documentation updated to reflect actual storage.

### Medium Priority (7 items)

**engine_core (0 items remaining, 6 completed):**
- [x] **DRY-001: Duplicate AudioManager placeholder pattern** - COMPLETED
- [x] **DRY-003: Duplicate GameContext creation pattern** - COMPLETED
- [x] **SRP-001: GameRunner still has multiple responsibilities** - COMPLETED
- [x] **SRP-003: EngineApplication duplicates GameRunner functionality** - RESOLVED (February 2026)
  - Resolution: `EngineApplication` deleted entirely (`application.rs` removed). Game API is the only API.
- [x] **ARCH-001: Dual API pattern creates confusion** - RESOLVED (February 2026)
  - Resolution: `EngineApplication` deleted, deprecated re-export removed from `lib.rs` and `prelude.rs`.

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

**ecs (1 item remaining, 2 resolved):**
- [ ] **ARCH-004: Hard-coded behaviors should move to scripting crate** - Behaviors overlap with Script system
  - Location: `ecs/src/behavior.rs` (PlayerPlatformer, ChaseTagged, etc.)
  - Fix: Migrate behaviors to `scripting/src/builtins/`, keep only marker trait in ECS
  - See: Phase 3A Scripted Behaviors
- [x] **SRP-002: ComponentStorage enum handles both storage types** - RESOLVED (February 2026)
  - Resolution: `ComponentStorage` enum deleted. Single `ComponentStore` (HashMap-based) is the only storage type.
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
- [ ] DRY-002: Duplicate coordinate transformation logic in ui_integration.rs
- [ ] DRY-004: Repeated hex color parsing error handling
- [ ] DRY-005: Duplicate surface error recovery pattern
- [ ] SRP-002: BehaviorRunner handles multiple behavior types inline
- [ ] KISS-002: Over-engineered lifecycle state machine (8 states, simplify to 3)
- [ ] ARCH-002: Timer vs GameLoopManager overlap

**ecs (4 items, 1 resolved):**
- [ ] DRY-001: Repeated entity existence checks (7+ duplicate patterns)
- [ ] DRY-003: Duplicate matrix computation in GlobalTransform2D
- [ ] DRY-004: Repeated builder pattern in audio_components.rs
- [x] ARCH-003: Dead code marked but not removed — RESOLVED (February 2026): archetype dead code removed
- [x] ARCH-004: Dual storage systems add complexity — RESOLVED (February 2026): single storage path

**renderer (7 items):**
- [ ] DRY-001: Duplicate surface error handling in renderer.rs
- [ ] DRY-003: Duplicate render pass descriptor in sprite.rs
- [ ] DRY-004: Duplicate texture descriptor creation
- [ ] SRP-003: RenderPipelineInspector mixes logging with operation wrapping
- [ ] KISS-001: RenderPipelineInspector is over-engineered
- [ ] ARCH-001: Redundant device/queue accessors
- [ ] ARCH-002: Time struct in renderer crate is misplaced

**ui (4 items):**
- [ ] DRY-004: Repeated font check and placeholder fallback
- [ ] SRP-002: UIContext has large widget methods (button/slider 40+ lines each)
- [ ] KISS-001: WidgetPersistentState has unused flexibility
- [ ] ARCH-003: TextDrawData duplicates GlyphDrawData info

**audio (2 items):**
- [ ] DOC-001: Spatial audio limitations undocumented
- [ ] FEATURE-001: Music mixing/bus support

**physics (2 items):**
- [ ] DRY-002: Repeated body builder pattern in add_rigid_body
- [ ] SRP-001: PhysicsWorld handles too many rapier types

**input (2 items):**
- [ ] DRY-001: Repeated input state tracking pattern
- [ ] DRY-003: Repeated unbind logic in input_mapping.rs

**common (3 items):**
- [ ] DOC-001: Macros module usage undocumented
- [ ] DRY-001: Duplicate matrix construction pattern in transform.rs
- [ ] ARCH-002: Camera2D vs renderer sprite_data Camera2D

**Cross-Crate (3 items):**
- [ ] DRY-002: Duplicate error variants across crates
- [ ] ARCH-001: Reduce coupling (event bus for cross-crate communication)
- [ ] ARCH-002: Configuration system (centralized config management)

**editor (3 items):**
- [ ] MAGIC-001: Hardcoded slider ranges in component_editors.rs (13 locations)
- [ ] MAGIC-002: Widget ID formula constants in editable_inspector.rs (fragile multipliers)
- [ ] MAGIC-003: Layout dimensions in editable_inspector.rs (hardcoded widths/heights)

**Total Remaining:** 45 items

---

## Development Guidelines

### AI-Friendly Development
This engine is built for AI-assisted development. Every feature must be verifiable from the command line:
1. **CLI-testable** - All logic testable without GPU/window. `cargo test --workspace` validates everything.
2. **No manual testing** - If a feature can't be verified by `cargo test`, it needs a test. AI can't click buttons.
3. **Small, focused files** - Files over 600 lines should be split. AI context windows are limited.
4. **Explicit over implicit** - No magic numbers, hidden side effects, or clever tricks.
5. **Strong typing** - Enums over strings, newtypes over primitives. Compiler catches what AI might miss.
6. **Consistent patterns** - Use established patterns so AI can predict structure.
7. **Verify before claiming** - Always run `cargo test --workspace` before claiming work is done.

### Editor Architecture
1. **Feature-gated** - Editor code compiles out in release builds
2. **Use existing systems** - Leverage UI, ECS, and rendering
3. **Modular tools** - Each tool is a separate module
4. **Game preview mode** - Run game inside editor viewport
5. **Command pattern** - All operations undoable
6. **Live editing** - Property changes visible immediately

### Scripted Behaviors
1. **Scripts are just Rust** - No DSL, no magic - pure Rust structs implementing a trait
2. **Hot-reload is essential** - Developer iteration speed > all other concerns
3. **Clear lifecycle naming** - `on_update` vs `on_physics` (not confusing update/fixed_update)
4. **Zero-cost abstractions** - Script API compiles to direct ECS calls, no overhead
5. **Inspector as documentation** - Script fields document themselves via attributes
6. **Fail gracefully** - Script errors are caught and logged, never crash the game

---

## Quick Reference

**Commands:**
```bash
# Run all tests
cargo test --workspace

# Run game demo
cargo run --example hello_world

# Run editor demo
cargo run --example editor_demo --features editor
```

**Key Files:**
- `AGENTS.md` - AI agent guidance (high-level)
- `training.md` - API patterns and examples
- `PROJECT_ROADMAP.md` - This file (single source of truth)
- `crates/*/TECH_DEBT.md` - Per-crate technical debt
- `examples/hello_world.rs` - Reference implementation
- `examples/editor_demo.rs` - Editor demo (uses `run_game_with_editor`, requires `--features editor`)

---

## Archive: Completed Phases (2025)

<details>
<summary>Click to expand completed work</summary>

### Phase 1: Stabilization - COMPLETE
**Goal:** Make the engine safe and functional.
- Memory safety & lifetime issues resolved
- Thread-safe input handling with event queue
- Panic-safe system registry

### Phase 2: Core Features - COMPLETE
**Goal:** Make the engine usable for simple 2D games.
- Sprite rendering with WGPU 28.0.0 and batching
- ECS with type-safe queries
- Asset management with texture loading
- Rapier2d physics integration with presets
- Scene serialization (RON format) with prefabs

### Phase 3: Usability - COMPLETE
**Goal:** Make the engine productive for developers.
- Scene graph with parent-child relationships
- Audio system with spatial audio
- Immediate-mode UI framework
- Simple Game API (`Game` trait, `run_game()`)
- SRP refactoring (game.rs: 862->553 lines, -36%)

**Technical Debt Resolved:**
- Managers extracted: GameLoopManager, UIManager, RenderManager, WindowManager, SceneManager
- Files extracted: game_config.rs, contexts.rs, ui_integration.rs, scene_manager.rs, behavior_runner.rs
- Test quality: 0 TODOs, 155+ assertions

### Editor Foundation - COMPLETE (January 2026)
**Goal:** Build editor UI framework and basic tools.
- Dockable panel system (64 tests)
- Scene viewport with camera pan/zoom (45 tests)
- Read-only component inspector (15 tests)
- Hierarchy panel with tree view (13 tests)
- Entity picking and selection (6 tests)
- Transform gizmos (10 tests)
- Menu bar, toolbar, editor input (148 tests total)

</details>
