# Renderer Analysis

## Audit Note (April 15, 2026)

This file was audited against the actual source in `crates/renderer/src/`. Items
removed because the underlying code has changed or the note no longer
load-bearing:
- "Bind groups created every frame" issue — camera bind group and texture bind
  groups are now cached (`SpritePipeline::camera_bind_group` and
  `texture_bind_group_cache`)
- "Previously Fixed Issues" section covering ECS texture handle integration and
  early 2026 sprite rendering fixes (ancient history)
- Test-coverage enumeration and the test-plan template (inventory churns with
  every test addition; the live count lives in `cargo test -p renderer` output)
- Production-readiness scoring that duplicated information in `AGENTS.md` /
  `PROJECT_ROADMAP.md`
- The "Review (January 19, 2026)" preamble (superseded by Current State)

Items kept because they still reflect real code or capture design rationale
useful for future work: SRP tension in `SpritePipeline`, duplicated
device/queue accessors, remaining `#[allow(dead_code)]` suppressions, glyph
texture cache memory issue (cross-crate with UI), architectural overview,
design-decision notes.

---

## Current State (Updated: April 2026)

WGPU 28.0.0 backend with instanced sprite rendering. 62 unit tests, 1 ignored
doctest. Sprite visibility is enforced upstream in the ECS `Sprite` component
(see `ecs/sprite_components.rs` — `visible: bool`); the renderer receives only
visible sprites in its batches.

**File sizes (April 2026):**

| File | Lines | Notes |
|------|-------|-------|
| `sprite.rs` | 1021 | Exceeds the project's 600-line guideline — candidate for splitting (see "SRP Violation" below) |
| `texture.rs` | 696 | Also over 600 lines |
| `sprite_data.rs` | 502 | GPU data structures + `DynamicBuffer` |
| `render_pipeline_inspector.rs` | 434 | Diagnostic/telemetry helper — see note below |
| `renderer.rs` | 428 | Device/queue/surface lifecycle |

---

## Architecture

```
Renderer (renderer.rs)
├── Surface Management (WGPU 28.0.0)
├── Device and Queue (Arc-wrapped for sharing)
└── White Texture Resource (built-in 1x1 white for colored sprites)

SpritePipeline (sprite.rs)
├── Render Pipeline (shader + vertex layout)
├── Vertex Buffer (quad geometry)
├── Instance Buffer (DynamicBuffer<SpriteInstance>)
├── Index Buffer (quad indices, 6 u16)
├── Camera Uniform Buffer
├── Cached Camera Bind Group      ← created once, buffer updated via write_buffer
├── Cached Texture Bind Groups    (HashMap<TextureHandle, BindGroup>)
├── Bind Group Layouts (camera + texture)
└── Sampler (default, currently unused after bind-group-cache refactor)

SpriteBatcher (sprite.rs)
├── Texture-based Grouping (HashMap<TextureHandle, SpriteBatch>)
├── Depth Sorting (per-batch)
└── Instance Collection

TextureManager (texture.rs)
├── Handle Registry
├── Texture Cache
└── Atlas Builder
```

### Design Decisions

**`Renderer` bundles init + render** (`renderer.rs` module docs). WGPU's
surface, device, and queue share lifetimes, so splitting init and render into
separate structs would add complexity without clear benefit for a 2D engine.
Documented intentionally in the module header.

**Bind groups cached, buffers updated.** Camera uniform uses
`queue.write_buffer` rather than recreating the bind group each frame. Texture
bind groups are created lazily on first use per handle via
`cache_texture_bind_groups` and reused thereafter. Cache invalidation is
manual — call `invalidate_texture_cache(handle)` when a texture is unloaded.

**Surface format hardcoded to `Bgra8UnormSrgb`** in the sprite pipeline's color
target (`sprite.rs` around line 391) even though the surface picks
`surface_caps.formats[0]` in `Renderer::new`. If a platform reports a different
preferred format the pipeline will mismatch. Low-priority: works on every
platform we've tested so far.

---

## Known Issues

### SRP Violation: SpritePipeline

`SpritePipeline` still holds ~13 fields (pipeline, layout, vertex/instance/index
buffers, camera buffer + bind group + layout, texture bind group layout +
cache, sampler, max_sprites_per_batch, Arc<Device>). `sprite.rs` is 1021 lines
— above the project's 600-line guideline.

