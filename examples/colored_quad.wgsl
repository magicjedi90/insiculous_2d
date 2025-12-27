// Colored quad shader - renders a bright colored quad for visibility testing
// Uses hardcoded vertices to create a quad centered on screen

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Quad vertices (two triangles) in clip space
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-0.5,  0.5),  // Top-left
        vec2<f32>(-0.5, -0.5),  // Bottom-left
        vec2<f32>( 0.5,  0.5),  // Top-right
        vec2<f32>(-0.5, -0.5),  // Bottom-left
        vec2<f32>( 0.5, -0.5),  // Bottom-right
        vec2<f32>( 0.5,  0.5)   // Top-right
    );
    
    out.clip_position = vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Bright cyan color for maximum visibility
    // This should be very obvious against dark background
    return vec4<f32>(0.0, 1.0, 1.0, 1.0); // Cyan (R=0, G=1, B=1, A=1)
}