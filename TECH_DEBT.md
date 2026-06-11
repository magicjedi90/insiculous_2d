# Technical Debt — Workspace Rollup

Last updated: June 11, 2026 (post tech-debt pass: engine_core 5 Mediums + ui 2 Mediums resolved)

This file is the high-level index of technical debt across the workspace.
Each crate carries the authoritative detail in its own `crates/<name>/TECH_DEBT.md`;
this rollup tracks open counts, the medium-priority items worth scheduling, and
what the 2026 audit passes resolved. High + Medium items are mirrored in
`PROJECT_ROADMAP.md`.

> The previous version of this file was a January 2026 review of the editor
> change set. All of its items were resolved or superseded by the June 2026
> remediation passes: mouse-button release tracking now exists via the shared
> `ButtonTracker` (`is_source_just_released`), editor shortcuts use real
> modifier combinations (Ctrl+S, Ctrl+Shift+P, …), `EditorInputMapping`
> delegates to the generic `InputMapping<EditorAction>`, and panel rendering
> moved out of `examples/editor_demo.rs` into `editor_integration`.

---

## Status by Crate

| Crate | Last Audited | Open (High / Med / Low) | Notes |
|-------|--------------|--------------------------|-------|
| `audio` | Jun 2026 (remediated) | 0 / 0 / 0 | Clean; remaining gaps (streaming, spatial runtime) reclassified as feature work |
| `common` | Feb 2026 | 0 / 2 / 3 | `CameraUniform` duplicated in renderer; cross-crate volume clamping |
| `ecs` | Feb 2026 | 0 / 0 / 4 | Dual/archetype storage fully removed; remaining items are micro-DRY |
| `ecs_macros` | Feb 2026 | 0 / 1 / 2 | Over-specified `syn` features (compile-time cost) |
| `editor` | Jun 2026 (remediated) | 0 / 0 / 0 | All tracked items resolved; component registry macro is single source of truth |
| `editor_integration` | Jun 2026 (remediated) | 0 / 0 / 2 | No file picker (Phase 2+); menu actions matched by label string |
| `engine_core` | Jun 2026 (debt pass) | 0 / 1 / 6 | Only ARCH-006 (behavior registry, Phase 2+) remains Medium |
| `input` | Jun 2026 (restructured) | 0 / 1 / 3 | Generic `InputMapping<A>`; open items are feature gaps (gamepad backend) |
| `physics` | Jun 2026 (remediated) | 0 / 0 / 3 | All correctness findings fixed; remaining items organizational |
| `renderer` | Jun 2026 (remediated) | 0 / 0 / 3 | Bloom/panic/dead-code fixes landed; ~700 lines dead code removed |
| `ui` | Jun 2026 (debt pass) | 0 / 1 / 3 | FontManager + context.rs splits done; general text input (JUN-T1) remains |

Workspace-wide invariants (verified by the June 2026 audits): no files over
600 lines, no `#[allow(dead_code)]`, no `unwrap()` outside tests, and
`cargo clippy --workspace` is clean.

---

## Open Medium-Priority Items

### engine_core (1)
- **[ARCH-006]** Behaviors hardcoded in scene serialization, bypassing `ComponentRegistry` — route through a registry/`Custom` variant; pairs with the Phase 4 scripting-crate migration of `ecs/src/behavior.rs` (Large; Phase 2+)

### ui (1)
- **[JUN-T1]** Text input is numeric-only and keyboard-layout-blind — blocks editor rename/search widgets; needs winit character events plumbed through `input`

### input (1)
- **[GAP-001]** No gamepad backend — state model is complete and tested, but nothing produces gamepad events (no gilrs integration). Dead-zone normalization should land with the backend.

### common (2)
- **[ARCH-001]** `CameraUniform` duplicated in renderer — use `common::CameraUniform` everywhere
- **[DRY-002]** Volume clamping duplicated across `audio` and `ecs` — `clamp_volume()` utility in common

### ecs_macros (1)
- **[KISS-001]** `syn = { features = ["full", "parsing"] }` is overkill for struct name/field extraction — `["derive"]` only

---

## Resolved in the 2026 Audit Passes (Highlights)

Full details live in each crate's `TECH_DEBT.md` "Resolved" sections.

- **ecs (Feb):** broken archetype/dual storage deleted entirely (single
  HashMap-based path), hierarchy cycle detection, `WorldHierarchyExt`
  extraction, generation-validated component ops
- **renderer (Jun):** bloom blur bug (uniform rewrite between passes), sprite
  overflow panic → growing `DynamicBuffer`, NaN-safe depth sort, per-frame
  bind-group/clone churn eliminated, `sprite.rs` split, ~700 lines dead code
  removed
- **ui (Jun):** glyph bitmaps shared as `Arc<[u8]>` (no per-frame copies),
  focused-widget state survives unseen frames, theme bypass fixed
  (`TextInputStyle`), dead draw/interact APIs deleted
- **input (Jun):** stale mouse-delta bug, unbind/rebind leak, strict
  action-edge semantics; `InputMapping<A>` made generic, `ButtonTracker<T>`
  deduplicates device state, `ThreadSafeInputHandler`/`init()`/`InputError`
  deleted (~250 lines)
- **audio (Jun):** per-play full-buffer clone removed (`Arc<[u8]>` +
  `Cursor`), live-sink volume re-apply, clamping at point of use,
  `stop(handle)` implemented, dead `PlaybackState` deleted
- **physics (Jun):** collision event clear/append contract (no stale
  re-emission, no sub-step loss), world-space contact points, one-update
  forces, raycast normalization, `PhysicsError`/`MovementConfig` deleted,
  directory splits under 600 lines
- **engine_core (Jun):** orphaned `scene_saver.rs`/`file_operations.rs`
  deleted (single save pipeline), Behavior↔BehaviorData conversion collapsed
  to one `From` pair, dead `game_loop.rs` deleted, clippy-clean incl.
  `--all-targets`
- **editor + editor_integration (Jun):** component registry macro
  (`stored_component.rs`) as single source of truth, `ComponentEdit<T>`
  writeback, 1,100-line files split into feature directories, full theme
  routing, duplicate `ComponentKind`/dispatch deleted
- **engine_core (Jun 11 debt pass):** `GlyphTextureCache` extracted from
  `GameRunner`, BehaviorRunner match split into per-variant handlers,
  asset_manager `unwrap()` → `let-else`, `ToastStyle` on `AchievementManager`
  (+ `reset()` logs save errors), all `lib.rs` glob re-exports made explicit
- **ui (Jun 11 debt pass):** `context.rs` (~990 lines) split into `context/`
  (mod/text/widgets/tests), `font.rs` split into `font/` (FontManager facade +
  `GlyphCache` + `layout`); public API unchanged, all files <600 lines

---

## Process

- Audit a crate → record findings in `crates/<name>/TECH_DEBT.md` with
  `[CATEGORY-NNN]` ids, priority, and suggested fix
- Fix High/Medium where the fix is contained; move resolved items to the
  crate's "Resolved" section with the resolution
- Update this rollup and the `PROJECT_ROADMAP.md` Technical Debt section
  after each audit pass
- Feature gaps (missing systems, e.g. audio streaming, gamepad backend,
  touch input) are tracked as roadmap work, not debt
