# Insiculous 2D - Project Roadmap

## Target: IDEAL 2D GAME ENGINE EDITOR

**Reference mockup:** `crates/editor/IdealEditor.png`

This roadmap targets a purpose-built 2D editor — not a 3D engine with 2D bolted on. Every design decision prioritizes pixel-perfect 2D workflows, orthographic viewports, and sprite-native tooling. The editor should feel like a specialized 2D tool on par with Godot and Unity's 2D mode.

---

## Current Status (February 2026) - PRODUCTION READY

**Engine Status:** Core systems complete, production-ready for 2D games
**Editor Status:** Functional editor with entity CRUD, component management, play/pause/stop, and live property editing. UI polish and advanced tooling remain.

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
| Editor | 210+ | 100% pass |
| Editor Integration | 23 | 100% pass |
| ECS Macros | 3 | 100% pass |
| Common | 26 | 100% pass |

**Total:** 664+ tests passing (100% success rate)
**Code Quality:** 0 TODOs, 155+ assertions, 29 ignored (GPU/window only)

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

### Completed Editor Features
- Dockable panel system (Scene View, Hierarchy, Inspector, Asset Browser placeholder)
- Menu bar with dropdowns (File, Edit, View, Entity)
- Toolbar with tool selection (Select, Move, Rotate, Scale + Q/W/E/R shortcuts)
- Scene viewport with camera pan/zoom and grid overlay (LOD, subdivisions, axis indicators)
- Entity picking (click-to-select, rectangle selection, depth sorting)
- Transform gizmos (translate, rotate, scale with writeback)
- Editable component inspector with live writeback (Transform2D, Sprite, RigidBody, Collider, AudioSource)
- Generic serde-based read-only display for any component
- Component add/remove (categorized popup, [X] remove buttons, RigidBody->Collider cascade)
- Entity CRUD (create empty/sprite/physics bodies, delete with hierarchy cleanup, duplicate with Ctrl+D)
- Hierarchy panel with tree view, expand/collapse, name resolution, click/Ctrl+click selection
- Play/Pause/Stop with world snapshot/restore (Ctrl+P, Ctrl+Shift+P, F5)
- Visual play state indicator (tinted viewport border)
- Input routing (editor vs game based on play state)
- Read-only inspector during Playing state
- Snap-to-grid system (toggle, configurable grid size)
- Selection system (single, multi, toggle, primary entity)
- Undo/redo system (command pattern, 100-entry history, command merging for continuous edits)

**Verification:** `cargo run --example hello_world` (platformer demo), `cargo run --example editor_demo --features editor` (editor UI)

---

## Design System Reference

Derived from the target mockup. All editor UI should converge on these specifications.

### Color Palette
| Token | Hex | Usage |
|-------|-----|-------|
| `bg-primary` | `#1e1e1e` | Main panel backgrounds |
| `bg-viewport` | `#000000` | Viewport / canvas area |
| `bg-input` | `#2d2d2d` | Input fields, dropdowns |
| `accent-blue` | `#0078d4` | Selection highlights, active buttons, "+ Add Component" |
| `accent-cyan` | `#00d9ff` | Panel headers, interactive highlights, gizmo labels |
| `border-panel` | `#007acc` | Panel borders (bright blue) |
| `border-subtle` | `#333333` | Grid lines, separators |
| `text-primary` | `#ffffff` | Primary text |
| `text-secondary` | `#cccccc` | Secondary text, labels |
| `text-muted` | `#888888` | Disabled text, placeholders |
| `gizmo-x` | `#00ff00` | X-axis (green, horizontal) |
| `gizmo-y` | `#ff0000` | Y-axis (red, vertical) |
| `play-green` | `#00cc44` | Play button, playing border tint |
| `pause-yellow` | `#ffcc00` | Pause border tint |
| `stop-red` | `#cc3333` | Stop button |
| `error-red` | `#ff4444` | Error logs, validation |
| `warn-yellow` | `#ffcc00` | Warning logs |

### Typography
| Context | Font | Size |
|---------|------|------|
| Panel headers | System/Segoe UI | 14px bold |
| Component section headers | System/Segoe UI | 13px bold, accent-cyan |
| Labels | System/Segoe UI | 12px |
| Input values | Monospace | 12px |
| Status bar | System/Segoe UI | 11px |
| Title bar | System/Segoe UI | 18px bold |

