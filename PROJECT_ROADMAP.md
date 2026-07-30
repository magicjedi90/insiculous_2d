# Insiculous 2D — Project Roadmap

## Vision: Deion the Insiculous

**The world of Deion is the project's identity.** Deion the Insiculous — a
SNES-styled hero, a ball of DEIONized water with an icicle mohawk — lives in a
food-coded world. Every game we ship is a window into that world: unique
"Deion Style" pixel-art characters and assets, not stand-in shapes. The
geometry-wars neon look that carried games 1–6 is demoted to an FX/accent
layer; SNES-era sprite art becomes the primary style.

**The 20 Games Challenge is the vehicle**, not the destination: arcade classics
teach the engine and expose gaps, building toward original Deion titles. The
challenge is **paused at game 7 (Tetris)** while the Deion Pivot (Phases E–I
below) lands; it resumes with the new asset style from day one.

**Engine Status (July 2026):** Core systems complete. 1253 tests passing
(100%), 0 ignored — every doc example compiles and runs (window/GPU-bound ones
are `no_run`). Full DRY/SRP/KISS audit + Game Programming Patterns audit closed
(history in `log_archive.md`); see `TECH_DEBT.md` for the live rollup.

**The Deion Pivot (adversarially reviewed Jul 28 2026):** plan drafted, reviewed
by kimi (15 findings adjudicated — 13 accepted, 2 rebutted), settled. Phases E–I
below are the reworked roadmap.

---

## Current Engine Capabilities

| System | Status | Notes |
|--------|--------|-------|
| ECS | ✅ Complete | HashMap-based per-type storage, type-safe queries, hierarchy |
| Physics | ✅ Complete | Rapier2d, platformer + top-down presets, collision events (bus + `take_collision_events()`) |
| Rendering | ✅ Complete | WGPU 28, instanced sprites, batching |
| Sprite Animation | 🔧 Rework (Phase E) | `SpriteAnimation` ticks frames but nothing writes `current_frame_region()` into `Sprite.tex_region` — disconnected from rendering. Phase E replaces it with named clips over a sheet grid |
| Pixel-Art Pipeline | 🔧 In progress (Phase E) | E1 ☑ `TextureFilter` knob (config default + per-call override), E2 ☑ `common::SheetGrid` shared UV math, E6 ☑ dead `atlas.rs` deleted (all Jul 30 2026). Remaining: named-clip animation (E3), `.sheet.ron` loader (E4), `#solid`/`#rgba` scene round-trip (E5) |
| Audio | ✅ Complete | Rodio backend, SFX/music/master buses (spatial audio components are editor-only data — no runtime system) |
| Input | ✅ Complete | Keyboard/mouse/gamepads (gilrs backend), `InputMapping<A>`, player-aware `InputSettings` (`ctx.players`, JSON-persisted bindings) |
| Local 2-Player | ✅ Complete | All games 2-player (Jul 2026) |
| Pause + Menu Chrome | ✅ Complete | Engine `PauseMenu` + `MenuPanel` window chrome — all games |
| UI | ✅ Complete | Immediate-mode, text editing, data-driven UiLabel/UiPanel/UiButton components |
| Localization | ✅ Complete | `ctx.strings`, RON locale files, per-locale fonts (Pong + Frogger localized) |
| Scene Serialization | ✅ Complete | RON format, prefabs, hierarchy (Phase E adds `tex_region`/`visible` + `#solid` round-trip fixes) |
| Behaviors | ✅ Complete | `PlayerPlatformer`, `PlayerTopDown`, `Patrol`, `FollowEntity`, `FollowTagged`, `Collectible`, `CameraFollow` |
| Scene Editor | ✅ Complete | Entity CRUD, inspector, gizmos, play/pause/stop, undo/redo, save/load, asset browser + drag-to-assign |
| Standalone Editor | ✅ Complete | `cargo run --bin editor -- /path/to/project` |
| Tilemap | ✅ Complete | `Tilemap` component, batched through the sprite pipeline (Frogger is first consumer) |
| Web/WASM Export | 🔧 In progress (Phase H) | H1 spike ☑ PASSED Jul 30 2026 (`coordination/H1_SPIKE.md`): 14/14 deps compile for wasm32, **audio decision: stay on rodio** (works in live browser), WebGPU demo screenshot-verified, gilrs has a real web backend (no no-op gating needed). Remaining (H2–H6, known refactor): `pollster::block_on` init → async, `thread::sleep` throttle, `Instant`/`SystemTime` → `web-time`, `std::fs` sites → bytes-primary API + localStorage persistence |

