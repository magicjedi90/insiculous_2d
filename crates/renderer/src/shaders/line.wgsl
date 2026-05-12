// Line render shader. Writes to the HDR target so emissive lines bloom.

struct Camera {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) emissive: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) emissive: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Lines live in the background plane: z = 0.9 places them behind sprites
    // at depth 0 but in front of the cleared depth buffer.
    let world = vec4<f32>(in.position, 0.9, 1.0);
    out.clip_position = camera.view_projection * world;
    out.color = in.color;
    out.emissive = in.emissive;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Same emissive multiplier as sprites so bright-pass picks them up.
    let glow_factor = 1.0 + in.emissive * 4.0;
    return vec4<f32>(in.color.rgb * glow_factor, in.color.a);
}