### Spacing
| Element | Value |
|---------|-------|
| Panel padding | 8px |
| Component section spacing | 12px |
| Input field height | 24px |
| Panel header height | 28px |
| Toolbar height | 36px |
| Status bar height | 22px |
| Icon size (toolbar) | 16x16 |
| Icon size (hierarchy) | 14x14 |
| Tree indent | 20px per level |

### Layout Proportions (from mockup)
```
+-------------------------------------------------------------------+
| TOOLBAR (36px)                                                     |
| [Play][||][Stop]  [Select tools]  TITLE  [Grid][Snap][Zoom]  [Save][Export][Settings] |
+----------+----------------------------------------+---------------+
| SCENE    | 2D VIEWPORT [Orthographic]             | INSPECTOR     |
| TREE     |                                        |               |
| (200px)  |   (flexible center)                    | (280px)       |
|          |                                        | [Transform]   |
|          |   Canvas bounds (1920x1080)            | [Sprite]      |
|          |   Grid overlay                         | [Physics]     |
|          |   Gizmos on selected                   | [+Add Comp]   |
|          |                                        |               |
|          |                                        |               |
| [Search] |                                        |               |
+----------+----+--------+--------+--------+--------+---------------+
| Bottom Panel Tabs: [Project] [Animation] [Tilemap] [Sprite Editor] [Profiler] |
| (180px, collapsible)                                                |
+-------------------------------------------------------------------+
| STATUS BAR (22px): Ready | Objects: 42 | FPS: 60 | VRAM: 128MB | v2.0.1 |
+-------------------------------------------------------------------+
```

---

## Roadmap: Editor-First Development

**Philosophy:** A game engine is only as good as its editor. The priority is making the editor a functional, productive tool for building games — on par with what developers expect from Unity and Godot. The editor should feel like a specialized 2D tool, not a 3D engine with 2D mode bolted on. Scripting, animation, and platform support come after the editor is usable.

---

### Phase 1: Functional Editor

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
- [x] **Simplified editor_demo.rs** - 351 lines -> 66 lines, just entity setup + `run_game_with_editor()`
- [x] **Removed hard-coded Escape exit** - Escape key now flows to `Game::on_key_pressed()` like any other key
- [x] **Removed unused engine_core dep from editor crate** - Cleaner dependency graph

#### 1B. Property Editing with Writeback ✅ COMPLETE (February 2026)
Inspector changes modify ECS components with live preview.

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

#### 1C. Play / Pause / Stop ✅ COMPLETE (February 2026)
Run the game inside the editor, pause to inspect state, stop to reset to the saved scene.

- [x] **Editor play states** - `EditorPlayState` enum: `Editing`, `Playing`, `Paused`
- [x] **Scene snapshot/restore** - Clone-based `WorldSnapshot` captures all known component types
- [x] **Visual play state indicator** - Tinted viewport border (blue/green/yellow)
- [x] **Input routing** - Game receives input only during Playing state
- [x] **Read-only inspector during play** - Inspector shows component values but disables editing during Playing state

#### 1D. Viewport Entity Picking ✅ COMPLETE (February 2026)
Click entities in the scene view to select them. Selection syncs to hierarchy and inspector.

- [x] **Viewport click-to-select** - EntityPicker with screen_to_world() conversion
- [x] **Rectangle selection** - Drag in viewport to select multiple entities

#### 1E. Entity Operations ✅ COMPLETE (February 2026)
Create, delete, duplicate entities from the editor UI.

- [x] **Create entity** - Entity menu: Create Empty, Create Sprite, Create Camera, Create Static/Dynamic/Kinematic Body
- [x] **Delete entity** - Remove entity and all components, reparent children
- [x] **Duplicate entity** - Ctrl+D deep copy with hierarchy preservation

#### 1F. Component Add/Remove ✅ COMPLETE (February 2026)
Attach and detach components on selected entities.

- [x] **Add Component button** - Categorized popup (Core, Rendering, Physics, Audio)
  - Uses ComponentRegistry for available types
  - Component added with default values
- [x] **Remove Component button** - [X] buttons on component headers
  - Cascade removal (removing RigidBody also removes Collider)
  - Physics bodies cleaned up when physics components removed

#### 1G. Undo/Redo ✅ COMPLETE (February 2026)
Every editor operation is reversible via the command pattern.

