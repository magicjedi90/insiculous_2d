// Bright-pass: samples the HDR target, keeps pixels whose luminance exceeds a
// threshold, and downsamples to the half-resolution bloom buffer in one go.

struct BloomParams {
    // Luminance threshold. Pixels below this are zeroed out.
    threshold: f32,
    // Soft knee — values between (threshold - knee) and threshold ramp in
    // gradually rather than clipping, hiding banding around the cutoff.
    knee: f32,
    // Unused composite-side knobs; padded so the struct matches the layout
    // used by every pass.
    intensity: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<uniform> params: BloomParams;
@group(0) @binding(1)
var src_tex: texture_2d<f32>;
@group(0) @binding(2)
var src_sampler: sampler;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Full-screen triangle covering NDC — no vertex buffer needed.
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var out: VsOut;
    let x = f32((idx << 1u) & 2u);
    let y = f32(idx & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let color = textureSample(src_tex, src_sampler, in.uv).rgb;
    let l = luminance(color);

    // Soft knee from "Next Generation Post Processing in Call of Duty: Advanced Warfare".
    // Smooths the threshold so we don't get a hard edge.
    let knee = max(params.knee, 0.0001);
    let soft = clamp(l - params.threshold + knee, 0.0, 2.0 * knee);
    let soft_curve = soft * soft / (4.0 * knee);
    let contribution = max(l - params.threshold, soft_curve) / max(l, 0.0001);

    return vec4<f32>(color * contribution, 1.0);
}
