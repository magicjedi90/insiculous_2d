// Final composite: HDR + blurred bloom -> sRGB swapchain.
//
// The swapchain is sRGB, so the GPU performs the linear -> sRGB encoding
// automatically. We just need to tonemap from HDR (values potentially >> 1)
// down to [0, 1] before writing.

struct CompositeParams {
    threshold: f32,    // unused here, kept for layout parity
    knee: f32,         // unused
    // Bloom contribution multiplier. 0 disables bloom entirely.
    intensity: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<uniform> params: CompositeParams;
@group(0) @binding(1)
var hdr_tex: texture_2d<f32>;
@group(0) @binding(2)
var bloom_tex: texture_2d<f32>;
@group(0) @binding(3)
var linear_sampler: sampler;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var out: VsOut;
    let x = f32((idx << 1u) & 2u);
    let y = f32(idx & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

// Reinhard tonemap — simple and stable. Doesn't crush highlights as hard
// as ACES, which is what we want for a neon Geometry-Wars look.
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    return c / (vec3<f32>(1.0) + c);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scene = textureSample(hdr_tex, linear_sampler, in.uv).rgb;
    let bloom = textureSample(bloom_tex, linear_sampler, in.uv).rgb;
    let combined = scene + bloom * params.intensity;
    let mapped = tonemap(combined);
    return vec4<f32>(mapped, 1.0);
}