- [x] **Command pattern** - `EditorCommand` trait with `execute()` and `undo()` methods
  - `CommandHistory` struct with undo/redo stacks (max 100 entries)
  - `StoredComponent` enum for type-safe component capture/restore
  - Commands: `SetTransformCommand`, `SetSpriteCommand`, `SetRigidBodyCommand`, `SetColliderCommand`, `SetAudioSourceCommand`, `CreateEntityCommand`, `DeleteEntityCommand`, `AddComponentCommand`, `RemoveComponentCommand`, `TransformGizmoCommand`, `MacroCommand`
- [x] **Command merging** - Continuous gizmo drags merge into single command
  - Inspector slider drags merge via `try_merge_or_push()`
  - Gizmo movements merge while actively dragging (initial/final transform capture)
  - Same-entity same-field edits collapse into one undo entry
- [x] **Undo/Redo UI** - Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y, Edit menu items
  - Display current command name in status bar
  - Undo/redo disabled during Playing state

#### 1H. Scene Save/Load 🎯 CURRENT PRIORITY
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

### Phase 2: Ideal Editor UI (Mockup-Driven) 🎯 NEXT

**Goal:** Transform the functional editor into the polished, 2D-native tool shown in the design mockup. Focus on getting the visual layout pixel-perfect first, then add interactivity. Every pixel should communicate "this is a 2D-first editor."

**Reference:** `crates/editor/IdealEditor.png`

#### 2A. Top Toolbar Redesign
Unified toolbar matching the mockup layout: play controls left, tool selection center-left, title center, status indicators center-right, action buttons right.

- [ ] **Toolbar layout overhaul** - Replace current toolbar with mockup layout
  - Left section: Play (triangle), Pause (||), Stop (square) buttons with colored backgrounds
  - Center-left: Tool icons — Select (cursor), Move (arrows), Rotate (arc), Scale (box) with visual feedback
  - Center: Editor title "IDEAL 2D GAME ENGINE EDITOR" (or project name)
  - Center subtitle: "Context-Aware - Modular - 2D-Optimized" tagline
  - Center-right: Status indicators — `Grid: ON`, `Snap: 8px`, `Zoom: 100%` (clickable to toggle/edit)
  - Right section: Save, Export, Settings buttons with icons
- [ ] **Grid/Snap/Zoom toolbar indicators** - Live-updating status chips
  - Grid toggle: click to toggle grid visibility (current state shown)
  - Snap value: click to cycle snap sizes (1px, 2px, 4px, 8px, 16px, 32px, Off)
  - Zoom level: shows current camera zoom percentage (mouse wheel to adjust)
- [ ] **Save/Export/Settings buttons** - Quick-access toolbar actions
  - Save: Ctrl+S (same as File > Save, with dirty indicator)
  - Export: Opens export dialog (build/package game)
  - Settings: Opens editor preferences panel

#### 2B. Scene Tree Enhancements
Upgrade the hierarchy panel to match the mockup's scene tree with search, icons, and polished interaction.

- [ ] **Scene tree header** - "SCENE TREE" header in accent-cyan, matching mockup style
- [ ] **Node type icons** - Distinct icons for entity types (14x14, left of name)
  - Camera icon for entities with Camera component
  - Sprite icon for entities with Sprite component
  - Physics icon for entities with RigidBody/Collider
  - Audio icon for entities with AudioSource
  - Script icon for entities with behaviors/scripts (e.g., "PlayerController.cs" style)
  - Folder/group icon for entities that are hierarchy parents only
  - Generic entity icon as fallback
- [ ] **Search bar** - Filter bar at bottom of scene tree panel
  - Text input: "Search nodes..." placeholder
  - Real-time filtering as user types
  - Highlight matches, collapse non-matching branches
  - Filter by name, component type
