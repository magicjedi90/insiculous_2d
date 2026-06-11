# Technical Debt: editor

Last audited: June 2026 (full DRY/SRP/KISS remediation pass)

## Summary
- Open issues: 0 (all previously tracked items resolved June 2026)
- See "Resolved" sections below for history and resolutions

---

## Magic Numbers

### ~~[MAGIC-001] Hardcoded slider ranges in component_editors.rs~~ ✅ RESOLVED
- **Resolution (June 2026):** All 13 ranges extracted to a `mod ranges` block of
  named `RangeInclusive<f32>` constants in `component_editors.rs`; the
  `EditableInspector::f32/vec2` API now takes `RangeInclusive<f32>` directly.
- **File:** `component_editors.rs`
- **Lines:** 57, 63, 69, 109, 133, 195, 201, 207, 277, 357, 382, 387, 392
- **Issue:** Component field ranges are hardcoded inline:
  - Position: -1000.0 to 1000.0
  - Rotation: -PI to PI
  - Scale: 0.01 to 10.0
  - Offset: -100.0 to 100.0
  - Depth: -100.0 to 100.0
  - Velocity: -500.0 to 500.0
  - Angular velocity: -10.0 to 10.0
  - Gravity scale: 0.0 to 2.0
  - Pitch: 0.1 to 3.0
  - Spatial audio distances: 0.0 to 5000.0, 0.0 to 1000.0
  - Rolloff: 0.0 to 5.0
- **Impact:** Changing ranges requires modifying multiple locations; not configurable per-project.
- **Suggested fix:** Extract to a `ComponentFieldRanges` configuration struct or const declarations.
- **Priority:** Low

### ~~[MAGIC-002] Widget ID formula constants in editable_inspector.rs~~ ✅ RESOLVED
- **Resolution (June 2026):** Multipliers promoted to named constants
  `COMPONENT_ID_STRIDE` / `FIELD_ID_STRIDE` with documented collision limits.
- **File:** `editable_inspector.rs`
- **Lines:** 528-530
- **Issue:** FieldId to WidgetId conversion uses hardcoded multipliers:
  ```rust
  let id_value = (id.component_index as u64) * 10000
      + (id.field_index as u64) * 100
      + id.subfield_index as u64;
  ```
  These constants (10000, 100) limit field_index to <100 and subfield_index to <100 before collisions occur.
- **Impact:** Works fine for current components (max 8 fields), but fragile for future expansion.
- **Suggested fix:** Use larger multipliers (1_000_000, 1_000) or switch to string-based hashing.
- **Priority:** Low

### ~~[MAGIC-003] Layout dimensions in editable_inspector.rs~~ ✅ RESOLVED
- **Resolution (June 2026):** Input widths, gaps, and channel-label offsets are
  now `EditableFieldStyle` fields (defaults preserve the previous values), so
  layout is configurable without code changes.
- **File:** `editable_inspector.rs`
- **Lines:** 629, 638, 715, 821-822
- **Issue:** Widget layout dimensions are hardcoded:
  - Slider width: 120.0
  - Value text offset: 130.0
  - Vec2 slider width: 80.0
  - Color slider width: 60.0
  - Color slider height: 12.0
- **Impact:** Not configurable via EditableFieldStyle; requires code changes to adjust layout.
- **Suggested fix:** Add these dimensions to `EditableFieldStyle` struct.
- **Priority:** Low

---

## Metrics

| Metric | Value |
|--------|-------|
| Test coverage | 226 tests (100% pass rate) |
| Files over 600 lines | 0 |
| Clippy warnings | 0 |
| High priority issues | 0 |
| Medium priority issues | 0 |
| Low priority issues | 0 |

---

## Design Decisions (June 2026 remediation)

1. **Component registry macro** — `stored_component.rs` now holds the single
   `editor_component_registry!` invocation that generates `StoredComponent`,
   `ComponentKind` (with `add_default`/`capture`/`remove`/`is_present`/
   `display_name`/`category`), `capture_all_components`, and
   `inspect_all_components`. **Add new editor-visible components there — one
   line — instead of writing match arms.**
