# Technical Debt: editor — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § editor (includes the June 2026 design decisions of record: registry macro in `stored_component.rs` is the single source of truth; context.rs split rejected; theme-driven colors).

## Game Programming Patterns Audit (July 2026 — closed; history in `log_archive.md`)
- [ ] **GPP-L7 (Low, document-only):** gizmo drags mutate the world directly between commands (intentional live feedback) — document that scene mutations don't all flow through commands mid-drag.

(GPP-14, GPP-L5 and GPP-L6 resolved Jul 13 2026 — see `log_archive.md`.)

## Metrics

| Metric | Value |
|--------|-------|
| Test coverage | 255 tests (100% pass rate) |
| Files over 600 lines | 0 |
| Clippy warnings | 0 |
| High priority open | 0 |
| Medium priority open | 0 |
| Low priority open | 1 (GPP-L7) |
