# Insiculous 2D — Project Roadmap

## Vision: The 20 Games Challenge

**Make games.** The engine is production-ready for 2D games. The goal from here is to build 20 games — starting with arcade classics to learn the engine's capabilities, then creating original titles.

The 20 games challenge is a structured progression: each game teaches new patterns, exposes engine gaps, and builds confidence. By game 20, we'll have shipped original work.

**Engine Status (April 2026):** Core systems complete. 724 tests passing (100%).

---

## Current Engine Capabilities

| System | Status | Notes |
|--------|--------|-------|
| ECS | ✅ Complete | HashMap-based per-type storage, type-safe queries, hierarchy |
| Physics | ✅ Complete | Rapier2d, platformer + top-down presets, collision callbacks |
| Rendering | ✅ Complete | WGPU 28, instanced sprites, batching |
| Sprite Animation | ✅ Complete | `SpriteAnimation` component + `SpriteAnimationSystem` |
| Audio | ✅ Complete | Rodio backend, spatial audio |
| Input | ✅ Complete | Keyboard/mouse/gamepad, action mapping |
| UI | ✅ Complete | Immediate-mode, buttons, sliders, panels |
| Scene Serialization | ✅ Complete | RON format, prefabs, scene graph hierarchy |
| Behaviors | ✅ Complete | `PlayerPlatformer`, `PlayerTopDown`, `Patrol`, `FollowEntity`, `FollowTagged`, `Collectible` |
| Scene Editor | ✅ Complete | Entity CRUD, inspector, gizmos, play/pause/stop, undo/redo, save/load |
| Standalone Editor | ✅ Complete | `cargo run --bin editor -- /path/to/project` |
| Camera Follow | ❌ Missing | Needed by ~75% of games |
| Lifetime/Auto-Despawn | ❌ Missing | Bullets, effects, explosions — needed by ~80% of games |
| Tilemap | ❌ Missing | Grid-based levels — needed for Pac-Man, Zelda, roguelikes |

---

## Phase A: Games 1–5 — Start Now

**Engine work required:** None. All five games are buildable today.

Each game lives in `games/<name>/` as a standalone cargo project consuming the engine.

---

### Game 1: Pong ☐

Two paddles, one ball, score display. The "Hello, World" of games.

**Teaches:** Physics bounce, score tracking, UI overlay, simple AI opponent
**Key components:** `RigidBody` (kinematic paddles, dynamic ball), `Collider`, `Sprite`, immediate-mode score UI
**Controls:** Player 1 W/S, Player 2 Up/Down. AI mode: right paddle tracks ball with lag.
**Win condition:** First to 7 points.
**Estimated scope:** ~200 lines game logic.

---

### Game 2: Breakout ☐

Ball bouncing off a paddle, destroying a grid of bricks. Classic single-player.

**Teaches:** Dynamic entity spawning/despawning via collision callbacks, brick grid layout, lives system
**Key components:** Ball (dynamic), paddle (kinematic), brick entities (static, destroyed on hit), `Collectible` behavior for power-ups
**Controls:** Mouse or arrow keys to move paddle.
**Win condition:** Clear all bricks. Lives lost when ball falls off screen.
**Estimated scope:** ~300 lines.

---

### Game 3: Space Invaders ☐

Grid of enemies marching left/right and descending, player fires upward.

**Teaches:** Formation movement, projectile firing from game logic (spawn bullet entity each frame), enemy death, wave management
**Key components:** Enemy formation (ECS entities, shared direction state), player bullets (spawned/despawned manually), barrier entities
**Controls:** Arrow keys to move, Space to fire.
**Win condition:** Eliminate all invaders. Lose if any reach the bottom.
**Estimated scope:** ~400 lines.

---

### Game 4: Snake ☐

Growing snake controlled in four directions, eat food to grow, avoid self.

**Teaches:** Grid-based movement (no physics needed), segment-following logic, game-over detection, procedural food spawning
**Key components:** Head entity + tail segment entities, timer-driven movement, grid logic in game code
**Controls:** Arrow keys or WASD.
**Win condition:** Survive as long as possible, maximize length.
**Estimated scope:** ~300 lines.

---

### Game 5: Asteroids ☐

Ship rotates and thrusts in 2D space, asteroids split on hit, screen wraps.

**Teaches:** Rotation-based movement, screen wrap logic, asteroid splitting (spawn new smaller entities), invincibility frames
**Key components:** `RigidBody` (dynamic ship with angular velocity), asteroid entities (dynamic), bullet entities (kinematic), transform-based rotation
**Controls:** Left/Right to rotate, Up to thrust, Space to fire.
**Win condition:** Survive waves, maximize score.
**Estimated scope:** ~400 lines.

