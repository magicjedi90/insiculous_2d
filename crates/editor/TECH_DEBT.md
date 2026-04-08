# Technical Debt: editor

Last audited: January 2026

## Summary
- Magic numbers: 3
- Priority: All Low

---

## Magic Numbers

### [MAGIC-001] Hardcoded slider ranges in component_editors.rs
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

### [MAGIC-002] Widget ID formula constants in editable_inspector.rs
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

### [MAGIC-003] Layout dimensions in editable_inspector.rs
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
| Total source files | 14 |
| Test coverage | 136 tests (100% pass rate) |
| High priority issues | 0 |
| Medium priority issues | 0 |
| Low priority issues | 3 |

---

## Recommendations

### Immediate Actions
None required - all issues are low priority and the code functions correctly.

### Future Improvements
1. **MAGIC-001** - Extract field ranges to configuration when adding per-project customization
2. **MAGIC-002** - Increase ID multipliers if adding components with many fields
3. **MAGIC-003** - Add layout dimensions to EditableFieldStyle if UI customization needed

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

### SRP-002: MenuBar.render() mixes layout, interaction, and rendering
- **File:** `src/menu.rs:246-302`
- **Issue:** 57-line method handles bounds calculation, click detection, state management, AND dropdown rendering
- **Suggested fix:** Split into update_menu_layout(), render_menu_titles(), handle_menu_interactions()
- **Priority:** Medium | **Effort:** Medium

### ~~DRY-003: Menu dropdown hardcoded constants~~ ✅ RESOLVED
- **File:** `src/menu.rs`
- **Resolution:** Promoted `ITEM_HEIGHT`, `ITEM_PADDING`, `DROPDOWN_WIDTH` to module-level constants as `DROPDOWN_ITEM_HEIGHT`, `DROPDOWN_ITEM_PADDING`, `DROPDOWN_WIDTH`. All references updated.
- **Resolved:** February 2026

### ~~DRY-004: Gizmo arrow head bounds calculation repeated 4+ times~~ ✅ RESOLVED
- **File:** `src/gizmo.rs`
- **Resolution:** Extracted `centered_handle_rect(&self, center) -> Rect` helper method. All 5 occurrences of centered rect creation now use this helper.
- **Resolved:** February 2026

### ARCH-001: Gizmo render_translate() is 95 lines
- **File:** `src/gizmo.rs:197-291`
- **Issue:** Large method with repeated handle rendering pattern
- **Suggested fix:** Extract `render_gizmo_handle()` helper
- **Priority:** Low | **Effort:** Medium

### ARCH-002: Component edit result structs have 26-33 fields each
- **File:** `src/component_editors.rs`
- **Issue:** TransformEditResult, SpriteEditResult, etc. have per-field boolean flags creating boilerplate
- **Suggested fix:** Use change mask pattern or generic EditResult<T>
- **Priority:** Medium | **Effort:** Large
