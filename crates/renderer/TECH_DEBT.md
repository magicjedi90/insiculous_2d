# Technical Debt: renderer

Last audited: June 2026 (full-crate audit + fix pass)

## Summary

The June 2026 audit fixed one rendering bug, two runtime panic hazards,
removed ~700 lines of dead code, eliminated all per-frame allocation churn it
found, and split `sprite.rs` (1,059 lines) into focused modules. Every file is
now under 600 lines, there are no `#[allow(dead_code)]` suppressions, no
`unwrap()` outside tests, and `cargo clippy -p renderer` is clean.

- Open issues: 3 (all Low)
- High/Medium priority issues: 0

---

## Open Issues

### [DRY-006] Camera buffer/layout duplicated between sprite and line pipelines
- **Files:** `sprite/pipeline.rs`, `line_pipeline.rs`
- **Issue:** Both pipelines declare an identical camera bind-group layout and
  each owns its own camera uniform buffer; the camera is uploaded twice per
  frame (once per pipeline).
- **Suggested fix:** Extract a shared `CameraBinding { buffer, layout,
  bind_group }` owned by `Renderer` and passed to both pipelines.
- **Priority:** Low (one redundant 80-byte upload per frame)

### [ARCH-006] Cross-batch transparency vs. depth writes
- **File:** `sprite/pipeline.rs`
- **Issue:** Sprites render with `depth_write_enabled: true` plus alpha
  blending. Batch submission order is deterministic (engine_core sorts by min
  depth, then texture handle), but an alpha-blended sprite drawn early can
  still punch invisible holes into sprites behind it from a later batch when
  depths interleave across textures.
- **Suggested fix (if it bites):** a globally depth-sorted instance list with
  batch breaks on texture change, or separate opaque/transparent passes.
- **Priority:** Low (not observable with current games' depth usage)

### [ARCH-007] `prepare_sprites` allocates a scratch Vec per frame
- **File:** `sprite/pipeline.rs`
- **Issue:** All batch instances are copied into a fresh `Vec` each frame
  before upload.
- **Suggested fix:** Keep a reusable scratch buffer on `SpritePipeline`, or
  upload per-batch at computed offsets.
- **Priority:** Low

### Deferred by design

- **Mipmap generation:** `TextureLoadConfig` deliberately has no
  `generate_mipmaps` flag. The old flag allocated a mip chain but never filled
  it, so minified sprites sampled uninitialized levels. Re-add only together
  with real mip generation (CPU downsample per level, or a blit chain).
- **`RendererConfig` scope:** currently only `vsync`. Extend with power
  preference / MSAA / bloom downsample factor when a game actually needs them.

---

## Resolved — June 2026 Audit

| Issue | Resolution |
|-------|------------|
| **Bloom blur was vertical-only** | `queue.write_buffer` flushes at submit, so rewriting one shared blur-params buffer between passes made every pass read the last write (`[0,1]`). Split into per-direction uniform buffers. |
| **Sprite overflow panicked at 1,001 sprites** | `DynamicBuffer::update` now grows the GPU buffer (next power of two) instead of `panic!`. Line pipeline's silent truncation replaced by the same growth path. |
| **NaN depth panicked the batch sort** | `sort_by_depth` uses `f32::total_cmp` (NaN sorts last, deterministic). Regression test added. |
| **Bloom created 2 + 2×iterations bind groups per frame** | Bind groups cached per render-target size, rebuilt on resize. Blur texel params now also written only on resize. |
| **Texture map deep-cloned twice per frame** | `render_with_sprites` no longer clones the map to splice in the white texture (cached via `cache_texture_bind_group`); `game.rs` uses `assets.textures()` by reference; `textures_cloned()` removed. |
| **`render_batcher` cloned every batch per frame** | Collects `&SpriteBatch` refs; also sorts deterministically (min depth, then handle). |
| **Magic white-texture handle `{ id: 0 }`** | `TextureHandle::WHITE` const; `TextureManager` allocation comment references it. |
| **`generate_mipmaps` produced broken textures** | Flag removed (allocated mips were never filled). |
| **`render_pipeline_inspector.rs` (434 lines)** | Deleted — was never declared in `lib.rs`, i.e. not even compiled. |
| **`shaders/sprite.wgsl`** | Deleted — only `sprite_instanced.wgsl` is referenced. |
| **`TextureResource::create_solid_color` stub** | Deleted — created textures without uploading pixel data, logged success, had zero callers. |
| **Legacy `Renderer::render()` + `render_basic()` + `run_with_app`** | Deleted — empty render pass and event-loop helper had no callers; `Game` trait / `run_game()` is the only path. |
| **4 unused `RendererError` variants** | `SwapChainCreationError`, `PipelineCreationError`, `RuntimeCreationError`, `AssetLoadingError` removed. |
| **6 `#[allow(dead_code)]` suppressions** | All removed: fields deleted (`SpriteBatcher.max_sprites_per_batch`, `SpritePipeline.layout`/`.sampler`) or became used (`DynamicBuffer.usage`, `LinePipeline.device`). |
| **Empty placeholder test** | `test_texture_resource_dimensions` (zero assertions) deleted. |
| **`sprite.rs` at 1,059 lines (limit 600)** | Split: `sprite.rs` (Sprite), `sprite/batch.rs` (SpriteBatch/SpriteBatcher), `sprite/pipeline.rs` (SpritePipeline). Atlas types colocated in new `atlas.rs` (from sprite.rs + texture.rs). |
| **No renderer configuration** | `RendererConfig { vsync }` added; wired through `GameConfig::with_vsync()` → `RenderManager::init`. |
| **Nondeterministic cross-batch draw order** | Already deterministic in `game.rs::sort_batches`; `render_batcher` now sorts the same way. |
| **line_pipeline doc claimed shared camera layout** | Doc corrected (each pipeline owns its camera binding; sharing tracked as DRY-006). |

## Resolved — January/February 2026 (prior audits)

| Issue | Resolution |
|-------|------------|
| Bind groups created every frame (sprites) | Camera bind group cached; texture bind groups cached per handle |
| DRY-001: duplicate surface error handling | `acquire_frame()` helper |
| DRY-002: duplicate sampler creation | `SamplerConfig::create_sampler()` |
| KISS-002: unsafe transmute for surface lifetime | WGPU 28 infers `'static` from `Arc<Window>` |
| ARCH-002: `Time` struct misplaced | Moved to `common` crate |
| ARCH-004: inconsistent error types | `From<TextureError> for RendererError` |
| SRP-002: Renderer init + rendering coupled | Documented as intentional (WGPU lifetimes) |

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 13 (+5 shaders) |
| Total lines | ~3,800 |
| Largest file | `texture.rs` (512 lines) |
| Tests | 69 (100% pass, headless) |
| `#[allow(dead_code)]` | 0 |
| `unwrap()` outside tests | 0 |
| `unsafe` blocks | 0 |
| Clippy warnings | 0 |
