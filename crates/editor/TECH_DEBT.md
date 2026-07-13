# Technical Debt: editor — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § editor (includes the June 2026 design decisions of record: registry macro in `stored_component.rs` is the single source of truth; context.rs split rejected; theme-driven colors).

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
- [ ] **GPP-14 (Medium, Command):** `CreateEntityCommand`/`DeleteEntityCommand` mint a NEW `EntityId` on undo/redo (`commands/entity_commands.rs:45-48,114-125`) — `Selection` and later commands referencing the old id go stale; remap on undo/redo (expose old→new id).
- [ ] **GPP-L7 (Low, document-only):** gizmo drags mutate the world directly between commands (intentional live feedback) — document that scene mutations don't all flow through commands mid-drag.

(GPP-L5 and GPP-L6 resolved Jul 13 2026 — see `log_archive.md`.)

## Metrics

| Metric | Value |
|--------|-------|
| Test coverage | 251 tests (100% pass rate) |
| Files over 600 lines | 0 |
| Clippy warnings | 0 |
| High priority open | 0 |
| Medium priority open | 1 (GPP-14) |
| Low priority open | 1 (GPP-L7) |
