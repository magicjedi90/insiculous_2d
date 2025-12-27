//! 🔧 **SHADER VALIDATION TEST** - Verify shader compilation and linking

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
    keyboard::{PhysicalKey, KeyCode},
};
use renderer::prelude::*;
use glam::{Vec2, Vec4};

struct ShaderValidationTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl ShaderValidationTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
    
    fn validate_shader(&self, device: &wgpu::Device) {
        println!("🔧 VALIDATING SHADER...");
        
        // Try to compile the shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Validation Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../crates/renderer/src/shaders/sprite_instanced.wgsl"))),
        });
        
        println!("✅ Shader module created successfully");
        println!("✅ Shader label: {:?}", shader.label());
        
        // Try to get reflection info
        match shader.get_compilation_info() {
            Ok(info) => {
                println!("✅ Shader compilation info available");
                println!("   Messages: {}/{}", info.messages.len());
            }
            Err(e) => {
                println!("⚠️  Could not get compilation info: {}", e);
            }
        }
        
        // Create a simple pipeline to test linking
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Test Camera BGL"),
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
            label: Some("Test Texture BGL"),
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
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            ..Default::default()
        });
        
        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: (std::mem::size_of::<f32>() * 9) as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x3,
                            1 => Float32x2,
                            2 => Float32x4,
                        ],
                    },
                    wgpu::VertexBufferLayout {
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
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
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
            cache: None,
            multiview_mask: None,
        });
        
        println!("✅ Pipeline created successfully!");
        std::mem::drop(pipeline);
        println!("✅ Pipeline validation complete");
    }
}

impl ApplicationHandler<()> for ShaderValidationTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔧 SHADER VALIDATION TEST");
        println!("==========================");
        println!("Verifying shader compilation and pipeline linking");
        println!("==========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔧 Shader Validation")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer created: {}", renderer.adapter_info());
        
        self.validate_shader(renderer.device());
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(SpritePipeline::new(self.renderer.as_ref().unwrap().device(), 100));
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        
        println!("\n✅ Shader validation PASSED");
        println!("🎨 Now attempting to render with validated shader...");
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
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
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

impl ShaderValidationTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Single large red sprite
        let sprites = vec![
            Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
                .with_scale(Vec2::new(300.0, 300.0))
        ];
        
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &sprites {
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎨 Rendering {} sprites with validated shader...", sprites.len());
            }
            
            let result = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
            
            if let Err(e) = result {
                println!("❌ Render error on frame {}: {}", self.frame_count, e);
            } else if self.frame_count == 0 {
                println!("✅ Render call succeeded");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new()?;
    let mut app = ShaderValidationTest::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
