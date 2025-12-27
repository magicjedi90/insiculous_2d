# Vertex Scale Tests - Coordinate Scaling Investigation

This directory contains comprehensive tests to determine if vertex position magnitude affects sprite visibility in the Insiculous 2D engine.

## Background

The hypothesis is that sprites may not be visible because the default vertex positions (±0.5 units) are too small relative to the camera projection scale, causing them to be clipped or lost in floating-point precision during transformation.

## Test Files

### 1. `vertex_scale_test.rs` - Basic Comparison Test
**Purpose**: Compares different vertex scale simulations side-by-side
**Method**: Creates multiple sprites with different combinations of simulated vertex scales and sprite scales
**Key Features**:
- Tests vertex scales: ±0.5, ±50, ±200, ±400
- Maintains constant world-space size (400x400 units) across all tests
- Uses contrasting colors for visibility
- Logs detailed vertex position information

**Run**: `cargo run --example vertex_scale_test`

### 2. `vertex_scale_test_advanced.rs` - Custom Pipeline Test
**Purpose**: Tests actual custom vertex buffers with different scales
**Method**: Creates custom WGPU pipelines with different vertex position magnitudes
**Key Features**:
- Creates actual vertex buffers with specified position scales
- Custom render pipeline for each vertex scale
- Tests: ±0.5, ±50, ±200, ±400 vertex positions
- Advanced logging of GPU buffer contents

**Run**: `cargo run --example vertex_scale_test_advanced`

### 3. `vertex_scale_diagnostic.rs` - Simulation Test
**Purpose**: Simulates different vertex scales using sprite transformations
**Method**: Uses the standard pipeline but creates sprites that simulate different vertex scales
**Key Features**:
- Tests the hypothesis without modifying the pipeline
- All tests produce the same final world-space size
- Comprehensive logging of expected vs actual results
- Camera visibility analysis

**Run**: `cargo run --example vertex_scale_diagnostic`

### 4. `vertex_scale_definitive.rs` - Custom Buffer Test
**Purpose**: Creates actual custom vertex buffers to definitively test the hypothesis
**Method**: Builds custom sprite pipelines with different vertex position scales
**Key Features**:
- Actual vertex buffer modification
- Tests extreme vertex scales: ±0.1, ±0.5, ±50, ±200, ±400
- Detailed vertex position logging
- Pipeline-specific vertex buffer creation

**Run**: `cargo run --example vertex_scale_definitive`

## Expected Results

### If Vertex Position Magnitude is the Issue:
- **Visible**: Tests with large vertex positions (±50, ±200, ±400)
- **Invisible**: Tests with small vertex positions (±0.5, ±0.1)

### If Sprite Scale is the Issue:
- **All tests visible**: Since all tests produce the same final world-space size

### If Camera/Projection is the Issue:
- **Inconsistent results**: Some tests visible depending on camera position and scale

## Test Methodology

1. **Control Variables**: All tests maintain constant final world-space dimensions
2. **Isolation**: Each test isolates vertex scale as the primary variable
3. **Visibility**: Uses bright, contrasting colors for clear visibility
4. **Logging**: Comprehensive logging of vertex positions, transformations, and camera parameters
5. **Camera Analysis**: Checks if sprites would be within camera view frustum

## How to Interpret Results

### Red Test (Standard ±0.5 vertices, 400x400 scale)
- **Visible**: Vertex scale is NOT the issue
- **Invisible**: Vertex scale MAY be the issue

### Green Test (Simulated ±200 vertices, 1x1 scale)
- **Visible**: Supports vertex scale hypothesis
- **Invisible**: Suggests other issues (sprite scale, camera, etc.)

### Blue Test (Simulated ±400 vertices, 0.5x0.5 scale)
- **Visible**: Strong evidence for vertex scale hypothesis
- **Invisible**: Contradicts vertex scale hypothesis

### Yellow Test (Simulated ±50 vertices, 4x4 scale)
- **Visible**: Supports vertex scale hypothesis
- **Invisible**: Suggests threshold effect or other issues

## Running All Tests

1. **Individual Tests**: Run each example separately to isolate results
2. **Comparison**: Compare visibility across different vertex scales
3. **Logging**: Check console output for detailed vertex position information
4. **Camera**: Note camera position, viewport size, and projection parameters

## Next Steps Based on Results

### If Vertex Scale is Confirmed as Issue:
1. Modify `SpritePipeline::new()` to use larger default vertex positions
2. Update vertex shader to handle different position scales
3. Add vertex scale configuration to pipeline creation
4. Test with various camera settings and projections

### If Vertex Scale is NOT the Issue:
1. Investigate sprite instance data upload
2. Check shader compatibility and attribute binding
3. Verify camera projection matrix calculation
4. Test texture coordinate and color attribute passing

### If Results are Inconclusive:
1. Test with different camera positions and zoom levels
2. Verify GPU buffer uploads and binding
3. Check for shader compilation errors
4. Test with simpler geometry (single triangle)

## Technical Details

### Current Vertex Buffer (Standard Pipeline):
```rust
let vertices = [
    SpriteVertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec2::new(0.0, 0.0), Vec4::ONE),
    SpriteVertex::new(Vec3::new(0.5, 0.5, 0.0), Vec2::new(1.0, 0.0), Vec4::ONE),
    SpriteVertex::new(Vec3::new(0.5, -0.5, 0.0), Vec2::new(1.0, 1.0), Vec4::ONE),
    SpriteVertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec2::new(0.0, 1.0), Vec4::ONE),
];
```

### Camera Projection (Orthographic):
```rust
Mat4::orthographic_rh(
    -half_width,    // -400 (for 800x600 viewport)
    half_width,     // +400
    -half_height,   // -300
    half_height,    // +300
    near,           // -1000
    far,            // +1000
)
```

### Transformation Pipeline:
1. Vertex positions (±0.5) multiplied by sprite scale
2. Transformed by sprite rotation matrix
3. Translated to sprite position
4. Transformed by camera view matrix
5. Projected to screen space via orthographic projection

## Conclusion

These tests provide a comprehensive approach to determining if vertex position magnitude is the root cause of sprite visibility issues. By systematically testing different vertex scales while controlling other variables, we can definitively identify the source of the problem and implement appropriate fixes.