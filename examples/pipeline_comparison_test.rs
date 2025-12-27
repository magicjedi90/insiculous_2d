//! 🎯 **PIPELINE COMPARISON TEST** - Triangle works, sprites don't. Find the difference!

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use wgpu::util::DeviceExt;
use glam::Vec2;

struct PipelineComparisonTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    triangle_pipeline: Option<wgpu::RenderPipeline>,
    camera: Camera2D,
}

const TRIANGLE_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5)
    );
    out.clip_position = vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0); // Green
}
"#;

const SPRITE_DEBUG_SHADER: &str = r#"
// Camera uniform
struct Camera {
    view_projection: mat4x4<f32>,
    position: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct InstanceInput {
    @location(3) world_position: vec2<f32>,
    @location(4) rotation: f32,
    @location(5) scale: vec2<f32>,
    @location(6) tex_region: vec4<f32>,
    @location(7) color: vec4<f32>,
    @location(8) depth: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) debug_color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Simple passthrough - NO TRANSFORM
    // Just output vertex position directly to clip space
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    
    // Color based on vertex position
    out.debug_color = vec4<f32>(vertex.position.x + 0.5, vertex.position.y + 0.5, 1.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.debug_color;
}
"#;

impl PipelineComparisonTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            triangle_pipeline: None,
            camera: Camera2D::default(),
        }
    }
    
    fn create_triangle_pipeline(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(TRIANGLE_SHADER)),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simple Triangle Layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }
    
    fn create_sprite_pipeline_no_transform(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SPRITE_DEBUG_SHADER)),
        });
        
        // Camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Camera BGL"),
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
        });
        
        // Texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Texture BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Sprite Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            ..Default::default()
        });
        
        // Vertex buffer layout
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 9) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x2,
                2 => Float32x4,
            ],
        };
        
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 15) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                3 => Float32x2,
                4 => Float32,
                5 => Float32x2,
                6 => Float32x4,
                7 => Float32x4,
                8 => Float32,
            ],
        };
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            multiview_mask: None,
            cache: None,
        })
    }
}

impl ApplicationHandler<()> for PipelineComparisonTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println("🎯 PIPELINE COMPARISON TEST");
        println("============================");
        println("Testing WHY triangle works but sprites don't:");
        println("");
        println("1. Triangle: Direct clip-space positions");
        println("2. Sprite: Has instance data, transforms, camera");
        println("");
        println("If sprite test fails: problem is instance/vertex data");
        println("If sprite test works: problem is the math/transforms");
        println("============================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🎯 Pipeline Comparison Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println("✅ Renderer: {}", renderer.adapter_info());
        
        // Create both pipelines
        let triangle_pipeline = self.create_triangle_pipeline(renderer.device(), renderer.surface_format());
        let sprite_debug_pipeline = self.create_sprite_pipeline_no_transform(renderer.device(), renderer.surface_format());
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        self.triangle_pipeline = Some(triangle_pipeline);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        println("\n🎬 Both pipelines created - comparing...\n");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            if FRAME_COUNT < 300 { // 5 seconds
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME_COUNT += 1;
            } else {
                event_loop.exit();
            }
        }
    }
}

impl PipelineComparisonTest {
    fn render_frame(&mut self) {
        static mut RENDERED_ANYTHING: bool = false;
        
        if let (Some(renderer), Some(sprite_pipeline), Some(triangle_pipeline)) = 
            (&self.renderer, &self.sprite_pipeline, &self.triangle_pipeline) {
            
            // Get frame
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Comparison Encoder"),
                }
            );
            
            unsafe {
                if !RENDERED_ANYTHING {
                    RENDERED_ANYTHING = true;
                    println("🎨 First render - drawing both:");
                    println("   Top-left: Green triangle (WORKING)");
                    println("   Bottom-right: Sprite quad (?), should show vertex colors");
                    println("");
                    println("👀 OBSERVE: If sprite appears, vertex data is OK");
                    println("👀 If ONLY triangle appears: vertex/input issue");
                }
            }
            
            // Two separate render passes
            
            // Pass 1: Triangle (TOP-LEFT - should work)
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Triangle Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { 
                                r: 0.05, g: 0.08, b: 0.15, a: 1.0 
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                
                render_pass.set_pipeline(triangle_pipeline);
                
                // Offset triangle to top-left
                unsafe {
                    static mut OFFSET_SET: bool = false;
                    if !OFFSET_SET {
                        OFFSET_SET = true;
                        // Triangle vertices are in NDC, so scale and offset
                        // We'll use viewport transform for this simple test
                        // For now, just draw at center
                    }
                }
                render_pass.draw(0..3, 0..1);
            }
            
            // Pass 2: Sprite (BOTTOM-RIGHT - debug version)
            {
                // Create sprite test
                let white_texture = TextureHandle { id: 0 };
                let sprite = Sprite::new(white_texture)
                    .with_position(Vec2::new(200.0, -150.0))  // Bottom-right
                    .with_color(glam::Vec4::new(1.0, 1.0, 1.0, 1.0))
                    .with_scale(Vec2::new(200.0, 200.0));
                
                let mut batcher = SpriteBatcher::new(100);
                batcher.add_sprite(&sprite);
                
                let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
                let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
                let texture_resources = std::collections::HashMap::new();
                
                // Use regular sprite pipeline for comparison
                sprite_pipeline.draw(
                    &mut encoder, 
                    &self.camera, 
                    &texture_resources, 
                    &batch_refs,
                    &view,
                    wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }
                );
            }
            
            // Submit
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println("🎯 PIPELINE COMPARISON TEST");
    println("============================");
    println("Comparing working triangle vs broken sprites");
    println("");
    println("📋 Expected:");
    println("   🟢 Top-Left: Green triangle (ALWAYS WORKS)");
    println("   🎨 Bottom-Right: Colored quad (SPRITE TEST)");
    println("");
    println("If bottom-right shows colors: sprite vertices ARE working");
    println("If bottom-right is empty: vertex/instance data is the problem");
    println("============================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = PipelineComparisonTest::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
