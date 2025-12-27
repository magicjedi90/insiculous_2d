//! 🎨 **FULLSCREEN MAGENTA TEST** - Fill entire screen without vertex buffers

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;

struct FullscreenMagentaTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl FullscreenMagentaTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            pipeline: None,
        }
    }
    
    fn create_pipeline(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fullscreen Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Full-screen triangle covering entire NDC
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0)
    );
    
    out.clip_position = vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pure magenta for maximum visibility
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
"#)),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fullscreen Pipeline Layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fullscreen Pipeline"),
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

impl ApplicationHandler<()> for FullscreenMagentaTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🎨 FULLSCREEN MAGENTA TEST");
        println!("===========================");
        println!("Will render fullscreen magenta triangle");
        println!("Window stays open for 3 seconds");
        println!("===========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🎨 FULLSCREEN MAGENTA TEST")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        let pipeline = self.create_pipeline(renderer.device(), renderer.surface_format());
        
        self.renderer = Some(renderer);
        self.pipeline = Some(pipeline);
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
            if FRAME_COUNT < 180 { // ~3 seconds at 60fps
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME_COUNT += 1;
            } else {
                println!("\n⏰ 3 seconds elapsed - exiting...");
                event_loop.exit();
            }
        }
    }
}

impl FullscreenMagentaTest {
    fn render_frame(&mut self) {
        if let (Some(renderer), Some(pipeline)) = (&self.renderer, &self.pipeline) {
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Fullscreen Encoder"),
                }
            );
            
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Fullscreen Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                
                render_pass.set_pipeline(pipeline);
                render_pass.draw(0..3, 0..1);
            }
            
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 FULLSCREEN MAGENTA TEST");
    println!("=========================");
    println!("👀 LOOK FOR: ENTIRE SCREEN FILLED WITH MAGENTA");
    println!("");
    println!("✅ If you see magenta: Basic wgpu pipeline works");
    println!("❌ If you see blue/clear color: Something fundamental is broken");
    println!("=========================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = FullscreenMagentaTest::new();
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
