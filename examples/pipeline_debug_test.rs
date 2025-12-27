//! 🔧 **PIPELINE DEBUG TEST** - Creates pipeline with debug shader

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use wgpu::util::DeviceExt;
use glam::Vec2;

struct PipelineDebugTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    pipeline: Option<wgpu::RenderPipeline>,
    camera: Camera2D,
}

impl PipelineDebugTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            pipeline: None,
            camera: Camera2D::default(),
        }
    }
    
    fn create_debug_pipeline(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        println("🔧 Creating DEBUG pipeline with vertex position colors...");
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("vertex_debug_shader.wgsl"))),
        });
        
        // Create vertex buffer layout
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::sprite_data::SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3, // position
                1 => Float32x2, // tex_coords
                2 => Float32x4, // color
            ],
        };
        
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::sprite_data::SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                3 => Float32x2, // world_position
                4 => Float32,    // rotation
                5 => Float32x2, // scale
                6 => Float32x4, // tex_region
                7 => Float32x4, // color
                8 => Float32,    // depth
            ],
        };
        
        // Create bind group layouts
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Camera Bind Group Layout"),
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
        
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug Texture Bind Group Layout"),
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
            label: Some("Debug Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            ..Default::default()
        });
        
        // Create the render pipeline
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
            cache: None,
            multiview_mask: None,
        })
    }
}

impl ApplicationHandler<()> for PipelineDebugTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println("🔧 PIPELINE DEBUG TEST");
        println("======================");
        println("Testing with debug shader that outputs vertex position colors");
        println("Window stays open for 3 seconds");
        println("======================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔧 PIPELINE DEBUG TEST")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let mut renderer = pollster::block_on(renderer::init(window)).unwrap();
        println("✅ Renderer: {}", renderer.adapter_info());
        
        let pipeline = self.create_debug_pipeline(renderer.device(), renderer.surface_format());
        println("✅ Debug pipeline created");
        
        self.renderer = Some(renderer);
        self.pipeline = Some(pipeline);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
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
            if FRAME_COUNT < 180 { // 3 seconds
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

impl PipelineDebugTest {
    fn render_frame(&mut self) {
        if let (Some(renderer), Some(pipeline)) = (&self.renderer, &self.pipeline) {
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Debug Encoder"),
                }
            );
            
            // Create quad vertices
            let vertices = [
                // position, tex_coords, color
                -0.5f32, 0.5, 0.0,  0.0, 0.0,  1.0, 1.0, 1.0, 1.0,  // top-left
                0.5, 0.5, 0.0,     1.0, 0.0,  1.0, 1.0, 1.0, 1.0,  // top-right
                0.5, -0.5, 0.0,    1.0, 1.0,  1.0, 1.0, 1.0, 1.0,  // bottom-right
                -0.5, -0.5, 0.0,   0.0, 1.0,  1.0, 1.0, 1.0, 1.0,  // bottom-left
            ];
            
            let vertex_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
            let index_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );
            
            // Instance data
            let instances = [
                0.0f32, 0.0,    // position
                0.0,            // rotation
                200.0, 200.0,   // scale
                0.0, 0.0, 1.0, 1.0, // tex_region
                1.0, 1.0, 1.0, 1.0, // color (should be overridden by debug)
                0.0,            // depth
            ];
            
            let instance_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            // Camera uniform
            let view_proj = self.camera.view_projection_matrix().to_cols_array();
            let camera_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Debug Camera Buffer"),
                    contents: bytemuck::cast_slice(&[view_proj].concat()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );
            
            let camera_bind_group = renderer.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Debug Camera Bind Group"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });
            
            // White texture
            let white_texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Debug White Texture"),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            
            let white_data = [255u8, 255, 255, 255];
            renderer.queue().write_texture(
                white_texture.as_image_copy(),
                &white_data,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: None },
                wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            );
            
            let view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = renderer.device().create_sampler(&wgpu::SamplerDescriptor::default());
            
            let texture_bind_group = renderer.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Debug Texture Bind Group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            
            // Render
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Debug Sprite Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &camera_bind_group, &[]);
                render_pass.set_bind_group(1, &texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
            
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
            
            static mut FIRST_RENDER: bool = true;
            unsafe {
                if FIRST_RENDER {
                    println("✅ First debug render call succeeded");
                    FIRST_RENDER = false;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println("🔧 PIPELINE DEBUG TEST");
    println("======================");
    println("Testing sprite pipeline with debug shader...");
    println("");
    println("👀 LOOK FOR: Colored gradients showing vertex positions");
    println("📋 If you see colors: vertex shader IS working");
    println("📋 If you see only background: vertex shader is NOT working");
    println("======================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = PipelineDebugTest::new();
    event_loop.run_app(&mut app)?;
    
    println("\n✅ Test complete!");
    Ok(())
}
