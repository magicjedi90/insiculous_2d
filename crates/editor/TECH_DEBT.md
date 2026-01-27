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