Suggested split when this crate is next revisited:
- `PipelineResources` — render pipeline, pipeline layout, bind group layouts
- `BufferManager` — vertex, instance, index buffers
- `CameraManager` — camera uniform buffer + cached bind group
- `TextureBindCache` — the `HashMap<TextureHandle, BindGroup>` + layout

Pull `SpriteBatcher` / `SpriteBatch` / `Sprite` and `TextureAtlas` into their
own modules while at it.

### Redundant Device/Queue Accessors

`Renderer` exposes both `device()` (returns `Arc<Device>`) and `device_ref()`
(returns `&Device`), likewise for the queue. Doc comments disambiguate, but
callers still guess. Consolidation candidate: keep only `&Device` / `&Queue`
and require callers to `Arc::clone` explicitly when they need ownership.

### `#[allow(dead_code)]` Suppressions

Current suppressions (not all truly dead — some are retention for potential
future use):

| File:line | Field | Justification in code |
|-----------|-------|-----------------------|
| `sprite.rs:164` | `SpriteBatcher::max_sprites_per_batch` | "Reserved for future batch splitting optimization" |
| `sprite.rs:228` | `SpritePipeline::layout` (pipeline layout) | "Keep for potential pipeline recreation" |
| `sprite.rs:247` | `SpritePipeline::sampler` | "Kept for potential future use (e.g., default sampler fallback)" |
| `sprite_data.rs:212` | `DynamicBuffer::usage` | "Stored for potential buffer recreation" |
| `texture.rs:386` | `create_placeholder_texture` | "Reserved for future error handling" |

Either wire them in (batch splitting when `sprite_count > max_sprites_per_batch`,
placeholder texture on load failure) or delete. The retention-for-maybe comments
accumulate cruft over time.

### Resource Cleanup

No explicit GPU resource destruction on renderer shutdown. WGPU resources are
Arc-wrapped and will drop, but we have no test that a full teardown is
leak-free. Low priority — headless tests don't exercise this path.

### Glyph Texture Cache Wastes Memory (Cross-crate)

The glyph-texture cache key (upstream in the UI→renderer bridge — see
`engine_core/ui_integration.rs`) includes color. Rasterizing glyphs is
color-independent; only the final sprite tint should carry color. The same
glyph in a different color allocates a fresh texture atlas entry. Owner: UI
integration, not renderer proper, but it inflates our texture resource table.
Tracked here because the waste shows up in the renderer's texture cache.

### `render_pipeline_inspector.rs` — Diagnostic Module

A 434-line diagnostic logger (operation types, surface/encoder/draw-call
instrumentation) that is not re-exported from `lib.rs` prelude and not used by
the main render path. Either:
- Promote it to an opt-in `renderer::diagnostic` module with tests and docs, or
- Archive it — dead code is worse than no code.

---

## Future Enhancements (Not Blocking)

- GPU resource cleanup on drop, with a teardown test
- Async texture loading (currently synchronous, blocks main thread)
- Automatic surface format detection (drop the hardcoded `Bgra8UnormSrgb`)
- Frustum culling for large scenes
- Text rendering as first-class (currently goes through UI glyph raster →
  sprite path)
- Post-processing pipeline (bloom, chromatic aberration, etc.)
- Integration test harness that validates end-to-end rendering without a real
  GPU (software rasterizer backend, or golden-image tests on CI with a virtual
  display)

---

## Cross-Crate Interactions

- **`engine_core/render_manager.rs`** owns the `Renderer` + `SpritePipeline`
  lifetime. The `RenderManager::render()` wiring calls into
  `render_with_sprites`.
- **`engine_core/ui_integration.rs`** builds the glyph texture cache and
  translates UI draw commands into sprite batches. Glyph-texture cache key
  lives here (see note above).
- **`ecs/sprite_components.rs`** defines `Sprite`, `Transform2D`, `Camera`.
  The `visible` flag on `Sprite` is filtered upstream — the renderer does not
  see invisible sprites.
- **`common::Time`** is the shared timing type; renderer does not maintain its
  own.

---

## Usage

```rust
// Create pipeline
let sprite_pipeline = SpritePipeline::new(renderer.device_ref(), 1000);

// Batch sprites
let mut batcher = SpriteBatcher::new(1000);
batcher.add_sprite(&sprite);

// Collect batches (owned clone to decouple from batcher mutability)
let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();

// Render (instance buffer upload happens inside render_with_sprites)
renderer.render_with_sprites(&mut sprite_pipeline, &camera, &textures, &batch_refs)?;
```

Run `cargo run --example hello_world` to see working sprite rendering.
