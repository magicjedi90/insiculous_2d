# Technical Debt: engine_core

Last audited: June 2026

## Summary
- High priority: 0
- Medium priority: 1
- Low priority: 6
- Resolved since February 2026 audit: DRY-001, DRY-003 (partial), ARCH-001, KISS-001, LOGIC-003, ARCH-002 (reclassified not-an-issue)
- Resolved June 2026 (post-audit cleanup): DRY-008, DRY-009, DEAD-001, QUAL-001, ARCH-005 (decision: keep)
- Resolved June 2026 (medium-priority sweep): SRP-001, SRP-002, LOGIC-002, ARCH-007, ARCH-003

---

## Medium Priority

### [ARCH-006] Gameplay behaviors hardcoded in engine serialization, bypassing ComponentRegistry
- **Files:** `scene_data.rs` (`BehaviorData`), `scene_loader.rs`, `scene_serializer.rs`, `scene_saver.rs`
- **Issue:** `ComponentData::Dynamic` exists and validates against
  `ecs::component_registry::global_registry()`, but all Behavior variants
  (PlayerPlatformer, ChaseTagged, Patrol, …) are hardcoded match arms. Behaviors are game
  mechanics; the engine's scene format should not need editing to add one.
- **Fix:** Route behaviors through the registry / a `Custom { type_name, data }` variant with
  game-registered factories. Larger design change; track for Phase 2+.
- **Priority:** Medium | **Effort:** Large

---

## Low Priority

### [KISS-002] LifecycleManager over-engineered for single-threaded use
- **File:** `lifecycle.rs`
- 7 states + 2 lock flags; `Clone` creates *independent* locks (`lifecycle.rs:51-56`), not
  shared state — a cloned manager silently stops tracking the original. Lock poisoning is
  now handled via `unwrap_or_else` (LOGIC-003 resolved). Document Clone semantics or simplify.

### [DRY-010] `merge_components()` duplicates its merge loop
- **File:** `scene_loader.rs:265-299` — same find-and-replace loop written twice (overrides,
  then inline components). Extract `apply_component_layer()` helper.

### [DRY-003] GameContext constructed twice with identical fields
- **File:** `game.rs:406-418` and `game.rs:600-612`. Down from 3 sites; a macro or helper
  is blocked by split borrows — acceptable to leave, documented here.

### [DRY-011] White texture handle `TextureHandle { id: 0 }` hardcoded in multiple places
- **File:** `ui_integration.rs:51,258` (+ similar in `game.rs`). Extract
  `const WHITE_TEXTURE: TextureHandle`.

### [SIZE-001] File sizes vs 600-line guideline
- `scene_loader.rs`: ~583 non-test lines — at the limit; the `add_component_to_entity`
  match (lines 316-529) is the bulk; extract component factory helpers.
- `game.rs`: 643 total — over once docs included; extracting `GameRunner` to
  `game_runner.rs` would resolve. `scene_serializer.rs`/`scene_saver.rs` are large on disk
  but ~273/~293 non-test lines (fine).

### [DOC-001] Doc gaps
- `scene_serializer.rs:190` `behavior_to_data()` (public) undocumented;
  `ui_manager.rs:22` `begin_frame()` undocumented; `contexts.rs:77` `lines` buffer
  append-vs-replace contract undocumented.

---

## Resolved (Reference)

