// Instanced sprite rendering shader with camera support

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
    @location(8) depth: f32,                 // Depth for sorting
}

// Output to fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Vertex shader
@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Build sprite transformation matrix
    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);
    
    // Rotation matrix
    let rot_matrix = mat3x3<f32>(
        vec3<f32>(cos_r, -sin_r, 0.0),
        vec3<f32>(sin_r,  cos_r, 0.0),
        vec3<f32>(0.0,    0.0,   1.0)
    );
    
    // Scale matrix
    let scale_matrix = mat3x3<f32>(
        vec3<f32>(instance.scale.x, 0.0, 0.0),
        vec3<f32>(0.0, instance.scale.y, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );
    
    // Combine rotation and scale
    let transform_matrix = rot_matrix * scale_matrix;
    
    // Transform vertex position
    let local_pos = transform_matrix * vec3<f32>(vertex.position.xy, 0.0);
    let world_pos = vec4<f32>(local_pos.xy + instance.world_position, instance.depth, 1.0);
    
    // Transform by camera view-projection matrix
    out.clip_position = camera.view_projection * world_pos;
    
    // Calculate texture coordinates based on texture region
    let base_uv = vertex.tex_coords;
    out.tex_coords = vec2<f32>(
        instance.tex_region.x + base_uv.x * instance.tex_region.z,
        instance.tex_region.y + base_uv.y * instance.tex_region.w
    );
    
    // Combine vertex color with instance color
    out.color = vertex.color * instance.color;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // For colored sprites without textures, we want to use the vertex color directly
    // If texture is white (1,1,1,1), this gives us the vertex color
    // If texture has alpha, we preserve it
    return vec4<f32>(tex_color.rgb * in.color.rgb, tex_color.a * in.color.a);
}