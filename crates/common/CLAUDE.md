# Common Crate — Agent Context

Shared types and math utilities used across all crates.

## Contents
- `math.rs` — Vec2, Vec3, Vec4, Mat4, mathematical constants
- `CameraUniform` — camera view/projection data (NOTE: duplicated in renderer crate — tech debt)

## Testing
- 27 tests (26 unit + 1 doc), run with `cargo test -p common`
