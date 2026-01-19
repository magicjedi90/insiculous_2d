# Technical Debt: Editor Change Set Review

Last audited: January 2026

## Summary
- DRY violations: 0
- SRP violations: 0
- KISS violations: 0
- Architecture issues: 0
- UX/ergonomics nits: 4
- Tooling/repo hygiene nits: 2

---

## UX / Ergonomics Nits

### [UX-001] Toolbar position is hard-coded relative to assumed panel sizes
- **File:** `crates/editor/src/context.rs`
- **Lines:** `EditorContext::new()`
- **Issue:** Toolbar is positioned at `Vec2::new(220.0, 34.0)`, which assumes a left panel width and menu height. If dock panel sizes change or menu height differs, the toolbar can overlap or float oddly.
- **Suggested fix:** Compute toolbar position based on the dock layout and menu bar height rather than fixed constants.
- **Priority:** Low

### [UX-002] Widget id offset may collide with other UI widgets
- **File:** `crates/editor/src/dock.rs`
- **Lines:** `impl From<PanelId> for WidgetId`
- **Issue:** Panel widget ids are offset by `+10000`. This is only safe if no other widgets occupy that range. If `ui` uses large ids, collisions are possible.
- **Suggested fix:** Consider a dedicated id namespace or a hash-based id derived from a stable string key.
- **Priority:** Low

### [UX-003] Hierarchy list uses index-based widget ids
- **File:** `examples/editor_demo.rs`
- **Lines:** `render_panel_content()` for `PanelId::HIERARCHY`
- **Issue:** UI ids use `hierarchy_entity_{i}`. If entity ordering changes, UI state can jitter between frames.
- **Suggested fix:** Use the entity id (`entity_id.value()`) in the widget id for stability.
- **Priority:** Low

### [UX-004] Parameter naming inconsistency in key handler
- **File:** `examples/editor_demo.rs`
- **Lines:** `on_key_pressed(&mut self, key, _ctx)`
- **Issue:** `_ctx` is used inside the function, which is misleading (underscore implies unused).
- **Suggested fix:** Rename `_ctx` to `ctx`.
- **Priority:** Low

---

## Tooling / Repo Hygiene Nits

### [TOOL-001] Root dependencies may be example-only
- **File:** `Cargo.toml`
- **Issue:** `editor`, `ui`, `common`, `glam`, `winit`, `env_logger`, `log` were added to root dependencies to support `examples/editor_demo.rs`. If the root crate doesn't otherwise use them, strict linting could flag unused dependencies.
- **Suggested fix:** Consider moving example-only dependencies to `[dev-dependencies]` if feasible.
- **Priority:** Low

### [TOOL-002] IDE metadata file change
- **File:** `.idea/insiculous_2d.iml`
- **Issue:** IDE-generated file changed. If the project excludes IDE files from commits, this should be omitted.
- **Suggested fix:** Ensure repository policy for `.idea` files is followed.
- **Priority:** Low