---

## Phase B: Engine Gap Work

**Three additions unblock Games 6–15.** Each is a small, focused addition with tests. No architectural changes.

---

### Gap 1: `CameraFollow` Behavior

**Blocks:** Game 10 (Platformer), 11 (Run & Gun), 12 (Zelda), 15 (Metroidvania), 16 (Bullet Hell), 17 (Roguelike), 19 (RTS)

**What:** A built-in behavior that smoothly moves the camera toward a target entity each frame.

**Location:** `crates/ecs/src/behavior.rs` (new variant) + `crates/engine_core/src/behavior_runner.rs` (new handler)

```rust
Behavior::CameraFollow {
    target_tag: String,      // Tag of entity to follow
    lerp_speed: f32,         // 0.0–1.0, how fast camera catches up
    offset: Vec2,            // Fixed offset from target (e.g. (0, 50) to show above)
    dead_zone: Option<Vec2>, // Optional: don't move camera inside this box
}
```

**Acceptance criteria:** Camera entity with this behavior smoothly follows tagged target. `lerp_speed = 1.0` snaps instantly, `0.05` gives smooth lag. Test: camera converges within 10 frames at lerp 0.5.

**Estimated scope:** ~80 lines + 3 tests.

---

### Gap 2: `Lifetime` Component + `LifetimeSystem`

**Blocks:** Game 3 (bullets), 5 (bullets), 8 (Galaga — multiple projectiles), 11 (Run & Gun), 16 (Bullet Hell)

**What:** A component that auto-despawns an entity after a given duration. Replaces manual timer tracking in game code for bullets, effects, and debris.

**Location:** `crates/ecs/src/sprite_components.rs` (component) + a new `LifetimeSystem` in `crates/ecs/src/`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifetime {
    pub remaining: f32,  // Seconds until despawn
}
```

`LifetimeSystem::update(&mut world, delta)` decrements all `Lifetime` components and calls `world.destroy_entity()` when `remaining <= 0`.

**Acceptance criteria:** Entity with `Lifetime { remaining: 0.5 }` is despawned exactly once, 0.5s after spawn. Test: entity exists at t=0.4, is gone at t=0.6.

**Estimated scope:** ~60 lines + 4 tests.

---

### Gap 3: `Tilemap` Component + Rendering

**Blocks:** Game 6 (Frogger), 7 (Tetris overlay), 9 (Pac-Man), 12 (Zelda), 13 (Tower Defense), 14 (Sokoban), 17 (Roguelike)

**What:** A component that holds a grid of tile indices and renders them efficiently using the sprite batch pipeline.

**Location:** `crates/ecs/src/sprite_components.rs` (component) + new rendering pass

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tilemap {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,           // World units per tile
    pub tileset: u32,             // Texture handle for the tileset
    pub tiles: Vec<u32>,          // tile_index per cell, 0 = empty
    pub tile_uv_size: Vec2,       // Fraction of tileset per tile (e.g. 1/8 for 8x8 tileset)
}
```

Rendering: Each non-zero tile becomes a sprite instance in the existing batch pipeline. No new GPU pipeline needed.

**Acceptance criteria:** Tilemap with a 16x16 grid of tiles renders correctly. Test: `Tilemap::sprite_instances()` returns correct count and UV regions for known input.

**Estimated scope:** ~150 lines + 5 tests.

---

## Phase C: Games 6–15 — Classic + Action

These games require one or more of the engine gap additions above.

| # | Game | Requires | Key New Patterns |
|---|------|----------|-----------------|
| 6 | Frogger | Tilemap | Lane scrolling, timed obstacles, death-and-reset |
| 7 | Tetris | Tilemap | Grid logic, piece rotation, line clearing |
| 8 | Galaga | Lifetime, SpriteAnimation | Enemy formation paths, multi-bullet patterns |
| 9 | Pac-Man | Tilemap, SpriteAnimation | Pathfinding AI (BFS), power-up state, ghost modes |
| 10 | Simple Platformer | CameraFollow, SpriteAnimation | Multi-level progression, camera smoothing |
| 11 | Run & Gun | CameraFollow, Tilemap, Lifetime | Horizontal scroll, level checkpoints |
| 12 | Zelda-style Top-Down | CameraFollow, Tilemap, SpriteAnimation | Room transitions, NPC dialog, item system |
| 13 | Tower Defense | Tilemap, SpriteAnimation | Wave spawning, tower placement, pathfinding |
| 14 | Sokoban / Puzzle | Tilemap | Move history / undo, level editor-compatible format |
| 15 | Metroidvania | CameraFollow, Tilemap, SpriteAnimation | Ability gating, persistent world state, map system |

