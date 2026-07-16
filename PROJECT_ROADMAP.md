# Insiculous 2D — Project Roadmap

## Vision: The 20 Games Challenge

**Make games.** The engine is production-ready for 2D games. The goal from here is to build 20 games — starting with arcade classics to learn the engine's capabilities, then creating original titles.

The 20 games challenge is a structured progression: each game teaches new patterns, exposes engine gaps, and builds confidence. By game 20, we'll have shipped original work.

**Engine Status (July 2026):** Core systems complete. 1109 tests passing (100%), 0 ignored — every doc example compiles and runs (window/GPU-bound ones are `no_run`). Full DRY/SRP/KISS audit + remediation passes completed for all crates, plus a Game Programming Patterns audit (closed Jul 2026; history in `log_archive.md`) — see `TECH_DEBT.md` for the live workspace rollup and `log_archive.md` for resolved history.

---

## Current Engine Capabilities

| System | Status | Notes |
|--------|--------|-------|
| ECS | ✅ Complete | HashMap-based per-type storage, type-safe queries, hierarchy |
| Physics | ✅ Complete | Rapier2d, platformer + top-down presets, collision events (bus + `take_collision_events()`) |
| Rendering | ✅ Complete | WGPU 28, instanced sprites, batching |
| Sprite Animation | ✅ Complete | `SpriteAnimation` component + `SpriteAnimationSystem` |
| Audio | ✅ Complete | Rodio backend, SFX/music/master buses (spatial audio components are editor-only data — no runtime system yet) |
| Input | ✅ Complete | Keyboard/mouse/gamepads (gilrs backend), `InputMapping<A>`, player-aware `InputSettings` (`ctx.players`: P1/P2 routing, analog axes, JSON-persisted bindings, gamepad menus) |
| Local 2-Player | ✅ Complete | All 5 games (Jul 2026): Pong 2P, Breakout co-op (top/bottom paddles + `*_2p` levels), Invaders/Asteroids co-op, Snake versus |
| Pause + Menu Chrome | ✅ Complete | Engine `PauseMenu` (world freeze, Resume/Restart/Quit, `ctx.time_scale`) + `MenuPanel` window chrome — all 5 games (Jul 2026) |
| UI | ✅ Complete | Immediate-mode, buttons, sliders, panels |
| Scene Serialization | ✅ Complete | RON format, prefabs, scene graph hierarchy |
| Behaviors | ✅ Complete | `PlayerPlatformer`, `PlayerTopDown`, `Patrol`, `FollowEntity`, `FollowTagged`, `Collectible`, `CameraFollow` |
| Scene Editor | ✅ Complete | Entity CRUD, inspector, gizmos, play/pause/stop, undo/redo, save/load |
| Standalone Editor | ✅ Complete | `cargo run --bin editor -- /path/to/project` |
| Camera Follow | ✅ Complete | `Behavior::CameraFollow` + main-camera-entity → render-camera sync |
| Lifetime/Auto-Despawn | ✅ Complete | `Lifetime` component + `LifetimeSystem` (ecs, in prelude) |
| Tilemap | ✅ Complete | `Tilemap` component, batched through the sprite pipeline |

---

## Phase A: Games 1–5 — ☑ COMPLETE (July 2026)

**Engine work required:** None (confirmed — all five shipped without engine changes beyond the planned promotions).

Each game lives in `../games/<name>/` as a standalone cargo project consuming the engine via path deps.

---

### Game 1: Pong ☑ COMPLETE (June 2026) — details in `log_archive.md`
### Game 2: Breakout ☑ COMPLETE (June 2026) — details in `log_archive.md`

---

### Game 3: Space Invaders ☑ COMPLETE (July 2026) — details in `log_archive.md`

---

### Game 4: Snake ☑ COMPLETE (July 2026) — details in `log_archive.md`

---

### Game 5: Asteroids ☑ COMPLETE (July 2026) — details in `log_archive.md`

Phase A is done — all five arcade games shipped.

---

## Phase B: Engine Gap Work — ☑ COMPLETE (July 2026)

**Three additions unblock Games 6–15.** All shipped.

### Gap 1: `CameraFollow` Behavior ☑ COMPLETE (July 2026) — details in `log_archive.md`

### Gap 2: `Lifetime` Component + `LifetimeSystem` ☑ COMPLETE (July 2026) — details in `log_archive.md`

### Gap 3: `Tilemap` Component + Rendering ☑ COMPLETE (July 2026) — details in `log_archive.md`

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

### Game Programming Patterns Audit (July 2026) — CLOSED
Full-codebase audit against the [Game Programming Patterns](https://gameprogrammingpatterns.com/contents.html)
catalog, run and largely resolved Jul 13 2026 (15 of 17 numbered findings fixed same-day —
summary + per-crate resolutions in `log_archive.md`). Remaining open GPP items live in the
per-crate `TECH_DEBT.md` files and `../games/TECH_DEBT.md` (GPP-03 closed with game 3's
rule-of-three; GPP-06/16 are parked with Phase 4 scripting; GPP-02 is a decision of record).

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
