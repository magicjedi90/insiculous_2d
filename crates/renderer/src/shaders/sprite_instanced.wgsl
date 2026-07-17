// Instanced sprite rendering shader with camera and emissive support.
// Writes to an HDR target (Rgba16Float). Bright pixels — driven by the
// per-instance `emissive` attribute — are picked up by the bloom pipeline.

// Camera uniform
struct Camera {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

// Texture bindings
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// Vertex attributes (per-vertex)
struct VertexInput {
    @location(0) position: vec3<f32>,  // Local quad vertex position
    @location(1) tex_coords: vec2<f32>, // Base texture coordinates
    @location(2) color: vec4<f32>,      // Vertex color
}

// Instance attributes (per-sprite)
struct InstanceInput {
    @location(3) world_position: vec2<f32>,  // Sprite position in world
    @location(4) rotation: f32,              // Sprite rotation in radians
    @location(5) scale: vec2<f32>,           // Sprite scale
    @location(6) tex_region: vec4<f32>,      // Texture region [u, v, width, height]
    @location(7) color: vec4<f32>,           // Color tint
    @location(8) depth: f32,                 // Depth (0 = near, 1 = far in NDC after camera)
    @location(9) emissive: f32,              // Emissive intensity
    @location(10) shape: vec4<f32>,          // SDF shape [kind, corner_radius, border_width, reserved]
}

// Output to fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) emissive: f32,
    @location(3) local_px: vec2<f32>,   // fragment position in local pixels (pre-rotation)
    @location(4) half_size: vec2<f32>,  // sprite half extents in local pixels
    @location(5) shape: vec4<f32>,      // SDF shape params (constant per instance)
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);

    // Counter-clockwise rotation for positive angles (world convention —
    // matches rapier bodies, glam, and the editor's collider overlay).
    // WGSL mat3x3 takes COLUMNS: x' = cos*x - sin*y, y' = sin*x + cos*y.
    let rot_matrix = mat3x3<f32>(
        vec3<f32>(cos_r,  sin_r, 0.0),
        vec3<f32>(-sin_r, cos_r, 0.0),
        vec3<f32>(0.0,    0.0,   1.0)
    );

    let scale_matrix = mat3x3<f32>(
        vec3<f32>(instance.scale.x, 0.0, 0.0),
        vec3<f32>(0.0, instance.scale.y, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );

    let transform_matrix = rot_matrix * scale_matrix;

    let local_pos = transform_matrix * vec3<f32>(vertex.position.xy, 0.0);
    let world_pos = vec4<f32>(local_pos.xy + instance.world_position, instance.depth, 1.0);

    out.clip_position = camera.view_projection * world_pos;

    let base_uv = vertex.tex_coords;
    out.tex_coords = vec2<f32>(
        instance.tex_region.x + base_uv.x * instance.tex_region.z,
        instance.tex_region.y + base_uv.y * instance.tex_region.w
    );

    out.color = vertex.color * instance.color;
    out.emissive = instance.emissive;

    // Quad vertices span ±0.5, so local pixels = vertex * scale, half
    // extents = scale/2. Rotation is irrelevant for the SDF — it operates
    // in the sprite's own (pre-rotation) space.
    out.local_px = vertex.position.xy * instance.scale;
    out.half_size = abs(instance.scale) * 0.5;
    out.shape = instance.shape;

    return out;
}

// Signed distance from `p` to a rounded box of half extents `b`, radius `r`.
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - (b - vec2<f32>(r, r));
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let base_rgb = tex_color.rgb * in.color.rgb;
    // emissive multiplies RGB so values exceed 1.0 and the bright-pass picks them up.
    // 1.0 + 4.0*intensity scales linearly without a hard threshold.
    let glow_factor = 1.0 + in.emissive * 4.0;
    let out_rgb = base_rgb * glow_factor;
    var alpha = tex_color.a * in.color.a;

    // SDF shape mask: kind 0 = plain quad (no mask), 1 = rounded rect,
    // 2 = circle. Distances are in local pixels, so the ~1.5px smoothstep
    // anti-aliasing band is zoom-independent on screen-space UI.
    let kind = in.shape.x;
    if (kind > 0.5) {
        let min_half = min(in.half_size.x, in.half_size.y);
        var radius = clamp(in.shape.y, 0.0, min_half);
        if (kind > 1.5) {
            radius = min_half; // circle: fully rounded
        }
        var d = sd_rounded_box(in.local_px, in.half_size, radius);
        let border = in.shape.z;
        if (border > 0.0) {
            // Keep only a ring of `border` thickness inside the outer edge
            d = abs(d + border * 0.5) - border * 0.5;
        }
        alpha = alpha * (1.0 - smoothstep(-0.75, 0.75, d));
    }

    return vec4<f32>(out_rgb, alpha);
}