Each game gets its own `games/<name>/` project with a `README.md` describing controls, mechanics, and what was learned.

---

## Phase D: Games 16–20 — Complex + Original

| # | Game | Focus |
|---|------|-------|
| 16 | Bullet Hell Shoot-em-up | High entity counts, Lifetime at scale, pattern scripting |
| 17 | Roguelike Dungeon | Procedural generation, fog of war, persistent save state |
| 18 | Fighting Game (simple) | Frame-precise animation, hitboxes, combo system |
| 19 | Strategy / Mini-RTS | Unit selection, pathfinding at scale, fog of war |
| 20 | Original Concept | Player's choice — built with full engine + editor workflow |

Game 20 is the capstone: an original game concept designed, built, and shipped using everything learned in games 1–19.

---

## Supporting Infrastructure

These items improve quality of life for game development. They are done when they unblock a specific game, not on a fixed schedule.

### Editor Polish (Backlog)
- [ ] Toolbar redesign (cleaner play controls, tool selection)
- [ ] Scene tree enhancements (icons, search, drag-and-drop reparenting)
- [ ] Inspector polish (collapsible sections, color picker, enum dropdowns)
- [ ] Asset browser (grid view, drag-to-assign texture onto Sprite)
- [ ] Copy/Paste entities (Ctrl+C / Ctrl+V)
- [ ] Multi-entity editing (shared properties, multi-gizmo)
- [ ] Prefab system (save entity as reusable template)
- [ ] Console panel (log output, filter, search)
- [ ] Tilemap editor tab (tile palette, brush, fill tools)
- [ ] Physics debugger overlay (collider wireframes, velocity vectors)

**Reference:** `crates/editor/IdealEditor.png` for target mockup

### Scripting (Phase 4 — after Game 10)
Hot-reloadable Rust script components via `dylib` + a `Script` trait. Unblocks faster game iteration for Games 11+. See old Phase 4 section for full spec — preserved in git history.

### Platform (Phase 6 — after Game 20)
WASM/Web export, mobile, desktop packaging. Not a prerequisite for making games; done when ready to ship.

---

## Technical Debt (High + Medium Only)

LOW priority items are tracked in `crates/*/TECH_DEBT.md` and are not listed here.

### Medium Priority

**renderer (2 items):**
- [ ] **SRP-001: SpritePipeline holds too many GPU resources** — `renderer/src/sprite.rs:225-254`. Split into PipelineResources, BufferManager, CameraManager, TextureBindGroupManager.
- [ ] **ARCH-003: Dead code with `#[allow(dead_code)]` suppressions** — `sprite.rs`, `sprite_data.rs`, `texture.rs`. Use or remove.

**ui (1 item):**
- [ ] **SRP-001: FontManager too many responsibilities** — `ui/src/font.rs:100-315`. Split into FontLoader, GlyphCache, TextLayoutEngine.

**ecs (2 items):**
- [ ] **ARCH-004: Hard-coded behaviors should move to scripting crate** — `ecs/src/behavior.rs`. Migrate to `scripting/src/builtins/` when scripting crate is created (Phase 4).
- [ ] **SRP-003: TransformHierarchySystem does double iteration** — `ecs/src/hierarchy_system.rs:87-118`. Reorganize to single pass.

**common (1 item):**
- [ ] **ARCH-001: `CameraUniform` duplicated in renderer crate** — Use `common::CameraUniform` everywhere, remove renderer copy.

**audio (1 item):**
- [ ] **ARCH-001: No streaming for large music assets** — All audio eagerly loaded. Add streaming path for long tracks, keep cache for SFX.

**input (1 item):**
- [ ] **TEST-001: Missing input timing + dead zone tests** — Add gamepad dead zone normalization and frame-accurate event timing tests.

**physics (1 item):**
- [ ] **TEST-001: Missing friction/kinematic/sensor validation** — Add coverage for friction/restitution, kinematic bodies, and sensors.

---

## Development Guidelines

### For Every Game
1. Each game is a standalone cargo project in `games/<name>/`
2. Depends only on `engine_core` + `ecs` (+ physics/audio as needed) — no editor dep
3. Has a `README.md` with: controls, how to run, what patterns it demonstrates
4. `cargo run` from the game directory launches it

### AI-Friendly Development
1. **CLI-testable** — All logic testable without GPU/window. `cargo test --workspace` validates everything.
2. **No manual testing** — If a feature can't be verified by `cargo test`, it needs a test.
3. **Small, focused files** — Files over 600 lines should be split.
4. **Explicit over implicit** — No magic numbers, hidden side effects, or clever tricks.
5. **Strong typing** — Enums over strings, newtypes over primitives.
6. **Verify before claiming** — Always run `cargo test --workspace` before claiming work is done.

