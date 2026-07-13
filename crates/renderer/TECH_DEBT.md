# Technical Debt: renderer — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § renderer.

## Game Programming Patterns Audit (July 2026) — see root `PATTERNS_AUDIT.md`
- [ ] **GPP-15 (Medium, Dirty Flag):** sprite batches rebuilt from scratch every frame even for static scenes (`sprite/batch.rs:116-120` + the rebuild loop in `engine_core/src/game.rs:419-453`) — gate the rebuild on a world change flag once ecs GPP-04's change tracking exists. Related: ARCH-007 below.

## Open Items

### [DRY-006] Camera buffer/layout duplicated between sprite and line pipelines — Low
- **Files:** `sprite/pipeline.rs`, `line_pipeline.rs` — identical camera bind-group layout, camera uploaded twice per frame.
- **Fix:** shared `CameraBinding { buffer, layout, bind_group }` owned by `Renderer`.

### [ARCH-006] Cross-batch transparency vs. depth writes — Low
- **File:** `sprite/pipeline.rs` — `depth_write_enabled: true` + alpha blending; an early alpha-blended sprite can punch invisible holes into later batches when depths interleave across textures.
- **Fix (if it bites):** globally depth-sorted instance list with batch breaks on texture change, or separate opaque/transparent passes.

### [ARCH-007] `prepare_sprites` allocates a scratch Vec per frame — Low
- **File:** `sprite/pipeline.rs` — all batch instances copied into a fresh `Vec` each frame before upload.
- **Fix:** reusable scratch buffer on `SpritePipeline`, or per-batch uploads at computed offsets.

## Deferred by design
- **Mipmap generation:** no `generate_mipmaps` flag on purpose (the old one allocated a mip chain and never filled it). Re-add only together with real mip generation.
- **`RendererConfig` scope:** currently only `vsync`; extend (power preference / MSAA / bloom downsample) when a game needs it.

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 70 (100% pass, headless) |
| `#[allow(dead_code)]` / non-test `unwrap()` / `unsafe` / clippy warnings | 0 |
| High priority open | 0 |
| Medium priority open | 1 (GPP-15) |
| Low priority open | 3 |
