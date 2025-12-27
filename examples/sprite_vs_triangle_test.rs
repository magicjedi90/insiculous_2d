//! 🎯 **SPRITE VS TRIANGLE TEST** - Direct comparison to find the bug
//! 
//! Renders both a sprite AND a triangle in the same frame to isolate differences

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use wgpu::util::DeviceExt;
use glam::Vec2;

struct SpriteVsTriangleTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    triangle_pipeline: Option<wgpu::RenderPipeline>,
    camera: Camera2D,
    frame_count: u32,
}

const TRIANGLE_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
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

impl SpriteVsTriangleTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            triangle_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
    
    fn create_triangle_pipeline(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(TRIANGLE_SHADER)),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Triangle Pipeline Layout"),
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
            cache: None,
            multiview_mask: None,
        })
    }
}

impl ApplicationHandler<()> for SpriteVsTriangleTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🎯 SPRITE VS TRIANGLE TEST");
        println!("===========================");
        println!("Rendering BOTH in same frame:");
        println!("🟢 Triangle (top-left)");
        println!("🔴 Sprite (bottom-right)");
        println!("===========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🎯 Sprite vs Triangle")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        let triangle_pipeline = self.create_triangle_pipeline(renderer.device(), renderer.surface_format());
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        self.triangle_pipeline = Some(triangle_pipeline);
        
        println!("✅ Both pipelines created");
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
                if self.frame_count >= 5 {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl SpriteVsTriangleTest {
    fn render_frame(&mut self) {
        if let (Some(renderer), Some(sprite_pipeline), Some(triangle_pipeline)) = 
            (&self.renderer, &self.sprite_pipeline, &self.triangle_pipeline) {
            
            if self.frame_count == 0 {
                println!("🎨 Frame {} - Rendering both...", self.frame_count);
            }
            
            // Get frame
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Create command encoder
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Combined Encoder"),
                }
            );
            
            // Render pass - clear FIRST, then draw both
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Combined Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1, g: 0.1, b: 0.1, a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                
                // Draw triangle FIRST (top)
                render_pass.set_pipeline(triangle_pipeline);
                render_pass.draw(0..3, 0..1);
                
                // // Now draw sprite using sprite system
                // // BUT we can't call draw() directly from here...
            }
            
            // So we need separate passes
            drop(render_pass);
            
            // Sprite pass
            sprite_pipeline.update_camera(&renderer.queue, &self.camera);
            
            // Create a test sprite at EXACT center
            let white_texture = TextureHandle { id: 0 };
            let sprite = Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))  // RED
                .with_scale(Vec2::new(150.0, 150.0));
            
            let mut batcher = SpriteBatcher::new(100);
            batcher.add_sprite(&sprite);
            
            let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
            let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
            let texture_resources = std::collections::HashMap::new();
            
            sprite_pipeline.draw(
                &mut encoder, 
                &self.camera, 
                &texture_resources, 
                &batch_refs,
                &view,
                wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }
            );
            
            // Submit
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
            
            if self.frame_count == 0 {
                println!("✅ Render complete - Check for:");
                println!("   🟢 Green triangle (top area)");
                println!("   🔴 Red sprite (center)");
            }
            
            self.frame_count += 1;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("🎯 SPRITE VS TRIANGLE TEST");
    println!("===========================");
    println!("This test renders BOTH in the same frame to compare:");
    println!("");
    println!("🟢 GREEN TRIANGLE (top area)");
    println!("   ✓ If you see this: basic rendering works");
    println!("");
    println!("🔴 RED SPRITE (center area)");
    println!("   ✓ If you see this: sprite system works");
    println!("   ✗ If you DON'T see this: sprite system is broken");
    println!("");
    println!("Expected result: BOTH should be visible!");
    println!("===========================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = SpriteVsTriangleTest::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
