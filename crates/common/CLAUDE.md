# Common Crate — Agent Context

Shared types used across all crates. Its only dependencies are `glam`, `serde`,
`thiserror`, and `bytemuck` — anything added here must stay dependency-light,
headless, and GPU-free. (Vector/matrix math comes straight from `glam`; there
is no engine-owned math module.)

## File Map
- `color.rs` — `Color` (the engine-wide color type)
- `rect.rs` — `Rect` (axis-aligned bounds: hit tests, intersection/union, UI layout)
- `transform.rs` — `Transform2D` (position/rotation/scale + point transforms)
- `camera.rs` — `Camera` (2D camera) and `CameraUniform` (view/projection data
  uploaded by the renderer; defined here only, not duplicated in renderer)
- `sheet_grid.rs` — `SheetGrid`: row-major cell grid over a sprite sheet /
  tileset plus its cell-index → UV-region math. Constructors: `new(cols, rows)`,
  `from_cell_size` (pixel sizes, truncating — partial trailing cells excluded),
  `from_uv_size` (the compatibility constructor `ecs::Tilemap` uses — it stores
  the given UV size verbatim, which is what keeps non-reciprocal sizes like
  `0.3` bit-identical). `uv_rect` is unchecked (out-of-range indices pass
  through and the sampler clamps); `uv_rect_checked` is the guarded variant
- `hash.rs` — `hash_u32` / `hash_f32` deterministic hashing for frame-driven
  pseudo-random values
- `time.rs` — `Time` (delta/elapsed tracking)
- `macros.rs` — small boilerplate-reduction macros

Every type above is re-exported at the crate root; `Color`, `Transform2D`,
`Camera`, `Rect`, `Time`, and `SheetGrid` are also in `common::prelude`.

## Testing
- `cargo test -p common` — 39 tests (37 unit + 2 doc), 0 ignored
