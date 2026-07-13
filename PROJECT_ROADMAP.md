# Insiculous 2D — Project Roadmap

## Vision: The 20 Games Challenge

**Make games.** The engine is production-ready for 2D games. The goal from here is to build 20 games — starting with arcade classics to learn the engine's capabilities, then creating original titles.

The 20 games challenge is a structured progression: each game teaches new patterns, exposes engine gaps, and builds confidence. By game 20, we'll have shipped original work.

**Engine Status (July 2026):** Core systems complete. 995 tests passing (100%), 0 ignored — every doc example compiles and runs (window/GPU-bound ones are `no_run`). Full DRY/SRP/KISS audit + remediation passes completed for all crates, plus a Game Programming Patterns audit (`PATTERNS_AUDIT.md`) — see `TECH_DEBT.md` for the live workspace rollup and `log_archive.md` for resolved history.

---

## Current Engine Capabilities

| System | Status | Notes |
|--------|--------|-------|
| ECS | ✅ Complete | HashMap-based per-type storage, type-safe queries, hierarchy |
| Physics | ✅ Complete | Rapier2d, platformer + top-down presets, collision callbacks |
| Rendering | ✅ Complete | WGPU 28, instanced sprites, batching |
| Sprite Animation | ✅ Complete | `SpriteAnimation` component + `SpriteAnimationSystem` |
| Audio | ✅ Complete | Rodio backend, SFX/music/master buses (spatial audio components are editor-only data — no runtime system yet) |
| Input | ✅ Complete | Keyboard/mouse, generic `InputMapping<A>` action layer (gamepad state model ready, no gilrs backend yet) |
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

Each game lives in `../games/<name>/` as a standalone cargo project consuming the engine via path deps.

---

### Game 1: Pong ☑ COMPLETE (June 2026) — details in `log_archive.md`
### Game 2: Breakout ☑ COMPLETE (June 2026) — details in `log_archive.md`

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

**Blocks:** Game 8 (Galaga — multiple projectiles), 11 (Run & Gun), 16 (Bullet Hell)
**Nice-to-have for:** Game 3 (Space Invaders), 5 (Asteroids) — buildable without it via manual despawn

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

Each game gets its own `../games/<name>/` project with a `README.md` describing controls, mechanics, and what was learned.

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

Workspace rollup with per-crate counts: root `TECH_DEBT.md`. LOW priority items
are tracked in `crates/*/TECH_DEBT.md` and are not listed here. Resolved items
live in `log_archive.md` (live docs carry open work only).

### Game Programming Patterns Audit (July 2026)

Full-codebase audit against the [Game Programming Patterns](https://gameprogrammingpatterns.com/contents.html)
catalog: **`PATTERNS_AUDIT.md`** (repo root) — 17 numbered findings + 12 Low items, each with
evidence and a pattern-based fix plan, mirrored as pointer entries in the per-crate
`TECH_DEBT.md` files and a new `../games/TECH_DEBT.md`. The **High** items:

- [x] **GPP-01 (State):** ~~`BehaviorState` bool soup~~ Resolved Jul 13 2026 (`StateMachine<BehaviorPhase>`; see `log_archive.md`)
- [ ] **GPP-02 (Data Locality):** HashMap-of-boxes component storage recorded as decision-of-record with a revisit trigger

Notable Medium: **GPP-03 (Flyweight/DRY)** — pong↔breakout duplication; promote only the
game-agnostic subset (ChaosTheme structure, grid-emit helper, visibility helper, small utils)
now, defer the genre-flavored subset (spawners, flow skeleton) until game 3 confirms rule-of-three
(Jesse's call, July 2026).

### Medium Priority

**engine_core (1 item):**
- [ ] **ARCH-006: Behaviors hardcoded in scene serialization** — `scene_data.rs`/`scene_loader.rs`/`scene_serializer.rs` match on Behavior variants instead of going through `ComponentRegistry`. Route through a registry/`Custom` variant; pairs with the Phase 4 scripting migration of `ecs/src/behavior.rs`.

**ui (1 item):**
- [ ] **JUN-T1: Text input is numeric-only and layout-blind** — blocks editor rename/search widgets; needs winit character events plumbed through the `input` crate.

**input (1 item):**
- [ ] **GAP-001: No gamepad backend** — state model complete and tested, but no gilrs integration produces events. Add a poll in the engine event loop; dead-zone normalization lands with it.

**common (2 items):**
- [ ] **ARCH-001: `CameraUniform` duplicated in renderer crate** — Use `common::CameraUniform` everywhere, remove renderer copy.
- [ ] **DRY-002: Volume clamping duplicated cross-crate** (`audio`, `ecs`) — add `clamp_volume()` utility in common.

**ecs_macros (1 item):**
- [ ] **KISS-001: Over-specified `syn` features** — `["full", "parsing"]` where `["derive"]` suffices; compile-time win.

---

## Development Guidelines

### For Every Game
1. Each game is a standalone cargo project in `../games/<name>/` (sibling to this repo)
2. Depends on `engine_core` (includes physics by default) + `ecs` if needed directly — no editor dep
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
# Run all engine tests
cargo test --workspace

# Run engine example
cargo run --example hello_world

# Run editor on a game project (games/ is a sibling directory)
cargo run --bin editor --features editor -- ../games/pong

# Run a game directly
cd ../games/pong && cargo run
```

**Key Files:**
- `AGENTS.md` — AI agent guidance (high-level)
- `training.md` — API patterns and examples
- `PROJECT_ROADMAP.md` — This file
- `../games/` — Sibling directory with all game projects
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

Completed milestones (Engine Core 2025, Editor Phase 1, Phase 2A standalone infrastructure) live in `log_archive.md`.
