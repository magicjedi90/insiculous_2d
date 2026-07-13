# Renderer Crate — Agent Context

You are working in the rendering crate. WGPU 28.0.0 backend with instanced sprite rendering, HDR + bloom post-processing.

## Architecture
```
Renderer (WGPU device, queue, surface, RendererConfig{vsync})
├── RenderTargets (HDR color + depth + bloom ping/pong, rebuilt on resize)
├── SpritePipeline (instanced quads -> HDR target)
│   ├── Vertex/index buffers (quad geometry)
│   ├── Instance buffer (DynamicBuffer — grows on demand, never panics)
│   ├── InstanceCache — skips the instance upload when nothing changed (GPP-15)
│   ├── Camera uniform buffer + bind group (cached)
│   └── Texture bind groups (cached per handle; TextureHandle::WHITE = built-in 1x1 white)
├── LinePipeline (line-list geometry -> HDR target, e.g. spring-mass grid)
└── BloomPipeline (extract -> H/V blur ping-pong -> composite to swapchain)
    └── Bind groups cached per target size; per-direction blur uniform buffers
```

## Rendering Flow (one frame)
1. Sprites + lines draw into the HDR target (Rgba16Float) with depth
2. Bloom extracts bright pixels (half-res), blurs H+V × iterations, composites to the sRGB swapchain
3. Camera uniforms uploaded once per pipeline per frame

## File Map
- `renderer.rs` — WGPU device/queue/surface lifecycle, `RendererConfig`, frame orchestration
- `sprite.rs` — `Sprite` data type; parent of the sprite submodules
- `sprite/batch.rs` — `SpriteBatch`, `SpriteBatcher` (CPU-side grouping by texture)
- `sprite/pipeline.rs` — `SpritePipeline` (GPU pipeline, bind group caches, draw)
- `sprite_data.rs` — GPU data structures (`SpriteVertex`, `SpriteInstance`, `DynamicBuffer`)
- `texture.rs` — `TextureManager`, `TextureHandle` (incl. `WHITE`), `SamplerConfig`
- `atlas.rs` — `TextureAtlas`, `TextureAtlasBuilder`, `AtlasRegion`
- `render_targets.rs` — HDR/depth/bloom textures, resize handling
- `bloom.rs` — bloom passes + `BloomConfig` (runtime-tunable)
- `line_pipeline.rs` — `LinePipeline`, `LineVertex`
- `shaders/` — `sprite_instanced.wgsl`, `line.wgsl`, `bloom_{extract,blur,composite}.wgsl`

## Key Guidelines
- **Cache bind groups — never create per-frame.** Sprite textures cache per handle; bloom caches per target size.
- **`queue.write_buffer` flushes at submit, not encode.** Never rewrite one uniform buffer between passes in the same submit — every pass sees only the last write. Use one buffer per distinct value (see bloom's H/V blur buffers).
- Batch by texture to minimize bind group switches; cross-batch submission order must be deterministic (callers sort by min depth, then handle)
- `DynamicBuffer` grows (next power of two) and never shrinks; pass `&Device` to `update`
- Float sorts use `total_cmp` — no `partial_cmp().unwrap()`
- All tests run headless (GPU-dependent doc examples are compile-only `no_run`)

## Known Tech Debt
See `TECH_DEBT.md` — 2 open issues, both Low (shared camera binding, cross-batch transparency vs depth writes).

## Testing
- 74 tests (73 unit + 1 compile-only doc), run with `cargo test -p renderer`

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

| Our Concept | Godot Equivalent | File |
|-------------|-----------------|------|
| SpritePipeline batching | Canvas item rendering | `servers/rendering/renderer_canvas_cull.cpp` — `canvas_render_items` |
| Sprite component | Sprite2D | `scene/2d/sprite_2d.cpp` |
| Camera2D | Camera2D | `scene/2d/camera_2d.cpp` |
| sprite_instanced.wgsl | Canvas shader | `servers/rendering/renderer_rd/shaders/canvas.glsl` |
| Texture caching | Texture storage | `servers/rendering/storage/texture_storage.cpp` |
| Bloom | Glow effect | `servers/rendering/renderer_rd/effects/copy_effects.cpp` |

**Remember:** We use WGPU, not Vulkan/OpenGL. Study Godot's *batching design* and *draw order logic*, not its graphics API calls.