---

## Phase A: Games 1–5 — ☑ COMPLETE (July 2026)

Pong, Breakout, Space Invaders, Snake, Asteroids — details in `log_archive.md`.
Each lives in `../games/<name>/` as a standalone cargo project.

## Phase B: Engine Gap Work — ☑ COMPLETE (July 2026)

`CameraFollow`, `Lifetime`, `Tilemap` — details in `log_archive.md`.

## Game 6: Frogger — ☑ COMPLETE (July 2026)

First Tilemap consumer, 43 tests — details in `log_archive.md`.

---

# The Deion Pivot — Phases E–I

Settled decisions (Jul 28 2026, adversarial review round 1; artifacts in
`review/` during the session, verdict adjudicated with Jesse):
- **Art source: mix** — Jesse hand-draws hero assets (Aseprite → PNG); simple
  tiles/props are code-generated **offline into PNGs** (never runtime rgba).
- **All 6 games get full Deion-world theming**; ChaosTheme neon becomes the
  FX/accent layer.
- **Deployment web-first** (WASM on a static site), then itch.io, Steam later.
- **Web assets fetch-by-default** (boot-phase manifest fetch into a bytes map;
  loaders stay sync); **WebGPU-only at launch**; **games stay standalone**.

## Phase E — Asset Pipeline (engine)

Make pixel art actually work, end-to-end, headless-tested.

| # | Task | Key decisions |
|---|------|---------------|
| E1 ☑ | `TextureFilter` knob | DONE Jul 30 2026. Config default (`GameConfig::with_texture_filter` → `AssetConfig.default_filter`) + per-call `load_texture_filtered`; `Linear` default for plain loads (back-compat) |
| E2 ☑ | `common::SheetGrid` | DONE Jul 30 2026. Tilemap delegates (behavior-identical, test-locked incl. out-of-range passthrough); `uv_rect_checked` for E3/E4 consumers. E4 note: `Deserialize` needs explicit `cell_uv` on the wire |
| E3 | `SpriteAnimation` rework | Named clips over a sheet grid (`AnimationClip { frame_indices, fps, looping }` + clip map + `play(name)`); system writes current UV into `Sprite.tex_region` while playing. Ownership rule documented: SpriteAnimation owns `tex_region` while a clip plays. **Crosses the full SSOT chain — single-agent task, never parallelized** |
| E4 | `load_sprite_sheet()` + `.sheet.ron` | PNG sheet + RON sidecar (grid, filter, named clips). Sheet loads default `Nearest`. **Named clips are the stable API — game code never references raw grid indices.** Schema goes in CLAUDE.md SSOT table |
| E5 | Scene serialization fixes | `create_solid_color` records `#solid:RRGGBB`; serialize `tex_region` + `visible` with `#[serde(default)]` (old scenes load unchanged); `#rgba` becomes a per-sprite save-time error naming the entity — **enforced only after F3 migrates Frogger's tileset** |
| E6 ☑ | Delete `renderer/src/atlas.rs` | DONE Jul 30 2026 (incl. orphaned `TextureError::TextureCreationError`) |
| E7 | Sprite-shader alpha-cutoff | Configurable threshold, conservative default; closes the renderer TECH_DEBT alpha/depth item |
| E8 | Inspector wiring | Via `/add-component` only; [Animation] timeline tab stays backlog |
| E9 | Docs | training.md + crate CLAUDE.md updates for the new APIs |

