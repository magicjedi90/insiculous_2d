# Technical Debt: ecs_macros

Last audited: February 2026

## Summary
- DRY violations: 1
- SRP violations: 0
- KISS violations: 1
- Architecture issues: 1

**Overall Assessment:** The ecs_macros crate is small and focused, providing derive macros for the ECS component system. Issues are minor and relate to over-specified dependencies and small code simplifications.

---

## New Findings (February 2026 Audit)

3 new issues (0 High, 1 Medium, 2 Low)

### [KISS-001] Over-engineered syn features
- **File:** `Cargo.toml:11`
- **Issue:** syn = { features = ["full", "parsing"] } is overkill for simple struct name/field extraction
- **Suggested fix:** Use features = ["derive"] only (80% size reduction)
- **Priority:** Medium | **Effort:** Medium

### [ARCH-001] ComponentMeta trait redefined in tests
- **File:** `tests/derive_test.rs:8-14`
- **Issue:** Trait duplicated instead of imported from ecs crate
- **Suggested fix:** Import from ecs or move trait to shared location
- **Priority:** Low | **Effort:** Small

### [DRY-001] Repeated field extraction pattern
- **File:** `src/lib.rs:33-54`
- **Issue:** Unnamed and Unit branches both return empty vec, could merge with wildcard
- **Suggested fix:** Collapse to `_ => vec![]`
- **Priority:** Low | **Effort:** Small

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 1 (+ 1 test file) |
| Total lines | ~60 |
| Test coverage | 1 test file |
| Dependencies | syn, quote, proc-macro2 |
| High priority issues | 0 |
| Medium priority issues | 1 |
| Low priority issues | 2 |

---

## Recommendations

### Immediate Actions
None required - the crate is functional and small.

### Short-term Improvements
1. **Fix KISS-001** - Reduce syn features to minimize compile time

### Technical Debt Backlog
- ARCH-001: Resolve ComponentMeta trait duplication in tests
- DRY-001: Simplify field extraction match arms
