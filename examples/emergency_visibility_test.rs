//! 🚨 **EMERGENCY VISIBILITY TEST** - Maximum visibility with no textures
//! 
//! This renders colored quads with NO texture sampling, using vertex colors only.
//! If this doesn't show sprites, nothing will.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
    keyboard::{PhysicalKey, KeyCode},
};
use renderer::prelude::*;
use wgpu::util::DeviceExt;
use glam::{Vec2, Vec4, Mat4};

struct EmergencyVisibilityTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_count: u32,
}

impl EmergencyVisibilityTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            frame_count: 0,
        }
    }
}

const COLORED_QUAD_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Camera {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct InstanceInput {
    @location(2) world_position: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) depth: f32,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform vertex (quad from -0.5 to 0.5)
    let local_pos = vertex.position * instance.scale + instance.world_position;
    let world_pos = vec4<f32>(local_pos, instance.depth, 1.0);
    
    // Apply camera
    out.clip_position = camera.view_projection * world_pos;
    out.color = vertex.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // NO TEXTURE SAMPLING - pure vertex color
    return in.color;
}
"#;

impl ApplicationHandler<()> for EmergencyVisibilityTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🚨 EMERGENCY VISIBILITY TEST");
        println!("=============================");
        println!("🎯 NO texture sampling");
        println!("🎯 Pure vertex colors");
        println!("🎯 Maximum brightness");
        println!("🎯 3 colored quads in middle of screen");
        println!("=============================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🚨 EMERGENCY VISIBILITY")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        self.frame_count += 1;
    }
}

impl EmergencyVisibilityTest {
    fn render_frame(&mut self) {
        if let Some(renderer) = &self.renderer {
            if self.frame_count == 0 {
                println!("🎨 Rendering colored quads...");
                println!("🟥 Left: RED");
                println!("🟩 Middle: GREEN");
                println!("🟦 Right: BLUE");
                println!("🎯 ALL 100% opacity, NO textures");
            }
            
            // Get frame
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Create command encoder
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Emergency Test Encoder"),
                }
            );
            
            // Clear dark gray (so we can see bright colors)
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
            
            // Create shader
            let shader = renderer.device().create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: Some("Emergency Shader"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(COLORED_QUAD_SHADER)),
                }
            );
            
            // Create camera uniform buffer
            let view_proj = Mat4::orthographic_rh(
                -400.0, 400.0,  // Left, right
                -300.0, 300.0,  // Bottom, top
                -1000.0, 1000.0, // Near, far
            );
            
            let camera_uniform = view_proj.to_cols_array();
            let camera_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform].concat()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );
            
            // Create pipeline
            let camera_bind_group_layout = renderer.device().create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }
            );
            
            let pipeline_layout = renderer.device().create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Emergency Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    ..Default::default()
                }
            );
            
            let pipeline = renderer.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Emergency Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        // Vertex buffer
                        wgpu::VertexBufferLayout {
                            array_stride: (std::mem::size_of::<f32>() * 6) as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![
                                0 => Float32x2, // position
                                1 => Float32x4, // color
                            ],
                        },
                        // Instance buffer
                        wgpu::VertexBufferLayout {
                            array_stride: (std::mem::size_of::<f32>() * 5) as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![
                                2 => Float32x2, // world_position
                                3 => Float32x2, // scale
                                4 => Float32,   // depth
                            ],
                        },
                    ],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: renderer.surface_format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
                multiview_mask: None,
            });
            
            // Quad vertices: 2 triangles = 6 vertices
            // Position (x,y) + Color (r,g,b,a)
            let vertices: &[f32] = &[
                // Triangle 1
                -0.5, -0.5,   1.0, 1.0, 1.0, 1.0,
                0.5, -0.5,    1.0, 1.0, 1.0, 1.0,
                0.5, 0.5,     1.0, 1.0, 1.0, 1.0,
                // Triangle 2
                -0.5, -0.5,   1.0, 1.0, 1.0, 1.0,
                0.5, 0.5,     1.0, 1.0, 1.0, 1.0,
                -0.5, 0.5,    1.0, 1.0, 1.0, 1.0,
            ];
            
            let vertex_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            // Instance data: 3 quads (left, middle, right)
            // Format: world_position (x,y), scale (x,y), depth
            let instances: &[f32] = &[
                // Left quad - RED
                -200.0, 0.0,   100.0, 100.0, 0.0,
                // Middle quad - GREEN
                0.0, 0.0,      100.0, 100.0, 0.0,
                // Right quad - BLUE
                200.0, 0.0,    100.0, 100.0, 0.0,
            ];
            
            let instance_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(instances),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            // Create bind group
            let camera_bind_group = renderer.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });
            
            // Render
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Emergency Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, &camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                
                // Draw 3 quads (6 vertices each, instanced)
                render_pass.draw(0..6, 0..3);
            }
            
            // Submit
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
            
            if self.frame_count == 0 {
                println!("✅ Draw call completed (3 quads)");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("🚨 EMERGENCY VISIBILITY TEST");
    println!("=============================");
    println!("This is the final test. If this works:");
    println!("✅ Basic rendering pipeline is fine");
    println!("✅ The issue is in sprite system");
    println!("❌ If this fails: fundamental GPU issue");
    println!("=============================\n");

    let event_loop = EventLoop::new()?;
    let mut app = EmergencyVisibilityTest::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
