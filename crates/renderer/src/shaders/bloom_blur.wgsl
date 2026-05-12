// Separable Gaussian blur — 9-tap kernel.
//
// Run twice per iteration: once with direction = (1, 0), once with (0, 1).
// Uses linear-sample fewer-tap trick: by sampling between two texels with
// bilinear filtering we get the sum of both samples for free, so a 9-tap
// blur is implemented as 5 sample calls.

struct BlurParams {
    // Pixel size in UV space — typically (1/width, 1/height) of the source.
    texel_size: vec2<f32>,
    // (1, 0) for horizontal pass, (0, 1) for vertical pass.
    direction: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> params: BlurParams;
@group(0) @binding(1)
var src_tex: texture_2d<f32>;
@group(0) @binding(2)
var src_sampler: sampler;

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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Pre-computed offsets and weights for a 9-tap Gaussian collapsed to
    // 5 linear samples (sigma ~= 2.0).
    let offset1 = 1.3846153846;
    let offset2 = 3.2307692308;
    let weight0 = 0.2270270270;
    let weight1 = 0.3162162162;
    let weight2 = 0.0702702703;

    let step = params.direction * params.texel_size;

    var rgb = textureSample(src_tex, src_sampler, in.uv).rgb * weight0;
    rgb = rgb + textureSample(src_tex, src_sampler, in.uv + step * offset1).rgb * weight1;
    rgb = rgb + textureSample(src_tex, src_sampler, in.uv - step * offset1).rgb * weight1;
    rgb = rgb + textureSample(src_tex, src_sampler, in.uv + step * offset2).rgb * weight2;
    rgb = rgb + textureSample(src_tex, src_sampler, in.uv - step * offset2).rgb * weight2;

    return vec4<f32>(rgb, 1.0);
}
