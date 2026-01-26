# Technical Debt: engine_core

Last audited: January 2026

## Summary
- DRY violations: 5
- SRP violations: 3
- KISS violations: 1 (1 resolved)
- Architecture issues: 3

---

## DRY Violations

### [DRY-001] Duplicate AudioManager placeholder pattern
- **File:** `game.rs`
- **Lines:** 352-354 (initialize_if_needed) and 381-382 (update_game_logic)
- **Issue:** The pattern of creating a placeholder AudioManager is duplicated:
  ```rust
  let mut placeholder_audio = AudioManager::new().ok();
  let audio = audio_manager.or(placeholder_audio.as_mut());
  ```
- **Suggested fix:** Extract into a helper method `get_audio_manager_or_placeholder()` or refactor to handle optional audio in a single place.
- **Priority:** Medium

### [DRY-002] Duplicate coordinate transformation logic in ui_integration.rs
- **File:** `ui_integration.rs`
- **Lines:** 39-41, 75-76, 93-96, 130-131, 143-144, 161-162, 184-185
- **Issue:** The coordinate transformation from screen space to world space is repeated 7 times:
  ```rust
  let center_x = bounds.x + bounds.width / 2.0 - window_size.x / 2.0;
  let center_y = window_size.y / 2.0 - (bounds.y + bounds.height / 2.0);
  ```
- **Suggested fix:** Extract to a helper function `screen_to_world_coords(x, y, width, height, window_size) -> Vec2`.
- **Priority:** Medium

### [DRY-003] Duplicate GameContext creation pattern
- **File:** `game.rs`
- **Lines:** 357-365, 385-393, 525-533
- **Issue:** The GameContext struct construction is repeated in 3 places with identical fields.
- **Suggested fix:** Create a factory method `create_game_context()` or use a builder pattern.
- **Priority:** Low (refactoring was started but this remains)

### [DRY-004] Repeated hex color parsing error handling
- **File:** `scene_loader.rs`
- **Lines:** 497-503, 505-511
- **Issue:** The hex color parsing error handling pattern is duplicated twice for 6-char and 8-char cases:
  ```rust
  .map_err(|_| SceneLoadError::InvalidTextureRef(format!("Invalid hex color: {}", hex)))?
  ```
- **Suggested fix:** Extract to a helper function `parse_hex_byte(hex, start, end)` that handles the error mapping.
- **Priority:** Low

### [DRY-005] Duplicate surface error recovery pattern
- **File:** `render_manager.rs`
- **Lines:** 131-138, 168-175
- **Issue:** The surface recreation logic after a surface error is duplicated:
  ```rust
  Err(RendererError::SurfaceError(_)) => {
      if let Err(e) = renderer.recreate_surface() {
          log::error!("Failed to recreate surface: {}", e);
          return Err(e);
      }
      log::debug!("Surface recreated after loss");
      Ok(())
  }
  ```
- **Suggested fix:** Extract to a helper method `handle_surface_error(&mut self)` on RenderManager.
- **Priority:** Low

---

## SRP Violations

### [SRP-001] GameRunner still has multiple responsibilities
- **File:** `game.rs`
- **Lines:** 162-446
- **Issue:** Despite recent refactoring, GameRunner still orchestrates:
  1. Game state (game instance, initialized flag)
  2. Glyph texture caching (prepare_glyph_textures)
  3. Rendering coordination (render_frame)
  4. Audio management fallback (placeholder creation)
  5. Scene lifecycle management
- **Note:** This was marked as improved from 8+ to ~5 responsibilities. The glyph caching could be extracted.
- **Suggested fix:** Extract glyph texture caching into `UIManager` or a new `GlyphCacheManager`.
- **Priority:** Medium

### [SRP-002] BehaviorRunner handles multiple behavior types inline
- **File:** `behavior_runner.rs`
- **Lines:** 104-243
- **Issue:** The `update()` method has one large match block handling 7 different behavior types (PlayerPlatformer, PlayerTopDown, ChaseTagged, Patrol, FollowEntity, FollowTagged, Collectible). Each behavior type is 15-40 lines inline.
- **Suggested fix:** Extract each behavior into separate handler methods or a trait-based pattern.
- **Priority:** Medium (readability concern, performance is already optimized)

### [SRP-003] EngineApplication duplicates GameRunner functionality
- **File:** `application.rs`
- **Lines:** 28-311
- **Issue:** `EngineApplication` provides similar functionality to `GameRunner` but with a different API pattern (scene-based vs trait-based). This creates parallel code paths.
- **Suggested fix:** Consider deprecating `EngineApplication` in favor of the simpler `Game` trait API, or clearly document the use cases for each.
- **Priority:** Low (documented as "deprecated" in training.md but code still active)

---

## KISS Violations

### ~~[KISS-001] Glyph cache key includes color unnecessarily~~ ✅ RESOLVED
- **File:** `contexts.rs`
- **Resolution:** Removed `color_rgb` from `GlyphCacheKey`. Glyph textures are now color-agnostic grayscale alpha masks. The text color is applied at render time by setting the sprite color. This allows the same glyph texture to be reused for any color, eliminating memory waste and cache misses. Also removed the unused `_color` parameter from `create_glyph_texture()` and fixed the sprite rendering to use the actual text color instead of white.