### Editor Architecture
1. **Feature-gated** — Editor code compiles out without `--features editor`
2. **Design system** — All colors/spacing from `EditorTheme`, never hardcoded
3. **Command pattern** — All operations undoable
4. **Live editing** — Property changes visible immediately

---

## Quick Reference

```bash
# Run all tests
cargo test --workspace

# Run engine example
cargo run --example hello_world

# Run editor on a game project
cargo run --bin editor --features editor -- games/pong

# Run a game
cd games/pong && cargo run
```

**Key Files:**
- `AGENTS.md` — AI agent guidance (high-level)
- `training.md` — API patterns and examples
- `PROJECT_ROADMAP.md` — This file
- `games/` — All game projects (created as games are built)
- `src/bin/editor.rs` — Standalone editor binary
- `crates/editor/IdealEditor.png` — Target mockup for editor UI
- `examples/hello_world.rs` — Reference implementation
- `examples/editor_demo.rs` — Editor demo (requires `--features editor`)

---

## Design System Reference

Derived from `crates/editor/IdealEditor.png`.

### Color Palette
| Token | Hex | Usage |
|-------|-----|-------|
| `bg-primary` | `#1e1e1e` | Main panel backgrounds |
| `bg-viewport` | `#000000` | Viewport / canvas area |
| `bg-input` | `#2d2d2d` | Input fields, dropdowns |
| `accent-blue` | `#0078d4` | Selection highlights, active buttons |
| `accent-cyan` | `#00d9ff` | Panel headers, interactive highlights |
| `border-panel` | `#007acc` | Panel borders |
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

### Spacing
| Element | Value |
|---------|-------|
| Panel padding | 8px |
| Component section spacing | 12px |
| Input field height | 24px |
| Panel header height | 28px |
| Toolbar height | 36px |
| Status bar height | 22px |

### Layout
```
+-------------------------------------------------------------------+
| TOOLBAR (36px)                                                     |
+----------+----------------------------------------+---------------+
| SCENE    | 2D VIEWPORT                            | INSPECTOR     |
| TREE     |   (flexible center)                    |   (280px)     |
| (200px)  |                                        |               |
+----------+----+--------+--------+--------+--------+---------------+
| Bottom Panel Tabs: [Project] [Animation] [Tilemap] [Profiler]     |
+-------------------------------------------------------------------+
| STATUS BAR (22px): Ready | Objects: 42 | FPS: 60 | v2.0.1        |
+-------------------------------------------------------------------+
```

---

## Archive: Completed Work

<details>
<summary>Click to expand</summary>

### Engine Core (2025) — COMPLETE
- Memory safety, thread-safe input, panic-safe system registry
- Sprite rendering (WGPU 28), ECS with type-safe queries, asset management
- Rapier2d physics, scene serialization (RON + prefabs), scene graph hierarchy
- Audio (Rodio, spatial), immediate-mode UI, Simple Game API (`Game` trait, `run_game()`)
- SRP refactoring: GameLoopManager, UIManager, RenderManager, WindowManager, SceneManager extracted
- Test count: 724 passing, 29 ignored (GPU/window only), 0 failed

### Editor Phase 1 (January–February 2026) — COMPLETE
- Dockable panel system, scene viewport (pan/zoom, grid overlay, LOD)
- Entity picking (click, rectangle), transform gizmos (translate/rotate/scale)
- Component inspector with live writeback (Transform2D, Sprite, RigidBody, Collider, AudioSource)
- Generic serde-based read-only display for any component
- Component add/remove (categorized popup, cascade removal)
- Entity CRUD (create empty/sprite/physics, delete, duplicate Ctrl+D)
- Hierarchy panel (tree view, expand/collapse, Ctrl+click multi-select)
- Play/Pause/Stop with `WorldSnapshot` capture/restore (Ctrl+P, F5)
- Undo/redo command system (11 command types, merging for continuous edits)
- Scene save/load (Ctrl+S/Ctrl+O/Ctrl+N, RON format, dirty flag)
- Editor preferences persistence (camera, zoom, grid, last scene)
- `EditorTheme` system (30+ color tokens, converter methods)
- Status bar (entity count, FPS, status messages)
- Snap-to-grid (toggle, configurable grid size)

### Phase 2A: Standalone Infrastructure (March 2026) — COMPLETE
- Standalone editor binary (`cargo run --bin editor -- /path/to/project`)
- Standalone game project (`my_platformer`) consuming engine as external dep
- Editor font path fix, extended engine prelude

</details>
