//! 🎯 **MINIMAL SPRITE TEST** - Strip down to bare vertex buffer rendering

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use wgpu::util::DeviceExt;

struct MinimalSpriteTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    pipeline: Option<wgpu::RenderPipeline>,
}

const MINIMAL_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

impl ApplicationHandler<()> for MinimalSpriteTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Minimal Sprite Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        
        // Create minimal pipeline with vertex buffer
        let shader = renderer.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Minimal"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(MINIMAL_SHADER)),
        });
        
        let layout = renderer.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Minimal Layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });
        
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 7) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x4,
            ],
        };
        
        let pipeline = renderer.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Minimal Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
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
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        
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
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn render_frame(&mut self) {
        if let (Some(renderer), Some(pipeline)) = (&self.renderer, &self.pipeline) {
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Create a quad with position + color
            // 2 triangles = 6 vertices
            let vertices: &[f32] = &[
                // position.xyz, color.rgba
                -0.5, 0.5, 0.0,    1.0, 0.0, 0.0, 1.0,  // top-left, red
                0.5, 0.5, 0.0,     0.0, 1.0, 0.0, 1.0,  // top-right, green
                0.5, -0.5, 0.0,    0.0, 0.0, 1.0, 1.0,  // bottom-right, blue
                
                -0.5, 0.5, 0.0,    1.0, 0.0, 0.0, 1.0,  // top-left, red  
                0.5, -0.5, 0.0,    0.0, 0.0, 1.0, 1.0,  // bottom-right, blue
                -0.5, -0.5, 0.0,   1.0, 1.0, 0.0, 1.0,  // bottom-left, yellow
            ];
            
            let vertex_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Encoder"),
                }
            );
            
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Pass"),
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
                
                render_pass.set_pipeline(pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..6, 0..1);
            }
            
            static mut FIRST_RENDER: bool = true;
            unsafe {
                if FIRST_RENDER {
                    println!("✅ First render - drawing quad from vertex buffer");
                    println!("🎯 EXPECTING: Colored quad (top-left red, top-right green, etc.)");
                    FIRST_RENDER = false;
                }
            }
            
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MINIMAL SPRITE TEST - Quad from vertex buffer\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = MinimalSpriteTest { window: None, renderer: None, pipeline: None };
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
