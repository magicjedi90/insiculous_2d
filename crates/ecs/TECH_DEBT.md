let# Technical Debt: ecs — LIVE (open items only)

Last audited: February 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § ecs.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
(GPP-04 + SRP-003 resolved Jul 13 2026 — dirty-flagged transform propagation, see `log_archive.md`.)
- [ ] **GPP-02 (Decision of record, Data Locality):** `ComponentStore` = `HashMap<EntityId, Box<dyn Component>>` is the accepted simplicity tradeoff. Future path: dense `Vec<T>` columns / archetype storage + bitset queries (see Future Enhancements below). **Trigger to revisit:** profiling shows component access dominating a frame, or games routinely exceed ~a few thousand live entities.
- [ ] **GPP-16 (Medium, Singleton):** `global_registry()` registration list is hardcoded (`component_registry.rs:92-107`) — games can't register components; add a one-shot init extension point (root of engine_core ARCH-006).
- [ ] **GPP-L1 (Low):** `world.entities()` allocates a `Vec<EntityId>` per call in hot paths — prefer `entity_ids()` iterator.
- [ ] **GPP-L12 (Low):** `EventBus` single-frame lifetime — document the emit-before-read contract.

## Open Items (pre-July-2026 audits)

### [DRY-001] Repeated entity existence checks — Low
- **File:** `world.rs:216-284` — `if !self.entities.contains_key(entity_id)` repeated 7+ times.
- **Fix:** `ensure_entity_exists()` helper. (Explicit checks aid debugging, hence Low.)

### [DRY-003] Duplicate matrix computation in GlobalTransform2D — Low
- **File:** `hierarchy.rs:143-201` — sin/cos computed multiple times in `matrix()`, `mul_transform()`, `transform_point()`.
- **Fix:** cache sin/cos or extract rotation application helper.

### [DRY-004] Repeated builder pattern in audio_components.rs — Low
- **File:** `audio_components.rs` — three components with nearly identical `with_volume()` clamping.
- **Fix:** `VolumeControl` trait with default impl, or helper fn. (Overlaps common DRY-002.)

---

## Future Enhancements (Not Technical Debt)

- **Proper archetype storage (ground-up rewrite):** dense columnar `Vec<T>`, typed push/get without `Box<dyn Component>`, archetype migration, drop handling. Gated on the GPP-02 trigger above.
- **System scheduling:** dependency graph, parallel execution, system groups.
- **Component introspection:** reflection, dynamic add/remove by name (pairs with GPP-16), editor integration.
- **Memory pooling** for entity/component allocations.

## Metrics

| Metric | Value |
|--------|-------|
| Test coverage | 187 tests (100% pass rate) |
| `#[allow(dead_code)]` | 0 |
| High priority open | 0 |
| Medium priority open | 1 (GPP-16) |
| Low priority open | 5 |
