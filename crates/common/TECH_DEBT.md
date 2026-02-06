# Technical Debt: common

Last audited: January 2026

## Summary
- DRY violations: 1
- SRP violations: 0
- KISS violations: 1
- Architecture issues: 2

**Overall Assessment:** The common crate is well-designed with minimal technical debt. It serves its purpose as a shared types crate effectively.

---

## DRY Violations

### [DRY-001] Duplicate matrix construction pattern in transform.rs
- **File:** `transform.rs`
- **Lines:** 67-90
- **Issue:** The `matrix()` method constructs translation, rotation, and scale matrices inline using `Mat3::from_cols_array()`. This pattern is similar to what exists in `ecs/hierarchy.rs` for `GlobalTransform2D.matrix()`.
- **Suggested fix:** Consider if the matrix construction could be shared, though this is minor since common crate shouldn't depend on ecs.
- **Priority:** Low (cross-crate duplication, acceptable for decoupling)

---

## SRP Violations

None identified. Each file has a single, well-defined responsibility:
- `color.rs` - Color representation and manipulation
- `rect.rs` - Rectangle representation and operations
- `transform.rs` - 2D transform representation
- `camera.rs` - 2D camera and projection
- `macros.rs` - Builder pattern macros

---

## KISS Violations

### ~~[KISS-001] `with_prefixed_fields!` macro requires paste dependency~~ ✅ RESOLVED
- **File:** `macros.rs` (macro removed)
- **Resolution:** Removed the unused `with_prefixed_fields!` macro. The `paste` crate dependency wasn't actually present in Cargo.toml, so the macro would have failed compilation if used.
- **Resolved:** January 2026

---

## Architecture Issues

### [ARCH-001] CameraUniform duplicated in renderer crate
- **Files:** `camera.rs:179-202` (common), `renderer/src/sprite_data.rs` (renderer)
- **Issue:** `CameraUniform` exists in both:
  1. `common::camera::CameraUniform` - GPU-ready struct with bytemuck derives
  2. `renderer::sprite_data::CameraUniform` - Also GPU-ready with same structure

  This is a DRY violation across crates. The renderer should use the common version.
- **Suggested fix:** Remove `CameraUniform` from renderer crate and use `common::CameraUniform` everywhere.
- **Priority:** Medium

### [ARCH-002] Camera2D vs renderer sprite_data Camera2D
- **Files:** `camera.rs` (common), `renderer/src/sprite_data.rs` (renderer)
- **Issue:** Similar to ARCH-001, there may be overlapping `Camera2D` types. The common crate has a comprehensive `Camera2D` with:
  - View/projection matrices
  - Screen/world coordinate conversion
  - World bounds calculation
  - Builder pattern

  The renderer may have its own simplified version.
- **Suggested fix:** Audit renderer's Camera2D usage to ensure common::Camera2D is the canonical source.
- **Priority:** Low (needs verification)

---

## Code Quality Notes

### Strengths
1. **Clean API design** - All types have sensible defaults and builder patterns
2. **Good documentation** - Each file has module docs and method docs
3. **Comprehensive functionality** - Color has lerp/darken/lighten, Rect has intersection/union
4. **Proper derives** - All types have appropriate `Debug, Clone, Copy, Serialize, Deserialize`
5. **Conversion traits** - `From` implementations for common type conversions
6. **Good test coverage** - Each file has focused unit tests

### Minor Observations
- `rect.rs:176-179`: `offset()` is an alias for `translate()` for compatibility - documented but could be considered redundant
- `transform.rs:129-133`: `right()` returns positive Y axis rotated, which is mathematically correct but might be confusing (typically "right" is perpendicular to forward, not up)

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 6 |
| Total lines | ~750 |
| Test coverage | Good (each file has tests) |
| Dependencies | glam, serde, bytemuck, paste |
| High priority issues | 0 |
| Medium priority issues | 1 |
| Low priority issues | 3 |

---

## Recommendations

### Immediate Actions
None required - the crate is clean.

### Short-term Improvements
1. **Fix ARCH-001** - Consolidate CameraUniform to common crate only
2. **Review KISS-001** - Remove unused `with_prefixed_fields!` macro if not planned

### Technical Debt Backlog
- ARCH-002: Verify Camera2D is properly shared across crates
- DRY-001: Accept as intentional decoupling

---

## Cross-Reference with PROJECT_ROADMAP.md

| This Report | PROJECT_ROADMAP.md | Status |
|-------------|-------------------|--------|
| ARCH-001: CameraUniform duplication | Not tracked | New finding |
| KISS-001: Unused macro | Not tracked | New finding |

**New issues to add to PROJECT_ROADMAP.md:**
- ARCH-001: CameraUniform exists in both common and renderer crates

---

## Dependency Graph

```
common
├── glam (math types)
├── serde (serialization)
├── bytemuck (GPU buffer casting)
└── paste (macro helper - only used by unused macro)

Used by:
├── engine_core
├── renderer
├── ecs
├── ui
├── physics
└── audio
```

The common crate correctly sits at the bottom of the dependency hierarchy with no engine crate dependencies.

---

## New Findings (February 2026 Audit)

2 new issues (0 High, 1 Medium, 1 Low)

### [KISS-002] Unused thiserror dependency
- **File:** `Cargo.toml:11`
- **Issue:** thiserror listed but no error types defined in crate
- **Suggested fix:** Remove thiserror from dependencies
- **Priority:** Low | **Effort:** Small

### [DRY-002] Volume clamping duplicated cross-crate
- **File:** (cross-crate: ecs/audio_components.rs, audio/manager.rs)
- **Issue:** Volume clamping pattern duplicated in audio and ecs crates
- **Suggested fix:** Create `clamp_volume()` utility in common crate
- **Priority:** Medium | **Effort:** Small