| Issue | Resolution |
|-------|------------|
| DRY-001/DRY-006/DRY-007 placeholder audio + ctx duplication | Consolidated into `initialize_and_update()` |
| DRY-002 coord transforms (ui_integration) | Helpers extracted (Feb 2026) |
| DRY-004 hex color parsing | `parse_hex_byte` helper (Feb 2026) |
| DRY-005 surface error recovery | `handle_render_error` helper (Feb 2026) |
| SRP-003/ARCH-001 EngineApplication dual API | Deleted entirely (Feb 2026) |
| KISS-001 glyph cache key color | Color removed from key; grayscale masks (Feb 2026) |
| LOGIC-001 `partial_cmp().unwrap()` depth sort | `total_cmp()` (Feb 2026) |
| LOGIC-003 lifecycle lock unwraps | All use `unwrap_or_else` poison recovery + test coverage |
| ARCH-002 Timer vs GameLoopManager | Reclassified not-an-issue: Timer is a user-facing utility, GameLoopManager is the engine frame timer; no duplicate tracking in GameRunner |
| ARCH-004 duplicate UIManager accessors | Consolidated to `ui_context()` (Feb 2026) |
| DRY-008 two parallel World→RON save pipelines | Resolved (Jun 2026): `scene_saver.rs` and `editor/file_operations.rs` turned out to be ORPHANED files — never declared as modules in their crates (editor doesn't even depend on engine_core). Both deleted; `scene_serializer.rs` is the single save pipeline, consumed by `editor_integration`. |
| DRY-009 Behavior ↔ BehaviorData conversion ×3 | Resolved (Jun 2026): single pair of `From` impls in `scene_data.rs` (load + save direction); loader uses `.into()`, serializer uses `BehaviorData::from`. Adding a variant now means: enum + the two `From` arms. |
| DEAD-001 `game_loop.rs` dead code | Resolved (Jun 2026): `game_loop.rs` + `tests/game_loop.rs` deleted, exports removed from `lib.rs`/`prelude.rs`. `GameLoopManager` is the only frame timer. |
| QUAL-001 clippy warnings | Resolved (Jun 2026): orphaned doc comment removed (game.rs), needless borrow fixed (game.rs), `push_arc` now derives segment count from sweep angle (also removed duplicated `CIRCLE_SEGMENTS / 2` at call sites), `!= true` assertion fixed, `Lifecycle` trait moved above test module. `cargo clippy -p engine_core --all-targets` is clean. |
| ARCH-005 `grid/` single-game code | Decision (Jun 2026): KEEP in engine as a general-purpose deformable-grid effect (and candidate for editor grid visualization). Module docs updated to state this and that `Default` tuning values are starting points. Pong remains the reference consumer. |
| SRP-001 GameRunner owned glyph texture caching | Resolved (Jun 2026): extracted `GlyphTextureCache` (`glyph_texture_cache.rs`) — owns the key→handle map and the prepare scan; `GameRunner` just calls `prepare()`/`textures()`. Cache-miss bookkeeping unit-tested headless. |
| SRP-002 BehaviorRunner giant match | Resolved (Jun 2026): one private handler per variant (`update_player_platformer`, `update_patrol`, …) plus a `BehaviorCommands` struct and `apply_commands()`; no logic change. |
| LOGIC-002 `unwrap()` on asset_manager | Resolved (Jun 2026): `initialize_and_update` now uses `let Some(...) else { log::warn!(...); return; }` — production code is `unwrap()`-free. |
| ARCH-007 achievement toast styling hardcoded | Resolved (Jun 2026): `ToastStyle` struct (dimensions, colors, border, font sizes) with `Default` matching the old appearance; `AchievementManager::set_toast_style()`/`toast_style()`; `reset()` now logs its save error like `unlock()`. |
| ARCH-003 glob re-exports in lib.rs | Resolved (Jun 2026): all 15 `pub use module::*` globs replaced with explicit re-export lists; prelude semantics unchanged; whole workspace compiles. |

---

## What's Healthy (June 2026 audit)

- Manager pattern holds: `RenderManager`, `WindowManager`, `UIManager`, `SceneManager`,
  `GameLoopManager` are each single-purpose and small.
- `SceneManager` vs `SceneLoader` separation is clean (stack management vs deserialization).
- `particles/` is genuinely reusable (config builder, pooling, no game-specific tuning);
  the emitter-in-ECS / particles-in-manager split is intentional — worth a module-level
  doc note.
- Production code is `unwrap()`-free (LOGIC-002 resolved); errors are typed
  (`thiserror`) throughout the scene pipeline.
- `ChaosMode` correctly carries selection only — no gameplay logic in the engine.