**Checkpoint: E2 + E4 merged = schema freeze.** Asset production (F2 onward)
does not start before it.

## Phase F — Deion Style Guide + Asset Production

| # | Task | Notes |
|---|------|-------|
| F1 | `docs/DEION_STYLE.md` + castings proposal | World bible (Deion, food-coded world), palette, metrics, per-game castings table, naming, export rules (**no anti-aliased edges** in pixel exports), clips-are-the-API convention. May start before schema freeze |
| F2 | `../games/deion_assets/` + sync script | Canonical asset source; sync copies into each game's `assets/sprites/`; **`--check` hash-compare mode** wired into build + definition of done. No symlinks |
| F3 | `scripts/gen_tiles` offline generator | image-crate bin producing PNGs; first consumers: Frogger lanes (migrates its in-code rgba tileset — unblocks E5's `#rgba` error), Breakout bricks |
| F4 | Placeholder sheets for all 6 games | Agent-made: correct cell size, blocked-out colors, **final clip names** — Phase G never blocks on art |
| F5 | First animated Deion on screen | Validation milestone; needs Phase E complete |

**Metrics:** 16px base cell, nearest filter, 5× integer scale to
`RENDER_UNIT = 80` — one art cell = one world unit = one collider unit.

**Split:** Jesse draws hero sheets (idle/walk/jump/hurt), per-game variants,
key characters, palette sign-off. Agents do everything else.

## Phase G — Re-skin Games 1–6

Order (each independently shippable): **Pong → Frogger → Breakout → Snake →
Space Invaders → Asteroids.** Pong validates the pipeline (smallest, has PNGs);
Frogger validates tile sheets; Breakout validates the scene-RON fixes;
Asteroids last (rotation/animation heavy).

Per game:
- Gameplay entities get sheet sprites; ChaosTheme/bloom/grid kept as accent;
  wireframes debug-only.
- **Collider/velocity audit** against new sprite dimensions (collider overlay,
  C key) — physics ignores `Transform2D.scale`; 1:1 cell/unit is a target for
  new art, never assumed for existing tuning.
- Audit headless tests asserting on `#white`/color tints.
- `deion_assets` sync `--check` in the definition of done.
- README + castings note.

Cross-cutting: **G0** update `/new-game` skill ("Neon look" → "Deion look") ·
**G7** rule-of-2+ promotion sweep after the 3rd re-skin.

## Phase H — WASM Port (spike starts immediately, parallel with E)

**H1 spike ☑ DONE Jul 30 2026** (`coordination/H1_SPIKE.md`, demo in
`../spikes/h1_wasm/`): browser demo renders a fetched texture (wgpu 28 +
winit 0.30, WebGPU, screenshot-verified); 14/14 dependency pass/fail table
with exact feature sets; **audio decision: STAY ON RODIO** (OutputStream +
symphonia decode confirmed in a live browser via an AudioManager-surface
mirror; kira / web-sys shim not needed). Listen test PASSED Jul 30 2026 —
Jesse confirmed all 4 audio checks by ear; the rodio decision is FINAL. Corrections to this phase's
assumptions: gilrs ships a web backend (no no-op gating needed); wgpu builds
for wasm with unchanged Cargo.toml. Forced H2 API change: `load_sound` /
`play_music` become bytes-primary (path versions native-only convenience).

| # | Task | Key decisions |
|---|------|---------------|
| H2 | `web-time` swap | Replaces `Instant`/`SystemTime` in game_loop_manager, timing, lifecycle, achievements |
| H3 | Redraw-driven loop | Rendering moves to `RedrawRequested` + `request_redraw` (one model native+web); `thread::sleep` throttle native-only |
| H4 | Async renderer init | `wasm_bindgen_futures::spawn_local` on wasm; pollster only at the native outer edge. Single-agent |
| H5 | Asset manifest + fetch boot phase | Generated per-game manifest; web boot fetches all entries into a bytes map (loading screen), sync bytes-twin loaders consume; locale dir-scan becomes a manifest list; `include_bytes!` for bootstrap minimum only |
| H6 | `KvStore` trait | Returns `Result`, errors logged never panic; native = JSON files (achievements keep atomic tmp+rename), wasm = localStorage. IndexedDB rejected (KB-scale blobs) |
| H7 | Audio backend | ☑ DECIDED (H1 spike): stay on rodio. Remaining H7 work: gesture-gated `OutputStream` init (start in `disabled()` mode, upgrade on first gesture; `try_default()` Ok does NOT prove the context is running — don't use as a health check) |
| H8 | Incremental wasm CI guard | `cargo check --target wasm32-unknown-unknown` starting on `common`/`ecs`, expanding crate-by-crate |
| H9 | Port all 6 games | Shared `scripts/build_wasm.sh` + index.html template (wasm-bindgen loader) + `[profile.release]` snippet (opt-level="s", lto). Gates on G per game only for final art |

WebGPU-only at launch; WebGL2 fallback revisited at the post-I2 launch review.
`thread::spawn` (lifecycle.rs) feature-gated to no-op on wasm; gilrs does NOT
need gating (H1 finding: gilrs-core has a web-sys Gamepad backend — web
gamepad support may be nearly free). H2–H6 parallelizable across crates now
that the spike is done. Note for H6/H9: 1.83 MB wasm for triangle+audio —
budget `wasm-opt -Oz` + compression; CI smoke tests need a real GPU session
or headless Chrome `--enable-unsafe-swiftshader` (headless Firefox has no
`navigator.gpu`).

## Phase I — Deployment

| # | Task | Notes |
|---|------|-------|
| I1 | Static site skeleton + first game live | `../games/website/`: landing gallery + one dir per game, assembled from per-game `dist/`; GitHub Pages. Verify `.wasm` serving; document WebGPU browser requirements on the page |
| I2 | Remaining games on the site | |
| I3 | itch.io via butler | `scripts/publish_itch.sh`, HTML5 project per game from the same dist zips; page copy/screenshots Jesse-side |
| I4 | `docs/STEAM_CHECKLIST.md` | Doc only — Steam = native packaging + Steamworks, explicitly deferred (Steam doesn't host HTML5) |

---

## Phase C (paused): Games 7–15 — Classic + Action

Resumes after Phase G with Deion styling from day one.

| # | Game | Requires | Deion casting | Key New Patterns |
|---|------|----------|---------------|-----------------|
| 7 | Tetris | Tilemap | TBD (DEION_STYLE.md castings table) | Grid logic, piece rotation, line clearing |
| 8 | Galaga | Lifetime, SpriteAnimation | TBD | Enemy formation paths, multi-bullet patterns |
| 9 | Pac-Man | Tilemap, SpriteAnimation | TBD | Pathfinding AI (BFS), power-up state, ghost modes |
| 10 | Simple Platformer | CameraFollow, SpriteAnimation | **Deion himself — first playable Deion** | Multi-level progression, camera smoothing |
| 11 | Run & Gun | CameraFollow, Tilemap, Lifetime | TBD | Horizontal scroll, level checkpoints |
| 12 | Zelda-style Top-Down | CameraFollow, Tilemap, SpriteAnimation | TBD | Room transitions, NPC dialog, item system |
| 13 | Tower Defense | Tilemap, SpriteAnimation | TBD | Wave spawning, tower placement, pathfinding |
| 14 | Sokoban / Puzzle | Tilemap | TBD | Move history / undo, level editor-compatible format |
| 15 | Metroidvania | CameraFollow, Tilemap, SpriteAnimation | TBD | Ability gating, persistent world state, map system |

## Phase D (paused): Games 16–20 — Complex + Original

| # | Game | Focus |
|---|------|-------|
| 16 | Bullet Hell Shoot-em-up | High entity counts, Lifetime at scale, pattern scripting |
| 17 | Roguelike Dungeon | Procedural generation, fog of war, persistent save state |
| 18 | Fighting Game (simple) | Frame-precise animation, hitboxes, combo system |
| 19 | Strategy / Mini-RTS | Unit selection, pathfinding at scale, fog of war |
| 20 | Original Concept — **Deion the Insiculous platformer** | The capstone: the founding concept, built with the full engine + editor + Deion asset pipeline |

---

## Supporting Infrastructure

Done when they unblock a specific game or pivot phase, not on a fixed schedule.

### Editor Polish (Backlog)
- [ ] Toolbar redesign (cleaner play controls, tool selection)
- [ ] Scene tree enhancements (icons, search, drag-and-drop reparenting)
- [ ] Inspector polish (collapsible sections, color picker, enum dropdowns)
- [ ] Copy/Paste entities (Ctrl+C / Ctrl+V)
- [ ] Multi-entity editing (shared properties, multi-gizmo)
- [ ] Prefab system (save entity as reusable template)
- [ ] Console panel (log output, filter, search)
- [ ] Tilemap editor tab (tile palette, brush, fill tools)
- [ ] Animation timeline tab (deferred from Phase E8)
- [ ] Physics debugger overlay (collider wireframes, velocity vectors)

**Reference:** `crates/editor/IdealEditor.png` for target mockup

### Scripting (after Game 10)
Hot-reloadable Rust script components via `dylib` + a `Script` trait. Unblocks
faster game iteration for Games 11+. Spec preserved in git history.

---

## Technical Debt (High + Medium Only)

Workspace rollup with per-crate counts: root `TECH_DEBT.md`. LOW priority items
are tracked in `crates/*/TECH_DEBT.md`. Resolved items live in `log_archive.md`.

### Medium Priority

**engine_core (1 item):**
- [ ] **ARCH-006: Behaviors hardcoded in scene serialization** — `scene_data.rs`/`scene_loader.rs`/`scene_serializer.rs` match on Behavior variants instead of going through `ComponentRegistry`. Route through a registry/`Custom` variant; pairs with the scripting-phase migration of `ecs/src/behavior.rs`.

**common (2 items):**
- [ ] **ARCH-001: `CameraUniform` duplicated in renderer crate** — Use `common::CameraUniform` everywhere, remove renderer copy.
- [ ] **DRY-002: Volume clamping duplicated cross-crate** (`audio`, `ecs`) — add `clamp_volume()` utility in common.

**ecs_macros (1 item):**
- [ ] **KISS-001: Over-specified `syn` features** — `["full", "parsing"]` where `["derive"]` suffices; compile-time win.

**renderer (1 item):**
- [ ] Alpha-blended sprites vs `depth_write_enabled: true` can punch holes across batches — becomes visible with real alpha-edged art; Phase E7 (alpha-cutoff) closes it.

---

## Development Guidelines

### For Every Game
1. Each game is a standalone cargo project in `../games/<name>/` (sibling to this repo)
2. Depends on `engine_core` (includes physics by default) + `ecs` if needed directly — no editor dep
3. Has a `README.md` with: controls, how to run, what patterns it demonstrates
4. `cargo run` from the game directory launches it
5. **Deion Style**: sprites from `.sheet.ron` sheets per `docs/DEION_STYLE.md` (post-Phase F); ChaosTheme neon is the accent layer

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
- `docs/DEION_STYLE.md` — Deion style guide (lands in Phase F)
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

Completed milestones (Engine Core 2025, Editor Phase 1, Phase 2A standalone
infrastructure, Games 1–6, Phase B engine gaps) live in `log_archive.md`.
