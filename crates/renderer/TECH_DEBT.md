# Technical Debt: renderer — LIVE (open items only)

Last audited: June 2026 (July 2026: Game Programming Patterns audit).
Resolved history: root `log_archive.md` § renderer.

## Game Programming Patterns Audit (July 2026 — closed; history in `log_archive.md`)
No open items (GPP-15 and ARCH-007 resolved Jul 13 2026 — see `log_archive.md`).

## Open Items

### [DRY-006] Camera buffer/layout duplicated between sprite and line pipelines — Low
- **Files:** `sprite/pipeline.rs`, `line_pipeline.rs` — identical camera bind-group layout, camera uploaded twice per frame.
- **Fix:** shared `CameraBinding { buffer, layout, bind_group }` owned by `Renderer`.

### [ARCH-006] Cross-batch transparency vs. depth writes — Low
- **File:** `sprite/pipeline.rs` — `depth_write_enabled: true` + alpha blending; an early alpha-blended sprite can punch invisible holes into later batches when depths interleave across textures.
- **Fix (if it bites):** globally depth-sorted instance list with batch breaks on texture change, or separate opaque/transparent passes.

## Deferred by design
- **Mipmap generation:** no `generate_mipmaps` flag on purpose (the old one allocated a mip chain and never filled it). Re-add only together with real mip generation.
- **`RendererConfig` scope:** currently only `vsync`; extend (power preference / MSAA / bloom downsample) when a game needs it.

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 74 (100% pass, headless) |
| `#[allow(dead_code)]` / non-test `unwrap()` / `unsafe` / clippy warnings | 0 |
| High priority open | 0 |
| Medium priority open | 0 |
| Low priority open | 2 (DRY-006, ARCH-006) |
