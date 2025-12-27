// Debug shader - bypass all transforms, just output vertices directly in NDC

// Camera uniform (still bound but not used)
struct Camera {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

// Texture bindings (still bound but not used)
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
    @location(0) debug_color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // DEBUG: IGNORE ALL TRANSFORMS - just use the vertex position directly as NDC
    // This will tell us if the vertex data is reaching the shader correctly
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    
    // Color based on vertex position to show different vertices
    out.debug_color = vec4<f32>(vertex.position.x + 0.5, vertex.position.y + 0.5, 0.5, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Output the debug color to verify shader is running
    return in.debug_color;
}
