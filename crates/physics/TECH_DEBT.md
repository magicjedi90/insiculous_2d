# Technical Debt: physics — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § physics.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
- [ ] **GPP-08 (Medium, Event Queue):** collision buffer contract is implicit (caller must `clear_collision_events()` once per frame) and consumers must `.to_vec()` the borrowed slice — replace with drain-style `take_collision_events() -> Vec<CollisionData>`; ordering becomes structural, snapshot clone disappears.
- [ ] **GPP-09 (Medium, Dirty Flag):** sync only ADDS bodies (`physics_system/sync.rs:41-81`) — live `Transform2D`/`Collider` edits are silent no-ops; add change detection to push ECS→rapier edits (also fixes editor live collider edits). Rides ecs GPP-04's change tracking.
- [ ] **GPP-10 (Medium, Observer):** synchronous collision callbacks are non-reentrant (fire under `&mut world`) and redundant with the event bus — deprecate in favor of bus + polled buffer.
- [ ] **GPP-L9 (Low):** deferred ops as two parallel tuple-Vecs with ordering-by-convention (`physics_system/mod.rs:88-90`) — single `Vec<DeferredBodyOp>` enum queue.
- [ ] **GPP-L10 (Low):** per-contact-pair `Vec<ContactPoint>` alloc per step (`stepping.rs:161-183`) — reuse buffers if profiling says so.

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
| Test coverage | 62 (58 lib + 1 integration + 3 doc), 0 ignored |
| High priority open | 0 |
| Medium priority open | 3 (GPP-08/09/10) |
| Low priority open | 6 |
