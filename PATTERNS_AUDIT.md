# Game Programming Patterns Audit

**Date:** July 13, 2026
**Reference:** [Game Programming Patterns](https://gameprogrammingpatterns.com/contents.html) by Robert Nystrom
**Scope:** All engine crates (`crates/*`) plus the games in `../games/` (pong, breakout)
**Method:** Full-codebase sweep against the book's 19 patterns (Command, Flyweight, Observer, Prototype, Singleton, State, Double Buffer, Game Loop, Update Method, Bytecode, Subclass Sandbox, Type Object, Component, Event Queue, Service Locator, Data Locality, Dirty Flag, Object Pool, Spatial Partition), cross-referenced against every `crates/*/TECH_DEBT.md` and `PROJECT_ROADMAP.md`. All file:line references verified against the working tree at audit time.

**Overall verdict:** The codebase is in strong shape. Component composition, the event bus, the editor's undo/redo Command stack, the particle Object Pool, and Flyweight asset sharing are all textbook implementations, and the June 2026 audit passes eliminated most global-state and per-frame-allocation sins. The findings below are design-debt and missed opportunities, not correctness bugs — each includes a fix plan grounded in the relevant book pattern.

Severity: **High** = architectural gap worth scheduling; **Medium** = real design debt, fix when touching the area; **Low** = polish, batched or opportunistic.

---

## Findings Table

| ID | Pattern | Severity | Location | Tracked before this audit? |
|----|---------|----------|----------|---------------------------|
| GPP-01 | State | ~~High~~ ✅ Resolved Jul 13 2026 | `crates/ecs/src/behavior.rs` | No |
| GPP-02 | Data Locality | High (latent) | `crates/ecs/src/component.rs` | Partial (ecs "Future Enhancements") |
| GPP-03 | Flyweight / DRY | Medium | `../games/pong` ↔ `../games/breakout` | No |
| GPP-04 | Dirty Flag | Medium | `crates/ecs/src/hierarchy_system.rs` | No (SRP-003 adjacent) |
| GPP-05 | Game Loop | Low | `crates/engine_core/src/game_loop_manager.rs` | No |
| GPP-06 | Type Object / Bytecode | Medium | `crates/ecs/src/behavior.rs`, `scene_loader.rs` | Yes (engine_core ARCH-006) |
| GPP-07 | Prototype | Medium | `crates/engine_core/src/scene_loader.rs` | Partial (DRY-010) |
| GPP-08 | Event Queue | Medium | `crates/physics/src/physics_world/stepping.rs` | No |
| GPP-09 | Dirty Flag / Observer | Medium | `crates/physics/src/physics_system/sync.rs` | No (documented footgun, untracked) |
| GPP-10 | Observer | Medium | `crates/physics/src/physics_system/` | Partial (ARCH-002 resolved differently) |
| GPP-11 | Component | Medium | `../games/breakout/src/types.rs` | No |
| GPP-12 | Type Object | Medium | `../games/breakout/src/levels.rs` | No (ARCH-101 same family) |
| GPP-13 | Component / DRY | Medium | `crates/editor_integration/src/panel_renderer/inspector.rs` | No |
| GPP-14 | Command | Medium | `crates/editor/src/commands/entity_commands.rs` | No |
| GPP-15 | Dirty Flag | Medium | `crates/renderer/src/sprite/batch.rs` + `engine_core/src/game.rs` | No (ARCH-007 adjacent) |
| GPP-16 | Singleton | Medium | `crates/ecs/src/component_registry.rs` | Partial (via ARCH-006) |
| GPP-17 | — (magic numbers) | ~~Medium~~ ✅ Resolved Jul 13 2026 | `../games/breakout/src/gameplay.rs` | No |
| GPP-L1..L12 | various | Low | see Low table | mixed |

---

## High Severity

### GPP-01 · State — behavior bool soup while a real FSM sits unused — ✅ RESOLVED (Jul 13 2026, see `log_archive.md` § ecs)

**Evidence:** `crates/ecs/src/behavior.rs:215-225`:

```rust
pub struct BehaviorState {
    pub timer: f32,
    pub patrol_toward_b: bool,
    pub is_chasing: bool,
    pub is_waiting: bool,
}
```

Patrol/chase logic branches on these flags ad hoc in `crates/engine_core/src/behavior_runner.rs:282-333`. This is exactly the boolean-flag soup the [State chapter](https://gameprogrammingpatterns.com/state.html) exists to eliminate — invalid combinations (`is_chasing && is_waiting`) are representable and nothing enforces transitions. Meanwhile the same crate ships a clean, tested State implementation — `crates/ecs/src/state_machine.rs` (`StateMachine` + `HierarchicalStateMachine`, with `previous`/`elapsed`/`just_entered`) — that **no code in the workspace uses** (verified: only defined, re-exported, and tested; zero consumers).

**Fix plan (State):** Define `enum PatrolPhase { PatrollingTowardA, PatrollingTowardB, Waiting, Chasing }` and store `StateMachine<PatrolPhase>` in `BehaviorState` (keep `timer` or use the FSM's `elapsed`). Rewrite the patrol/chase handlers in `behavior_runner.rs` as a match on the current state with explicit transitions. Deletes three bools, makes illegal states unrepresentable, and gives `state_machine.rs` its first real consumer (or, if after this it still has none elsewhere planned, that module should be questioned instead — dead capability is its own debt).

### GPP-02 · Data Locality — HashMap-of-boxes component storage (decision of record)

**Evidence:** `crates/ecs/src/component.rs:36-41` — `ComponentStore { components: HashMap<EntityId, Box<dyn Component>> }`, one HashMap per type. Every component access is hash lookup → heap pointer chase → downcast; `World::query_entities` (`world.rs:407-418`) scans all entities and does one `has_type` hash probe per required component. This is the storage layout the [Data Locality chapter](https://gameprogrammingpatterns.com/data-locality.html) warns about: zero contiguity, cache-hostile at scale.

**Context:** This is a *deliberate, documented* simplicity tradeoff (`ecs/CLAUDE.md`; the broken archetype storage was intentionally deleted in Feb 2026 — PATTERN-001/002). At current scenes (dozens–hundreds of entities) it is not measurable. `ecs/TECH_DEBT.md` lists "Proper Archetype Storage" under Future Enhancements, but there is no trigger condition recorded anywhere.

**Fix plan (Data Locality):** No code change now. Record as decision-of-record in `crates/ecs/TECH_DEBT.md`: the accepted layout, the future path (dense `Vec<T>` columns or archetype storage, bitset-accelerated queries), and the trigger to revisit (profiling shows component access dominating a frame, or games routinely exceeding ~a few thousand live entities — e.g. a bullet-hell entry in the 20 Games Challenge).

---

## Medium Severity

### GPP-03 · Flyweight / DRY — pong↔breakout duplication (promote the generic subset; defer the rest)

*Severity revised High → Medium after review with Jesse: pong and breakout are both paddle-and-ball games, so some of this similarity is genre coincidence rather than a missing engine layer. The finding is split accordingly below.*

**Evidence:** the two games copy-paste gameplay-adjacent machinery that differs only in tuning constants:

- `chaos_theme.rs` — identical struct + `for_mode()` + `particle_count_mult` machinery; the diff between the two files is 4 color-literal lines out of 62.
- `effects.rs` — structurally identical `ParticleConfig::burst(...).with_*()` preset builders.
- `spawning.rs` — both define `spawn_paddle` / `spawn_wall` / `spawn_background` and a same-shaped `spawn_ball`.
- `step_and_emit_grid` duplicated near-verbatim (`pong/src/gameplay/mod.rs:63-74` vs `breakout/src/gameplay.rs:122-131`), including the F1 collider-overlay toggle and the magenta `debug::draw_colliders(...)` call.
- `update_entity_visibility` (hide sprites in menu states) duplicated (`pong/src/gameplay/flow.rs:109-126` vs `breakout/src/gameplay.rs:479-494`).
- The `Serving`/`Playing`/`GameOver` input-transition skeleton duplicated (`breakout/src/gameplay.rs:171-198` vs `pong/src/gameplay/flow.rs:12-30`).
- Small utilities: `hash_f32` (`breakout/src/gameplay.rs:16`, `pong/src/power_ups.rs:14`), `game_root()` (`pong/src/main.rs:22`, `breakout/src/levels.rs:24`).

Per the book's Flyweight framing, shared *intrinsic* machinery should exist once with games supplying *extrinsic* tuning — but only where the machinery is actually game-agnostic. Both games being paddle-and-ball arcade games, the duplicated items split into two buckets:

**Fix plan, part 1 — promote now (game-agnostic; nothing here knows what a paddle is):**
1. `ChaosTheme` structure + `for_mode()` + `particle_count_mult` — `ChaosMode` is already an engine-wide concept and every challenge game needs per-mode presentation; engine owns the struct, games supply the palette (extend `chaos_mode.rs`, no new module needed).
2. `step_and_emit_grid` + the F1 collider-overlay toggle as one helper — `grid/` is already an engine module; this is its natural driver.
3. `update_entity_visibility(world, hidden)` (hide sprites while in menu states) — small, generic.
4. `hash_f32` and `game_root()` into `common` / `engine_core` (`game_root` may partially duplicate `GameConfig.asset_base_path` — reconcile rather than add).

Promotion protocol per the standing directive: engine tests per piece, refactor both games onto it, doc updates — ideally before scaffolding game 3 so `/new-game` inherits it.

**Fix plan, part 2 — deferred (genre-flavored; wait for rule-of-three):** `spawn_paddle`/`spawn_ball`/`spawn_wall` shapes, the particle preset *semantics* (`paddle_hit_burst` etc. — the builder API they share is already engine), and the `Serving`/`Playing`/`GameOver` input-flow skeleton. These may be paddle-genre coincidence. Decision point: when Space Invaders (game 3) is built, promote whatever it duplicates a third time; whatever it doesn't need stays game code. Promoting them now risks an `arcade` module shaped like "games we happened to build first" — exactly the utility pollution to avoid.

### GPP-04 · Dirty Flag — transform hierarchy fully recomputed every frame

**Evidence:** `crates/ecs/src/hierarchy_system.rs:88-113` — `TransformHierarchySystem::update` unconditionally recomputes `GlobalTransform2D` for every entity every frame: full root scan, then recursive `propagate_transforms` (`:141-164`) which clones the parent global and allocates a children `Vec` per node. Static subtrees pay full price forever. This is the exact motivating example of the [Dirty Flag chapter](https://gameprogrammingpatterns.com/dirty-flag.html).

**Fix plan (Dirty Flag):** Add change tracking to `Transform2D` mutation (a dirty bit set by `get_mut::<Transform2D>` at the `ComponentStore` level, or a per-entity dirty set maintained by the world) plus parent-change marking from `WorldHierarchyExt::set_parent`. `update` then propagates only dirty subtrees (a dirty parent dirties its descendants). Also folds in the tracked ecs SRP-003 (double iteration) — one traversal, dirty-gated. Regression test: mutate one leaf in a deep hierarchy, assert siblings' `GlobalTransform2D` are not recomputed (e.g. via a computation counter or by checking untouched values are byte-identical without a recompute path).

### GPP-05 · Game Loop — no render interpolation (Low)

**Evidence:** the loop is better than a first read suggests: `PhysicsSystem` runs a genuine fixed-timestep accumulator with a spiral-of-death cap (`crates/physics/src/physics_system/update.rs:43-63`, `MAX_STEPS_PER_UPDATE = 8`), and `GameLoopManager` clamps runaway deltas (`MAX_DELTA_TIME = 0.1`). Game logic receives variable dt — a legitimate hybrid the [Game Loop chapter](https://gameprogrammingpatterns.com/game-loop.html) endorses. What's missing is the chapter's last refinement: rendering samples the latest fixed physics state with no interpolation, so at frame rates that don't divide evenly into the 1/60 physics step, physics-driven sprites can visibly stutter (position snaps to step boundaries).

**Fix plan (Game Loop):** Only if/when stutter is observed on high-refresh displays (120/144 Hz): have the physics→ECS writeback keep previous and current body transforms and blend by `time_accumulator / fixed_timestep` at render time. Not worth doing speculatively; record the trigger.

### GPP-06 · Type Object / Bytecode — behaviors are a closed enum; the data-driven path is stubbed

**Evidence:** `Behavior` is a closed enum (`crates/ecs/src/behavior.rs:13-108`) matched exhaustively in `behavior_runner.rs:136-186`; adding one behavior touches ecs + four engine_core files. Per the [Type Object chapter](https://gameprogrammingpatterns.com/type-object.html), behaviors are game content and shouldn't require engine edits. The escape hatch exists but is dead: `ComponentData::Dynamic` (`scene_loader.rs:477-506`) validates against `global_registry()` then **discards the created component** — `World` has no type-erased `add_boxed(entity, TypeId, Box<dyn Component>)` counterpart to the monomorphized `add_component<T>`.

**Tracked:** engine_core **ARCH-006** (Medium/Large, paired with Phase 4 scripting).

**Fix plan (Type Object):** Covered by ARCH-006; this audit adds the concrete unblocking step — implement `World::add_boxed` so `ComponentData::Dynamic` actually attaches components, then migrate behaviors to registry-created components with game-registered factories. Until Phase 4, the enum is acceptable; the dead `Dynamic` path silently swallowing data is the part worth fixing early (it logs a warn and drops user scene data on the floor).

### GPP-07 · Prototype — prefabs are load-time-only

**Evidence:** `PrefabData` + override merging (`scene_data.rs:92-97`, `scene_loader.rs:240-274`) is a solid data-driven Prototype for *scene instantiation* — but `SceneInstance` (`scene_loader.rs:22-34`) discards `data.prefabs` after loading. There is no runtime "instantiate prefab by name", so the pattern can't serve its primary purpose from the [Prototype chapter](https://gameprogrammingpatterns.com/prototype.html): spawning many copies at runtime (bullets, enemies, drops). Breakout works around it by spawning balls/pickups from hand-rolled component code. Related: `merge_components` is O(n²) string-name matching (tracked as DRY-010 for its duplicated loop).

**Fix plan (Prototype):** Retain the prefab table on `SceneInstance` (or `SceneManager`) and add `spawn_prefab(world, assets, name, overrides) -> EntityId` reusing the existing `merge_components` + instantiate path. Games then declare a "Ball" prefab once in RON instead of duplicating spawn code (feeds GPP-03's spawner consolidation).

### GPP-08 · Event Queue — collision buffer contract is implicit; consumers must `.to_vec()`

**Evidence:** the collision pipeline is a correct Event Queue with two API-level footguns, both already paid for once:
1. **Implicit clear/append ordering:** `PhysicsWorld::step()` appends and never clears; the driver must call `clear_collision_events()` exactly once before the first sub-step (`stepping.rs:41-49,191-197`; done in `physics_system/update.rs:48`). Any new driver that forgets re-delivers stale `started` events forever — the June 2026 "stale event re-emission" bug was this exact class.
2. **Borrow-based read:** `collision_events()` returns `&[CollisionData]` borrowed from the physics world, so consumers that want to mutate physics/world in response must snapshot with `.to_vec()` first (a documented CLAUDE.md footgun; every game does the clone).

The [Event Queue chapter](https://gameprogrammingpatterns.com/event-queue.html)'s point is that the queue's lifecycle should be owned by the queue, not by caller discipline.

**Fix plan (Event Queue):** Replace the borrow-read with a drain: `PhysicsSystem::take_collision_events() -> Vec<CollisionData>` (internally swaps with an empty pooled Vec). Taking ownership removes the `.to_vec()` requirement, removes one clone per consumer, and makes "clear" structural — the buffer is empty because it was taken, so the ordering contract enforces itself. Keep `collision_events()` briefly as a deprecated shim; migrate pong/breakout/pickups. The event-bus delivery path (`world.emit_event`) is unaffected.

### GPP-09 · Dirty Flag / Observer — physics sync only ADDS; live edits are silent no-ops

**Evidence:** `sync_entity_to_physics` (`crates/physics/src/physics_system/sync.rs:41-81`) inserts bodies/colliders only when absent; after creation rapier is authoritative and editing `Transform2D` or `Collider` on a live entity does nothing (documented in physics/CLAUDE.md and the root CLAUDE.md footguns, and visible in the editor: the collider overlay updates instantly while the simulation keeps stale shapes). This is a missing change-detection design — the engine has no way to *notice* the ECS-side edit.

**Fix plan (Dirty Flag):** Piggyback on GPP-04's change tracking: when `Transform2D`/`Collider`/`RigidBody` on an entity with a live body is mutated, mark it; `sync_entity_to_physics` then pushes the edit (`set_body_transform` for transforms, collider rebuild for shape changes). This turns two documented footguns (this one and "editor collider edits don't reach the simulation") into working behavior. Until then it stays a footgun — now at least a *tracked* one.

### GPP-10 · Observer — synchronous collision callbacks are a non-reentrant legacy channel

**Evidence:** collision callbacks are `Box<dyn FnMut(&CollisionData) + Send + Sync>` invoked inline while `PhysicsSystem::update` holds `&mut self` and `&mut world` (`physics_system/mod.rs:60-66`, `update.rs:82-89`). A callback cannot touch the world or physics — only captured `Arc`/atomics — which is the coupling problem the [Observer chapter](https://gameprogrammingpatterns.com/observer.html) warns about with synchronous notification. The engine already has the better channel: collision events go to the world event bus and to the polled `collision_events()` buffer, which is how the games and `engine_core::pickups` actually consume them.

**Fix plan (Event Queue over Observer):** Deprecate `with_collision_callback`/`add_collision_callback` in favor of the event bus + polled buffer; migrate the remaining callback users (mostly tests) and delete in a later pass. One channel, decoupled, no `Send + Sync` contortions.

### GPP-11 · Component — breakout mirrors world state in a shadow `Vec<Brick>`

**Evidence:** `../games/breakout/src/types.rs:16-24,44` — `Brick { entity, value, color, hits_left, drop }` held in `self.bricks: Vec<Brick>`, hand-synced with the ECS on every destruction (`gameplay.rs:329`: `self.bricks.remove(i)` alongside the entity despawn). Gameplay state (`hits_left`, `drop`) lives outside the World, so the ECS and the mirror can drift — the class of bug the [Component chapter](https://gameprogrammingpatterns.com/component.html) exists to prevent.

**Fix plan (Component):** Move per-brick state into a `BrickState { value, hits_left, drop }` component on the brick entity; query `Pair<BrickState, Transform2D>` where the Vec is iterated today. Destruction becomes "despawn the entity" with no parallel bookkeeping. Pairs naturally with GPP-12.

### GPP-12 · Type Object — brick behavior is a stringly-typed mini-language

**Evidence:** brick kinds are encoded as strings in `EntityTag` — `"armored3+drop_multiball"` — parsed by `parse_brick_tag` (`../games/breakout/src/levels.rs:130-152`), with the grammar existing only in that parser and its test table; malformed tokens degrade with `eprintln!`. The editor shows the tag as an opaque read-only string, so designers hand-edit the mini-language in RON. This is a [Type Object](https://gameprogrammingpatterns.com/type-object.html) expressed as strings, same smell family as the tracked editor_integration **ARCH-101** (menu-label string matching).

**Fix plan (Type Object):** Promote `BrickSpec { hits: u8, drop: Option<DropKind> }` to a real serializable component (registered via `/add-component` so it's scene-loadable and inspector-editable with typed fields). Keep the tag string as authoring sugar if desired (loader converts tag → component once), but the runtime and editor operate on the typed form. Blocked-on note: game-defined components in scenes need GPP-06/GPP-16's registry extensibility — a breakout-local component can't ride `ComponentData::Dynamic` until `World::add_boxed` exists, so short-term the component is attached in `levels.rs` after load (still removes runtime re-parsing).

### GPP-13 · Component / DRY — editable inspector bypasses the component registry

**Evidence:** `crates/editor_integration/src/panel_renderer/inspector.rs:79-241` hand-writes ~11 near-identical blocks (`if let Some(c) = world.get::<T>() { EditableInspector … apply_component_edit … }`) — one per component type — while the *read-only* inspector path is already registry-driven via `inspect_all_components` from the `editor_component_registry!` macro. The "add a component = one registry line" claim holds everywhere except this file, which the `/add-component` skill must patch by hand.

**Fix plan (registry dispatch):** Extend `editor_component_registry!` to also generate the editable-path dispatch (each entry already knows its type and editor function; generate the `capture → edit → apply_component_edit` block per entry, mirroring how `ComponentKind`/`capture_all_components` are generated). Then `/add-component` genuinely becomes one line, and the 11 blocks collapse.

### GPP-14 · Command — undo/redo of create/delete mints new EntityIds

**Evidence:** `crates/editor/src/commands/entity_commands.rs:45-48,114-125` — `CreateEntityCommand::execute` and `DeleteEntityCommand::undo` create a **new** entity and overwrite `self.entity`. Any `Selection` (or later command in the history referencing the old id) silently goes stale across an undo/redo cycle. The [Command chapter](https://gameprogrammingpatterns.com/command.html) calls this out: commands that reify object identity must keep references valid across undo (Godot's UndoRedo keeps stable object pointers for the same reason).

**Fix plan (Command):** Cheapest correct fix at current scope: after any undo/redo that recreated an entity, remap — have the command expose `(old_id, new_id)` and `CommandHistory`/caller update `Selection` and any queued commands. Longer-term (if command chains referencing entities grow): reserve/reuse stable ids on recreate (the ECS generation counter currently forbids resurrection, so remapping is the pragmatic choice). Add a regression test: create → delete → undo → assert selection still resolves.

### GPP-15 · Dirty Flag — sprite batches rebuilt from scratch every frame

**Evidence:** every frame, engine_core re-queries all sprites and rebuilds every batch (`crates/engine_core/src/game.rs:419-453`: fresh `SpriteBatcher`, re-add every sprite, re-sort, clone batches out), and `SpriteBatcher::clear()` (`renderer/src/sprite/batch.rs:116-120`) empties instance Vecs. Capacity is retained (good), the per-batch `sorted` bool is a proper mini dirty flag (good), but for a mostly-static scene the entire CPU-side rebuild is waste the [Dirty Flag chapter](https://gameprogrammingpatterns.com/dirty-flag.html) targets. Related tracked item: renderer ARCH-007 (the scratch-Vec copy in `prepare_sprites`).

**Fix plan (Dirty Flag):** Once GPP-04's change tracking exists, reuse it: rebuild batches only when any `Sprite`/`Transform2D`/`GlobalTransform2D` changed or an entity was created/destroyed (a world change-counter is enough — no per-sprite granularity needed at first). Static scenes then upload nothing new. Measure before doing anything finer-grained.

### GPP-16 · Singleton — global component registry is not extensible

**Evidence:** `crates/ecs/src/component_registry.rs:27,92-107` — `static COMPONENT_REGISTRY: OnceLock<ComponentRegistry>` with a hardcoded built-in registration list. To the book's credit this is the *approved* singleton shape (write-once, immutable `&'static`, no hidden mutation — the [Singleton chapter](https://gameprogrammingpatterns.com/singleton.html)'s complaints don't apply). The debt is that games cannot register their own components, which is the mechanical root of ARCH-006 and the blocker for GPP-12's typed brick component riding the scene pipeline.

**Fix plan:** Provide a one-shot extension point before first use — e.g. `init_global_registry(extra: impl FnOnce(&mut ComponentRegistry))` called from `run_game` with registrations supplied via `GameConfig`, falling back to built-ins-only. Keeps the immutable-after-init property; removes the closed-world limitation.

### GPP-17 · Magic numbers — breakout tuning escaped constants.rs — ✅ RESOLVED (Jul 13 2026, see `log_archive.md` § games)

**Evidence:** `../games/breakout/src/gameplay.rs` — inline grid-impulse tuning (`strength: 200.0/260.0/700.0`, `radius: 70.0/90.0/160.0` at `:277-278, 373-374, 415-416`), brick battle-damage factors (`*= 0.65` ×3, `emissive *= 0.5` at `:316-319`), lost-ball bounds pad (`+60.0` at `:392-393`) — while `constants.rs` exists for exactly this.

**Fix plan:** Hoist to `constants.rs` with named constants. Mechanical.

---

## Low Severity (batch/opportunistic)

| ID | Pattern | Finding | Fix |
|----|---------|---------|-----|
| GPP-L1 | Object Pool / Data Locality | `world.entities()` allocates a fresh `Vec<EntityId>` per call in hot paths: render scan (`engine_core/src/game.rs:75`), behavior runner (`behavior_runner.rs:126`), hierarchy (`hierarchy_system.rs:94`), physics (`physics_system/update.rs:23`, `sync.rs:85`) | Use the non-allocating `entity_ids()` iterator where no mutation follows; reusable scratch buffers elsewhere. (Renderer's `prepare_sprites` scratch Vec already tracked as ARCH-007.) |
| GPP-L2 | Spatial Partition | O(n²) tag-proximity scans: `find_nearest_tagged_position` / `check_tagged_overlap` (`engine_core/src/behavior_runner.rs:473-496`) scan all entities per querying entity. Rapier's BVH broad-phase correctly covers physics; these tag scans bypass it | Fine at current entity counts. If chaser/collectible counts grow: uniform grid keyed by tag, or route overlap checks through rapier sensors. Note: `engine_core/src/grid/` is a *visual* spring-mass effect, not a spatial partition — don't reach for it |
| GPP-L3 ✅ Jul 13 | Singleton | `SoundHandle::new()` uses a process-global `static NEXT_ID: AtomicU32` (`audio/src/sound.rs:14-16`) — ids survive across `AudioManager` instances, unusable for serialization/determinism | Instance-local `next_handle` counter on `AudioManager`, like `TextureManager` (`texture.rs:127`) |
| GPP-L4 ✅ Jul 13 | Double Buffer | First mouse move after startup computes delta against baseline `(0,0)` → spurious warp (`input/src/mouse.rs:38-42`) | Skip delta on the first `update_position` (Option-al previous position) |
| GPP-L5 ✅ Jul 13 | Command / Data Locality | `CommandHistory::enforce_limit` uses `Vec::remove(0)` — O(n) shift per push past the cap (`editor/src/commands/mod.rs:164-168`) | `VecDeque` + `pop_front` |
| GPP-L6 ✅ Jul 13 | Command | Ctrl+Z/Ctrl+Y call `mark_dirty()` even when the undo/redo stack is empty — a no-op keypress dirties a clean scene (`editor_integration/src/editor_game/shortcuts.rs:124-135`) | `mark_dirty()` only when `undo()`/`redo()` actually applied a command (return a bool) |
| GPP-L7 | Command | Gizmo drags mutate `Transform2D` directly each frame, pushing one merged command only on release (`editor_integration/src/editor_game/viewport_interaction.rs:117-149`) | Intentional (live visual feedback) — document the invariant: nothing may depend on all scene mutations flowing through commands mid-drag |
| GPP-L8 | Flyweight | `GlyphInfo` stores `character`/`font_size` duplicating its cache key (`ui/src/font/glyph_cache.rs:38-42`); `TextDrawData` duplicates chars (tracked ui ARCH-003). Per-entity `Behavior` components carry full copies of shared "kind" config (move speeds, ranges) — the Type Object of GPP-06 would make that shared | Strip redundant fields opportunistically; behavior-config sharing rides GPP-06 |
| GPP-L9 | Command | Physics deferred ops are two parallel tuple-Vecs with ordering-by-convention — resets drained before velocities (`physics/src/physics_system/mod.rs:88-90`, `update.rs:33-39`) | Single `Vec<DeferredBodyOp>` enum queue: ordering becomes structural, next deferred op is one variant away |
| GPP-L10 | Object Pool | Breakout balls/pickups are spawn/destroy churned; physics allocates a `Vec<ContactPoint>` per contact pair per step (`physics_world/stepping.rs:161-183`) | Acceptable at arcade scale; pool if profiling ever says otherwise |
| GPP-L11 | Update Method | `breakout/src/gameplay.rs` is one 495-line file where pong splits the identical structure into `gameplay/{mod,balls,flow,…}` | Mirror pong's split next time breakout is touched |
| GPP-L12 | Event Queue | The ECS `EventBus` is single-buffered with per-frame flush — an event emitted after a reader ran that frame is missed | Acceptable; document the emit-before-read contract on `EventBus`. Double-buffer only if a real ordering bug appears |

---

## Already Good — patterns the codebase implements well

- **Component** — blanket `Component` impl over `Any + Send + Sync` (`ecs/src/component.rs:22-34`); entities are pure composition, no inheritance anywhere.
- **Event Queue** — `EventBus` (`ecs/src/event.rs`): typed per-event-type buffers, emit/read/flush-per-frame; physics collisions delivered by polling, not callback soup.
- **Command (buffer)** — `BehaviorCommands` + ordered `apply_commands()` (`behavior_runner.rs:32-44,409-470`) defers mutations collected during iteration; clean borrow-conflict avoidance.
- **Command (undo/redo)** — `EditorCommand` trait with `try_merge`, `push_already_executed`, macro-generated `Set*Commands` with field-hint merge, `MacroCommand` grouping (`editor/src/commands/`). GPP-14 is the one gap.
- **Object Pool** — `particles/manager.rs` ring buffer: fixed slab, round-robin overwrite-oldest, zero per-frame allocations.
- **Flyweight** — sprites hold `texture_handle` ids, never pixels; `Arc<[u8]>` sound bytes decoded from a shared cursor; `Arc<[u8]>` glyph bitmaps; bind groups cached per handle; `TextureHandle::WHITE` shared 1×1.
- **Update Method** — uniform `System::update(world, dt)`; `SystemRegistry::update_all` isolates per-system panics with `catch_unwind`.
- **Game Loop (physics half)** — fixed-timestep accumulator with catch-up cap and spiral-of-death guard (`physics_system/update.rs:43-63`); `MAX_DELTA_TIME` clamp in `GameLoopManager`.
- **State** — pong/breakout use data-carrying FSM enums (`TitleScreen { selection }`, `GameOver { won }`), not flag soup; `EditorPlayState` likewise; `ecs::StateMachine` itself is a textbook implementation (see GPP-01 — it just needs users).
- **Service Locator** — done the recommended way: `GameContext` bundles services and is passed explicitly; no global mutable service statics anywhere in the workspace (grep-verified).
- **Singleton avoidance** — managers are owned instances; the one global (`global_registry()`) is write-once immutable (see GPP-16 for its extensibility gap).
- **Prototype (load-time)** — prefab + override merging in the scene loader; physics preset catalog (`RigidBody::player_platformer()` etc.).
- **Double Buffer** — `ButtonTracker` just_pressed/just_released cleared once per frame; wheel/mouse deltas accumulate-then-clear.
- **Spatial Partition** — correctly delegated to rapier's BVH broad-phase and `QueryPipeline` rather than reimplemented.
- **Dirty Flag (small wins)** — `SpriteBatch.sorted` skip-if-sorted; editor `is_dirty` save flag with title-bar `*`, set/cleared at all the right places.

---

## Stale Documentation Found During Audit

- `training.md` "Current Known Limitations" listed *"Glyph texture cache includes color in key (memory waste)"* — **resolved Feb 2026** (engine_core KISS-001): `GlyphKey { font_id, character, size_tenths }` excludes color; `glyph_texture_cache.rs:18` documents color-agnostic keys. Corrected in this pass.
- `crates/editor/TECH_DEBT.md` claimed "Open issues: 0" — no longer accurate given GPP-14 + GPP-L5/L6/L7; header updated with a pointer to this audit.

## Suggested Order of Attack

1. **GPP-01** (behavior FSM) — self-contained, deletes a bug class, activates dead capability.
1b. **GPP-03 part 1** (generic subset: ChaosTheme structure, grid-emit helper, visibility helper, small utils) — do before game 3 so `/new-game` inherits it; part 2 waits for Space Invaders to confirm rule-of-three.
3. **GPP-08 + GPP-10** (physics event API) — one crate, removes two documented footguns.
4. **GPP-13** (registry-driven editable inspector) — makes `/add-component` honest.
5. **GPP-04 → GPP-09 → GPP-15** (change tracking, then its two consumers) — one design, three payoffs.
6. GPP-11/GPP-12/GPP-17 next time breakout is touched; GPP-14 next time editor commands are touched; the rest opportunistically.
