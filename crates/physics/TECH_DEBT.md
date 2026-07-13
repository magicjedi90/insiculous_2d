# Technical Debt: physics — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § physics.

## Game Programming Patterns Audit (July 2026 — closed; history in `log_archive.md`)
- [ ] **GPP-L10 (Low):** per-contact-pair `Vec<ContactPoint>` alloc per step (`stepping.rs:161-183`) — reuse buffers if profiling says so.

(GPP-08, GPP-09, GPP-10, GPP-L9 resolved Jul 13 2026 — see `log_archive.md`.)

### [EDIT-001] RigidBody config edits on live bodies not pushed — Low
- **File:** `physics_system/sync.rs`
- **Issue:** GPP-09's external-edit detection covers `Transform2D` (teleport) and `Collider` (rebuild); changing `RigidBody` config (body_type, damping, gravity_scale, can_rotate) on a live body still requires recreating it. Complication: `velocity`/`angular_velocity` are writeback fields, so a naive whole-struct compare would false-positive every frame.
- **Fix:** compare only the config fields (or a config sub-struct) against a baseline and rebuild the body on mismatch (preserving pose + velocity).

## Open Items (pre-July-2026 audits)

### [SRP-001] PhysicsWorld handles too many rapier types — Low
- **File:** `physics_world/mod.rs:73-108` — 13 rapier2d types + 4 entity-handle mappings on one struct.
- **Fix:** group into `RapierSimulation` / `RapierSets` / `EntityMapping` sub-structs. (Rapier's API somewhat requires the breadth.)

### [API-001] Missing getter methods for PhysicsSystem timing config — Low
- **File:** `physics_system/mod.rs` — no public getters for `fixed_timestep`, `max_delta_time`, `time_accumulator`.

### [MISSING-001 remainder] Degenerate physics values — Low
- `pixels_per_meter` is validated; **gravity can still be NaN, negative collider dimensions accepted.**

### [SRP-002] Collider clamping inconsistent with builder pattern — Low
- **File:** `components.rs:267-276` — friction/restitution clamped but shape dimensions not validated. Centralize in a `validate()` method.

### Test coverage gaps (feature gaps, not debt)
- No friction/restitution tests, no kinematic body tests, no high-speed tunneling tests.

## Metrics (June 2026)

| Metric | Value |
|--------|-------|
| Largest file | 600 lines (`physics_system/tests.rs`) |
| Test coverage | 64 (55 lib + 6 integration + 3 doc), 0 ignored |
| High priority open | 0 |
| Medium priority open | 0 |
| Low priority open | 6 |