### [KISS-002] Over-engineered lifecycle state machine
- **File:** `lifecycle.rs`
- **Lines:** 1-274
- **Issue:** The `LifecycleManager` implements 7 states with 2 locks (init_lock, shutdown_lock) and complex state transition validation. While thread-safe, most engine usage is single-threaded.
- **Suggested fix:** For typical use cases, a simpler enum-based state machine without locks would suffice. The current implementation could be feature-gated for advanced multi-threaded scenarios.
- **Priority:** Low (working but over-engineered for current use cases)

---

## Architecture Issues

### [ARCH-001] Dual API pattern creates confusion
- **Files:** `game.rs`, `application.rs`
- **Issue:** Two parallel APIs exist:
  1. `Game` trait + `run_game()` (recommended, simpler)
  2. `EngineApplication` (older, more complex)
  Both are fully exported and documented, which may confuse users.
- **Suggested fix:**
  - Clearly mark `EngineApplication` as deprecated in code
  - Move it to a `legacy` module
  - Update lib.rs to not re-export it by default
- **Priority:** Medium

### [ARCH-002] Timer vs GameLoopManager overlap
- **Files:** `timing.rs` (Timer), `game_loop_manager.rs` (GameLoopManager)
- **Issue:** Both `Timer` and `GameLoopManager` track time with similar functionality:
  - Timer: start_time, last_update, delta_time, elapsed
  - GameLoopManager: last_frame_time, delta_time, frame_count, total_time
- **Suggested fix:** `GameLoopManager` should use `Timer` internally instead of duplicating time tracking logic.
- **Priority:** Low

### [ARCH-003] Inconsistent module visibility
- **File:** `lib.rs`
- **Lines:** 25-43
- **Issue:** Some modules are `pub mod` while others are `mod` with `pub use`:
  - `pub mod behavior_runner;` (contents accessible)
  - `mod game;` with `pub use game::*;` (contents accessible, module not)
  - `pub mod scene_manager;` (contents accessible)
  This inconsistency makes the API surface unclear.
- **Suggested fix:** Standardize on one pattern. Recommended: all `mod X;` with selective `pub use` for public API.
- **Priority:** Low

---

## Previously Resolved (Reference)

These issues from ANALYSIS.md have been resolved:

| Issue | Resolution |
|-------|------------|
| SRP: GameRunner.update_and_render() | FIXED: Extracted 7 focused methods |
| Debug println!() in behavior.rs | FIXED: Replaced with log::info!() |
| Excessive clone() in behavior loop | FIXED: ~85% reduction in allocations |
| Redundant GameContext creation | PARTIAL: Helper method added, but pattern remains |
| Double-check in asset_manager | FIXED: Single check for entire frame |
| First-frame UI flicker | FIXED: Font rendering bug resolved |

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 20 |
| Total lines | ~3,200 |
| Test coverage | 53 tests (100% pass rate) |
| High priority issues | 1 |
| Medium priority issues | 5 |
| Low priority issues | 7 |

---

## Recommendations

### Immediate Actions
1. **Fix KISS-001** (glyph cache key) - Highest impact, causes memory waste

### Short-term Improvements
2. **Fix DRY-001 and DRY-002** - Common patterns that could be extracted
3. **Address ARCH-001** - Deprecate EngineApplication to reduce confusion

### Technical Debt Backlog
- SRP-001: Extract glyph caching from GameRunner
- SRP-002: Refactor BehaviorRunner into handler methods
- ARCH-002: Unify Timer and GameLoopManager

---

## Cross-Reference with PROJECT_ROADMAP.md

| This Report | PROJECT_ROADMAP.md | Status |
|-------------|-------------------|--------|
| KISS-001: Glyph cache color | Medium: "Glyph texture key collision" | Known, unresolved |
| SRP-001: GameRunner | Completed "SRP refactoring" | Partially resolved, more extraction possible |
| ARCH-001: Dual API | Not tracked | New finding |
| DRY-002: Coord transform | Not tracked | New finding |
| ARCH-002: Timer overlap | Not tracked | New finding |

**New issues to add to PROJECT_ROADMAP.md:**
- DRY-002: Coordinate transformation duplication in ui_integration.rs
- ARCH-001: EngineApplication deprecation needed
- ARCH-002: Timer/GameLoopManager consolidation

---

## Future Enhancements (Not Technical Debt)

These features would enhance the engine but are not required for current functionality:

### Advanced Asset Pipeline
- Multi-threaded asset loading with progress tracking
- Asset dependency management and automatic reloading
- Compressed texture formats (DXT, ETC, PVRTC)
- Font asset preprocessing and character set optimization

### Performance Optimizations
- Frustum culling for large worlds
- Occlusion culling for 2D layers
- Instanced rendering for identical sprites
- GPU particle systems

### Advanced Physics
- Physics material system (friction, bounciness)
- Terrain/collision mesh support
- Joint constraints for complex objects
- Trigger volumes for game logic

### Editor Integration
- Live scene editing with immediate preview
- Visual debugger for entity/component state
- Performance profiler integration
- Hot-reloading for assets and scenes
