# Renderer Crate — Agent Context

You are working in the rendering crate. WGPU 28.0.0 backend with instanced sprite rendering.

## Architecture
```
Renderer (WGPU device, queue, surface)
└── SpritePipeline (render pipeline, bind groups, buffers)
    ├── Vertex buffer (quad geometry)
    ├── Instance buffer (per-sprite transforms, colors, UVs)
    ├── Camera uniform buffer + bind group (cached)
    └── Texture bind groups (cached per handle)
```

## Rendering Flow
1. `RenderManager::begin_frame()` — acquire surface texture
2. Collect sprite batches from ECS (group by texture handle)
3. Upload instance data to GPU buffer
4. For each batch: bind texture, draw instanced quads
5. UI overlay: draw UI commands on top
6. `RenderManager::end_frame()` — present

## File Map
- `renderer.rs` — WGPU device/queue/surface lifecycle
- `sprite.rs` — SpritePipeline (main rendering, batching, draw calls)
- `sprite_data.rs` — GPU data structures (Vertex, SpriteInstance, CameraUniform)
- `texture.rs` — Texture loading, caching, bind group management
- `shader/sprite.wgsl` — WGSL vertex + fragment shader

## Key Guidelines
- Cache bind groups — never create per-frame
- Batch by texture to minimize bind group switches
- Instance buffer grows but never shrinks
- GPU tests marked `#[ignore]` — most tests use mock/stub objects

## Known Tech Debt
- SpritePipeline manages 13 resources in one struct (SRP violation)
- Dead code with `#[allow(dead_code)]` in sprite.rs, sprite_data.rs, texture.rs
- CameraUniform duplicated in common and renderer crates

## Testing
- 62 tests, run with `cargo test -p renderer`

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

| Our Concept | Godot Equivalent | File |
|-------------|-----------------|------|
| SpritePipeline batching | Canvas item rendering | `servers/rendering/renderer_canvas_cull.cpp` — `canvas_render_items` |
| Sprite component | Sprite2D | `scene/2d/sprite_2d.cpp` |
| Camera2D | Camera2D | `scene/2d/camera_2d.cpp` |
| sprite.wgsl | Canvas shader | `servers/rendering/renderer_rd/shaders/canvas.glsl` |
| Texture caching | Texture storage | `servers/rendering/storage/texture_storage.cpp` |

**Remember:** We use WGPU, not Vulkan/OpenGL. Study Godot's *batching design* and *draw order logic*, not its graphics API calls.
