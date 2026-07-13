# Technical Debt: engine_core — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § engine_core.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
(GPP-07 and DRY-010 resolved Jul 13 2026 — runtime `spawn_prefab` + merge-layer helper; see `log_archive.md`.)
- [ ] **GPP-03 (Medium, Flyweight/DRY):** pong↔breakout duplication — promote only the game-agnostic subset now (ChaosTheme structure, grid-emit helper, visibility helper, `hash_f32`/`game_root`); genre-flavored spawners/flow skeleton wait for game 3's rule-of-three (see `../games/TECH_DEBT.md`).
- [ ] **GPP-06 (cross-ref):** = ARCH-006 below; the concrete unblocking step is `World::add_boxed` so `ComponentData::Dynamic` (`scene_loader.rs:477-506`) stops silently discarding validated components. Pairs with ecs GPP-16.
- [ ] **GPP-05 (Low, Game Loop):** no render interpolation between fixed physics steps — only act if stutter observed on high-refresh displays.
- [ ] **GPP-L2 (Low, Spatial Partition):** O(n²) tag scans in `behavior_runner.rs:473-496` — grid/sensor-based lookup if chaser/collectible counts grow.

## Medium Priority

### [ARCH-006] Gameplay behaviors hardcoded in engine serialization, bypassing ComponentRegistry
- **Files:** `scene_data.rs` (`BehaviorData`), `scene_loader.rs`, `scene_serializer.rs`
- **Issue:** `ComponentData::Dynamic` exists and validates against `global_registry()`, but all Behavior variants are hardcoded match arms. Behaviors are game mechanics; the engine's scene format should not need editing to add one.
- **Fix:** Route behaviors through the registry / a `Custom { type_name, data }` variant with game-registered factories. Pairs with Phase 4 scripting and GPP-06/GPP-16.
- **Priority:** Medium | **Effort:** Large

## Low Priority

### [KISS-002] LifecycleManager over-engineered for single-threaded use
- **File:** `lifecycle.rs` — 7 states + 2 lock flags; `Clone` creates *independent* locks (`lifecycle.rs:51-56`). Document Clone semantics or simplify.

### [DRY-010] `merge_components()` duplicates its merge loop
- **File:** `scene_loader.rs:265-299` — same find-and-replace loop written twice. Extract `apply_component_layer()` helper.

### [DRY-003] GameContext constructed twice with identical fields
- **File:** `game.rs:406-418`, `game.rs:600-612`. Macro/helper blocked by split borrows — acceptable to leave, documented.

### [DRY-011] White texture handle `TextureHandle { id: 0 }` hardcoded in multiple places
- **File:** `ui_integration.rs:51,258` (+ similar in `game.rs`). Use `TextureHandle::WHITE`.

### [SIZE-001] File sizes vs 600-line guideline
- `scene_loader.rs`: ~583 non-test lines — extract component factory helpers from the `add_component_to_entity` match.
- `game.rs`: 643 total incl. docs — extracting `GameRunner` to `game_runner.rs` would resolve.

### [DOC-001] Doc gaps
- `scene_serializer.rs:190` `behavior_to_data()` undocumented; `ui_manager.rs:22` `begin_frame()` undocumented; `contexts.rs:77` `lines` buffer append-vs-replace contract undocumented.
