# Technical Debt: input — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § input.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
- [ ] **GPP-L4 (Low, Double Buffer):** first mouse move after startup computes delta against baseline `(0,0)` → spurious warp (`mouse.rs:38-42`); skip delta until a previous position exists.

## Open Items

### [GAP-001] No gamepad backend — Medium
The state model (`GamepadState`, auto-registration, `InputSource::Gamepad` bindings) is complete and tested, but nothing produces gamepad events: no gilrs integration, and winit doesn't carry gamepad input. The default `Gamepad(0, …)` preset bindings are inert.
**Next step:** a `gilrs` poll in the engine event loop translating to `InputEvent::GamepadButton*/GamepadAxisUpdated`. Dead-zone normalization lands with the backend.

### [GAP-002] `MousePosition` / `(f32, f32)` instead of shared Vec2 — Low
`MousePosition` duplicates a 2D vector type; `movement_delta()` returns a bare tuple. Unifying touches `ui`, `editor`, `editor_integration` — do as its own small cross-crate pass.

### [GAP-003] No touch / gesture support — Low (feature gap)
No tap/drag/pinch recognition, no `WindowEvent::Touch` handling. Track in PROJECT_ROADMAP.md if mobile/web targets become real.

### [GAP-004] Binding persistence — Low (feature gap)
No save/load for `InputMapping` (serde on `InputSource` + remapping UI). Needed for "rebind keys" settings screens.

## Metrics (post-June-2026 refactor)

| Metric | Value |
|--------|-------|
| Tests | 62 passing, 0 ignored |
| Clippy warnings | 0 (including `--all-targets`) |
| High priority open | 0 |
| Medium priority open | 1 (GAP-001) |
| Low priority open | 4 |