- [ ] **Selection highlighting** - Checkmark indicator on selected entities (like mockup's `Player [✓]`)
  - Primary selection in accent-cyan text
  - Multi-selection with checkmarks on each
- [ ] **Entity count badges** - Show child count for collapsed groups (e.g., "Enemies [5]")
- [ ] **Visibility toggle** - Eye icon per entity to hide/show in viewport (does not affect game logic)
- [ ] **Drag-and-drop reparenting** - Drag entities in hierarchy to change parent
  - Visual drop target indicator (above, below, as child)
  - Reparent updates transform hierarchy (recalculate local transform from global)
  - Prevent circular reparenting (can't parent to own descendant)
  - Drop at root level to unparent
- [ ] **Right-click context menu** - Context actions per entity
  - Create Child, Duplicate, Delete, Rename
  - Copy/Cut/Paste
  - Focus in Viewport

#### 2C. Inspector Polish
Upgrade the inspector to match the mockup's collapsible sections, color picker, and preview area.

- [ ] **Inspector header** - "INSPECTOR" in accent-cyan, matching mockup style
- [ ] **Collapsible component sections** - Triangle toggle (▼/▶) on section headers
  - Click header to collapse/expand component fields
  - Collapsed state persisted per component type
  - Section header shows component name in accent-cyan bold (e.g., "▼ Transform", "▼ Sprite Renderer", "▼ Physics 2D")
- [ ] **Color picker widget** - Visual color swatch + hex input
  - Small color swatch preview (16x16) next to hex value
  - Click swatch to open color picker popup
  - Hex input (e.g., "#FFFFFF") with live preview
  - Used for Sprite color, background color, etc.
- [ ] **Flip checkboxes** - Inline checkbox row for Sprite flip X/Y
  - `[ ]X  [✓]Y` layout matching mockup
- [ ] **Dropdown menus** - For enum fields (Body Type: Dynamic/Static/Kinematic)
  - Click to open dropdown list
  - Currently read-only for body type — wire up for editing
- [ ] **Preview thumbnail** - Mini viewport in top-right of inspector
  - Shows selected entity's sprite/appearance
  - Small (80x80) preview area
  - Tool mode icons overlaid (move, rotate, scale, etc.)
- [ ] **Numeric input fields** - Monospace display matching mockup style
  - `Position:  X  120.0  Y  85.0` layout with labels
  - `Rotation:  0.0°` with degree symbol
  - `Scale:     X  1.0   Y  1.0` aligned columns
  - Click-to-edit, drag-to-adjust, Tab to next field
- [ ] **"+ Add Component" button** - Full-width blue button at bottom of inspector
  - Uses `accent-blue` background (#0078d4)
  - Centered text: "+ Add Component"
  - Opens categorized popup (already implemented, just style update)

#### 2D. Viewport Enhancements
Polish the 2D viewport to match the mockup's orthographic view with canvas bounds and gizmo styling.

- [ ] **Viewport header** - "2D VIEWPORT [Orthographic]" label at top of viewport panel
- [ ] **Canvas bounds overlay** - Always-visible game resolution rectangle
  - Dotted/dashed line rectangle showing game resolution (e.g., 1920x1080)
  - Label: "UI Canvas (1920x1080)" in text-secondary color
  - Configurable resolution (match GameConfig window size)
  - Semi-transparent fill outside bounds (subtle darkening)
- [ ] **Gizmo visual polish** - Match mockup's gizmo style
  - X-axis: green arrow/line extending right with "X" label
  - Y-axis: red arrow/line extending up with "Y" label
  - Selection box: white/cyan dashed rectangle around selected entity
  - Axis labels at arrow tips
- [ ] **Dark viewport background** - Near-black (#000000) viewport canvas
  - Distinct from panel background (#1e1e1e)
  - Makes sprites and objects visually pop
- [ ] **Grid styling** - Subtle grid matching mockup
  - Primary grid lines in `border-subtle` (#333333)
  - Origin axes more prominent (red Y, green X)
  - Grid adapts to zoom level (already implemented via LOD)

#### 2E. Bottom Panel — Tabbed Workspace
Replace the single asset browser placeholder with a tabbed panel system supporting multiple workspace tools.

- [ ] **Tab bar** - Horizontal tabs at top of bottom panel
  - Tabs: Project, Animation, Tilemap, Sprite Editor, Profiler
  - Active tab highlighted with accent-blue underline
  - Tab icons (folder, film strip, grid, image, chart)
  - Click to switch, tabs persist across sessions
- [ ] **Project tab** - File/asset browser (replaces current asset browser placeholder)
  - Grid/list view toggle
  - Folder navigation with breadcrumb path
  - Thumbnail generation for textures
  - Search bar with type filtering
  - Drag-and-drop asset assignment to inspector/viewport
- [ ] **Animation tab** - Sprite animation timeline (stub initially)
  - Placeholder: "Animation editor — coming in Phase 4"
  - Will house dopesheet, keyframe editor, animation preview
- [ ] **Tilemap tab** - Tile painting workspace (stub initially)
  - Placeholder: "Tilemap editor — coming in Phase 5"
  - Will house brush tools, tile palette, layer management
- [ ] **Sprite Editor tab** - Sprite sheet slicing tool (stub initially)
  - Placeholder: "Sprite editor — coming in Phase 4"
  - Will house sprite sheet importer, frame definition, atlas preview
- [ ] **Profiler tab** - Performance monitoring (stub initially)
  - Placeholder: "Profiler — coming in Phase 5"
  - Will house frame time graph, system breakdown, draw call stats
- [ ] **Panel collapse** - Bottom panel can be collapsed to just the tab bar (click tab to toggle)
  - Double-click tab to collapse/expand
  - Drag top edge to resize

#### 2F. Status Bar
Persistent status bar at the very bottom of the editor window, below all panels.

- [ ] **Status bar layout** - Full-width bar (22px height) with three sections
  - Left: Status message (e.g., "Ready", "Saving...", "Entity created", undo command name)
  - Center: Runtime stats — `Objects: 42 | FPS: 60 | VRAM: 128MB`
  - Right: Version string (e.g., "v2.0.1 - Stable")
- [ ] **Object count** - Live entity count from ECS world
- [ ] **FPS counter** - Smoothed frames-per-second from game loop
- [ ] **VRAM estimate** - Approximate GPU memory usage from renderer
  - Texture memory + buffer memory
  - Updated every 60 frames (not every frame)
- [ ] **Status messages** - Contextual messages that auto-clear after 3 seconds
  - "Entity created" after Create Entity
  - "Scene saved" after Ctrl+S
  - "Undo: Move Entity" after Ctrl+Z
  - Error messages persist until dismissed

#### 2G. Design System Theme Implementation
Apply the design system colors, typography, and spacing to all existing editor UI.

- [ ] **Theme constants** - Centralized color/spacing definitions
  - `EditorTheme` struct with all design system tokens
  - Used by all panels, toolbar, status bar, menus
  - Replaces any hardcoded colors scattered across editor code
- [ ] **Panel border styling** - All panels bordered with `border-panel` (#007acc)
  - Consistent 1px border on all panels
  - Panel headers use `accent-cyan` text on `bg-primary` background
- [ ] **Input field styling** - Consistent input appearance
  - `bg-input` (#2d2d2d) background for all text/number inputs
  - `text-primary` for values, `text-secondary` for labels
  - 24px height, 4px border-radius
- [ ] **Button styling** - Consistent button appearance
  - Primary buttons: `accent-blue` background, white text
  - Secondary buttons: `bg-input` background, `text-secondary`
  - Hover: lighten by 10%, Active: darken by 10%
- [ ] **Font system** - Monospace for numeric values, system font for labels
  - Ensure monospace font loaded for inspector numeric fields
  - Coordinate displays always use fixed-width characters

**Phase 2 Milestone:** The editor visually matches the design mockup. All panels have proper headers, borders, and colors. The toolbar shows grid/snap/zoom status. The bottom panel has tabs (even if some are stubs). A status bar shows FPS and entity count. The scene tree has icons and search. The inspector has collapsible sections and a color picker.

**Success Metrics:**
- Screenshot of editor is visually comparable to `IdealEditor.png`
- All design system colors applied consistently (no hardcoded one-offs)
- Bottom panel tabs switchable (even if content is placeholder)
- Status bar shows live FPS and entity count
- Scene tree search filters entities in real-time
- Inspector sections collapse/expand smoothly

---

### Phase 3: Productive Editor

**Goal:** Quality-of-life features that make the editor efficient for daily use. Asset management, multi-editing, prefabs, and a console.

#### 3A. Asset Browser (Project Tab Content)
Full implementation of the Project tab content from Phase 2E.

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

#### 3B. Multi-Object Editing
- [ ] **Shared property editing** - Select multiple entities, edit common properties
  - Inspector shows fields shared across all selected entities
  - Mixed values shown as "--" or indeterminate state
  - Edits applied to all selected entities simultaneously
- [ ] **Multi-transform** - Gizmo operates on all selected entities
  - Move: all selected entities translate by same delta
  - Rotate: all selected entities rotate around selection center
  - Scale: all selected entities scale relative to selection center

#### 3C. Copy/Paste
- [ ] **Copy/Cut/Paste entities** - Ctrl+C, Ctrl+X, Ctrl+V
  - Clipboard holds serialized entity data
  - Paste at mouse position or viewport center
  - Paste preserves component values and hierarchy
  - Cut = Copy + Delete

#### 3D. Prefab System
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

#### 3E. Console Panel
- [ ] **Log output** - Display engine log messages in Console panel
  - Color-coded by level: Info (white), Warn (yellow), Error (red), Debug (gray)
  - Scrollable with auto-scroll to bottom
  - Clear button
  - Log level filter dropdown
- [ ] **Search and filter** - Filter log messages by text or source
  - Regex support for advanced filtering
  - Collapse repeated messages with count

#### 3F. Localization System (i18n)
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

**Phase 3 Milestone:** Import sprites through asset browser, create prefab templates, edit multiple entities at once, view logs in console, switch editor language to Pirate.

**Success Metrics:**
- Import 50+ assets in < 5 minutes with drag-and-drop
- Edit shared properties across 10 selected entities simultaneously
- Prefab changes propagate to all instances instantly
- Switch languages in < 100ms without restart

---

### Phase 4: Scripting & Animation

**Goal:** Runtime behaviors via scripts and 2D animation tooling. These depend on a functional editor for inspector integration and workflow. Animation tools populate the Animation and Sprite Editor bottom tabs.

#### 4A. Scripted Behaviors (Script Components)
Unity/Godot-style script components — attach behavior scripts to entities with hot-reload support.

- [ ] **Script trait system** - Rust-native script components
  - `Script` trait with lifecycle hooks (`on_start`, `on_update`, `on_physics`, `on_destroy`)
  - Clear separation: `on_update` for visual/frame logic, `on_physics` for physics/movement
  - Automatic component registration with ECS
  - Access to `ScriptContext` (entity, world queries, delta time, input)
- [ ] **Hot-reload support** - Iterate without recompiling the game
  - Scripts compiled as dynamic libraries (.so/.dll)
  - File watcher detects changes and reloads automatically
  - State preservation across reloads (optional serialization)
  - Graceful error handling — script errors don't crash the game
- [ ] **Inspector integration** - Automatic UI for script fields
  - `#[inspectable]` attribute for field customization
  - Auto-generated editors for: f32 (sliders), Vec2/Vec3, bool, enums, String
  - Live editing — changes reflect immediately in running game
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

#### 4B. Sprite Animation System (Animation + Sprite Editor Tabs)
Populate the Animation and Sprite Editor bottom tabs with functional tooling.

- [ ] **Sprite sheet importer** (Sprite Editor tab) - Import and slice sprite sheets
  - Automatic grid-based slicing
  - Manual frame definition with visual editor
  - Support for: PNG, Aseprite files (.ase/.aseprite)
  - Atlas preview with frame numbering
- [ ] **Animation timeline editor** (Animation tab) - Keyframe-based animation
  - Dopesheet view for frame timing
  - Curve editor for tweening/easing
  - Preview window with playback controls
  - Onion skinning (previous/next frame overlay)
- [ ] **Animation controller** - State machine for animations
  - Animation states (Idle, Run, Jump, Attack)
  - Transitions with conditions (parameters, triggers)
  - Blend trees for smooth transitions
- [ ] **Animation components** - ECS integration
  - `AnimationPlayer` — Play animations on entities
  - `Animator` — State machine controller
  - `SpriteSheet` — Reference to sprite sheet asset

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

**Phase 4 Milestone:** Create a script, attach to entity, edit fields in inspector, hot-reload on save. Animate a character with idle/run/jump controlled by physics. Animation and Sprite Editor tabs are functional.

**Success Metrics:**
- Script hot-reload in < 500ms on change
- Zero boilerplate for simple scripts (just impl Script)
- Script errors caught and logged, game continues running
- 100+ frame animations at 60 FPS
- Seamless animation transitions

---

### Phase 5: Advanced Editor Tools

**Goal:** Professional-grade development tools built on top of the functional editor. Populates remaining bottom panel tabs and adds specialized viewport overlays.

#### 5A. Physics Debugger
- [ ] **Collider wireframe rendering** - Overlay physics shapes on viewport
  - Box, circle, capsule outlines
  - Color-coded by body type (dynamic = blue, static = green, kinematic = yellow)
- [ ] **Velocity vector visualization** - Arrow showing body velocity
- [ ] **Collision point highlighting** - Flash on contact points
- [ ] **Toggle overlay** - Quick on/off for physics debug rendering (toolbar toggle)

#### 5B. Profiler Integration (Profiler Tab)
Populate the Profiler bottom tab with real-time performance data.

- [ ] **Frame time graph** - Real-time graph with 16.6ms target line
  - Rolling 120-frame history
  - Color-coded: green (< 16ms), yellow (16-33ms), red (> 33ms)
- [ ] **System timing breakdown** - ECS, Physics, Render, UI time per frame
  - Stacked bar chart per frame
  - Percentage breakdown
- [ ] **Draw call and batch count** - Rendering statistics
  - Sprite batches, texture switches, draw calls per frame
- [ ] **Memory usage tracking** - Per-system allocation tracking
  - ECS component memory, texture memory, audio buffer memory
  - Feeds into status bar VRAM display

#### 5C. Tilemap Editor (Tilemap Tab)
Populate the Tilemap bottom tab with tile painting tools.

- [ ] **Tile palette** - Visual tile selection from tileset texture
  - Grid display of available tiles
  - Click to select active tile
  - Multi-tile selection for stamp tool
- [ ] **Paintbrush and fill tools** - Paint tiles in viewport
  - Brush: paint single tiles
  - Rectangle: fill rectangular regions
  - Bucket: flood fill connected tiles
  - Eraser: remove tiles
- [ ] **Multiple layers** - Background, foreground, collision layers
  - Layer visibility toggles
  - Active layer selection
  - Layer ordering/reordering
- [ ] **Autotiling support** - Automatic tile neighbor matching
  - Bitmask-based autotile rules
  - Preview in tile palette
- [ ] **Tile property editing** - Per-tile collision, metadata
  - Collision shape per tile
  - Custom properties (walkable, damage, etc.)

#### 5D. Particle System Editor
- [ ] **Emitter configuration** - Rate, shape, burst settings
- [ ] **Particle properties** - Lifetime, size, color, velocity curves
- [ ] **Curve editors** - Visual curve editing for property animation
- [ ] **Real-time preview** - Live particle preview in viewport

#### 5E. Visual Scripting
- [ ] **Node graph editor** - Visual programming interface
  - Event nodes (OnStart, OnUpdate, OnCollision)
  - Action nodes (PlaySound, SetPosition, SpawnEntity)
  - Flow control (Branch, Sequence, Loop)
  - Variable system (local, global, blackboard)

#### 5F. Asset Pipeline
- [ ] **Sprite atlas generator** - Automatic texture packing
  - Bin packing algorithms (MaxRects, Shelf)
  - Automatic sprite referencing updates
- [ ] **Asset import pipeline** - Automated import with caching
  - Watch folders for auto-import
  - Import presets for common asset types
  - Background import with progress tracking
- [ ] **Audio asset manager** - Waveform preview and editing

**Phase 5 Milestone:** Debug physics collisions visually, profile frame time in the Profiler tab, create tilemaps in the Tilemap tab, build particle effects.

**Success Metrics:**
- < 5ms overhead for physics debug rendering
- Particle systems with 1000+ particles at 60 FPS
- Automatic atlas packing with < 10% wasted space
- Tilemap painting at 60 FPS with 1000+ tiles

---

### Phase 6: Platform & Deployment

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

**Phase 6 Milestone:** Deploy `hello_world.rs` to Web, mobile, and desktop.

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
  - Resolution: Dual-storage system removed. `World::new_optimized()` and `ComponentStorage` enum deleted. Single storage path via `ComponentRegistry` -> `ComponentStore` (HashMap-based). Documentation updated to reflect actual storage.

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
  - See: Phase 4A Scripted Behaviors
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
7. **Design system** - All colors/spacing from `EditorTheme`, never hardcoded
8. **2D-native** - Orthographic viewport, pixel grid, sprite-first tooling

### Scripted Behaviors
1. **Scripts are just Rust** - No DSL, no magic — pure Rust structs implementing a trait
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
- `crates/editor/IdealEditor.png` - Target mockup for editor UI
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
