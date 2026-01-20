# Technical Debt: Editor Change Set Review

Last audited: January 2026

## Summary
- DRY violations: 0
- SRP violations: 0
- KISS violations: 0
- Architecture issues: 1
- Critical bugs/infra issues: 1
- UX/ergonomics nits: 6
- Tooling/repo hygiene nits: 2

---

## Critical Bugs / Infra Issues

### [BUG-001] Mouse button release events are not tracked in editor input mapping
- **File:** `crates/editor/src/editor_input.rs`
- **Lines:** `EditorInputMapping::is_action_just_released`, `EditorInputMapping::update_state`
- **Issue:** `InputHandler` exposes `is_mouse_button_just_pressed` but no `is_mouse_button_just_released`, so `just_released` is hard-coded to `false`. Any editor feature relying on mouse release transitions cannot work reliably.
- **Suggested fix:** Add mouse button release tracking in the `input` crate and plumb it through `EditorInputMapping` (or remove the `just_released` fields until supported).
- **Priority:** Medium

---

## Architecture Issues

### [ARCH-001] Viewport zoom limits ignore `ViewportInputConfig`
- **File:** `crates/editor/src/viewport.rs`, `crates/editor/src/viewport_input.rs`
- **Lines:** `SceneViewport::set_camera_zoom`, `ViewportInputConfig`
- **Issue:** Zoom clamping is hard-coded to `0.1..10.0` while `ViewportInputConfig` exposes `min_zoom`/`max_zoom` that are never applied. This makes configuration misleading and prevents per-viewport limits.
- **Suggested fix:** Thread the `ViewportInputConfig` min/max values into `SceneViewport` (or remove the unused config fields).
- **Priority:** Low

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

### [UX-005] Editor shortcuts do not enforce modifier keys
- **File:** `crates/editor/src/editor_input.rs`
- **Lines:** `EditorInputMapping::set_default_bindings`
- **Issue:** Actions such as `SelectAll`, `Duplicate`, `Undo`, `Redo`, `Copy`, and `Paste` are bound to raw letter keys without modifier checks. This can trigger unintended actions when typing in text fields or using single-key shortcuts.
- **Suggested fix:** Extend input bindings to support modifier combinations (e.g., `Ctrl+Key`) or enforce modifiers in the action handlers.
- **Priority:** Medium

### [UX-006] Selection drag completion is lost when mouse leaves the viewport
- **File:** `crates/editor/src/viewport_input.rs`
- **Lines:** `ViewportInputHandler::handle_input`
- **Issue:** If the cursor leaves the viewport while dragging a selection rectangle, the handler returns early and never emits a click/selection completion, dropping the interaction.
- **Suggested fix:** Keep selection state active when the cursor leaves the viewport and finalize on mouse release.
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