2. **context.rs state/delegation split evaluated and rejected** — the
   delegation methods are cohesive accessors over the editor's sub-systems;
   splitting state from delegation would add indirection without reducing
   coupling. Tests were moved to `context/tests.rs` instead, keeping
   `context/mod.rs` under the 600-line limit.
3. **Theme-driven colors** — widget code takes `&EditorTheme` (menu, toolbar,
   hierarchy) or themed style structs (`GizmoPalette`,
   `EditableFieldStyle`, `InspectorStyle`). Struct `Default` impls keep the
   previous literals so an unthemed instance looks identical.

---

## New Findings (February 2026 Audit)

8 new issues found (0 High, 4 Medium, 4 Low)

### ~~DRY-001: Repeated vec-like detection in inspector.rs~~ ✅ RESOLVED
- **File:** `src/inspector.rs`
- **Resolution:** Extracted `has_exact_keys(map, keys)` helper. `is_vec_like()` simplified from 25 lines to 3 lines with 3 calls to the helper.
- **Resolved:** February 2026

### ~~SRP-001/DRY-002: EditorInputMapping action checking tripled~~ ✅ RESOLVED
- **File:** `src/editor_input.rs`
- **Resolution:** Extracted `check_action_with()` generic helper using closure predicates. All 3 methods (`is_action_pressed`, `is_action_just_pressed`, `is_action_just_released`) now delegate to this helper, reducing ~50 lines to ~20.
- **Resolved:** February 2026

### ~~SRP-002: MenuBar.render() mixes layout, interaction, and rendering~~ ✅ RESOLVED
- **File:** `src/menu.rs`
- **Resolution (June 2026):** `render()` is now a 4-line orchestrator over
  `layout_titles()` (pure geometry, unit-tested), `render_title_bar()`,
  `apply_toggle()`, and `render_open_dropdown()`.

### ~~DRY-003: Menu dropdown hardcoded constants~~ ✅ RESOLVED
- **File:** `src/menu.rs`
- **Resolution:** Promoted `ITEM_HEIGHT`, `ITEM_PADDING`, `DROPDOWN_WIDTH` to module-level constants as `DROPDOWN_ITEM_HEIGHT`, `DROPDOWN_ITEM_PADDING`, `DROPDOWN_WIDTH`. All references updated.
- **Resolved:** February 2026

### ~~DRY-004: Gizmo arrow head bounds calculation repeated 4+ times~~ ✅ RESOLVED
- **File:** `src/gizmo.rs`
- **Resolution:** Extracted `centered_handle_rect(&self, center) -> Rect` helper method. All 5 occurrences of centered rect creation now use this helper.
- **Resolved:** February 2026

### ~~ARCH-001: Gizmo render_translate() is 95 lines~~ ✅ RESOLVED
- **File:** `src/gizmo.rs`
- **Resolution (June 2026):** Extracted `render_axis_handle()` (line + hoverable
  arrow) and `begin_drag_if()` (drag-start bookkeeping, reused by rotate/scale).
  All gizmo colors now come from `GizmoPalette` / `EditorTheme::gizmo_palette()`.

### ~~ARCH-002: Component edit result structs have 26-33 fields each~~ ✅ RESOLVED
- **File:** `src/component_editors.rs`
- **Resolution (June 2026):** All five `*EditResult` structs replaced by the
  generic `ComponentEdit<T> { new_value, field_hint }` returned as
  `Option<ComponentEdit<T>>`. The `field_hint` preserves undo merging of
  continuous slider drags. The five near-identical `Set*Command` impls in
  `commands/set_commands.rs` collapsed into the `impl_set_component_command!`
  macro. Note: if multiple fields change in one frame, `field_hint` is the
  last-changed field rather than a hardcoded priority — only the undo
  merge-group label differs.
