//! NDC QUAD TEST - Render quad directly in clip space

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;

struct NdcQuadTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    pipeline: Option<wgpu::RenderPipeline>,
}

const SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, -0.5), vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5), vec2<f32>(-0.5, 0.5)
    );
    out.clip_position = vec4<f32>(positions[idx], 0.0, 1.0);
    out.color = vec4<f32>(positions[idx].x + 0.5, positions[idx].y + 0.5, 1.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

impl ApplicationHandler<()> for NdcQuadTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("NDC QUAD TEST");
        
        let window_attributes = WindowAttributes::default()
            .with_title("NDC QUAD TEST")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        
        // Create pipeline
        let shader = renderer.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NDC Quad"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });
        
        let pipeline_layout = renderer.device().create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Layout"),
                bind_group_layouts: &[],
                ..Default::default()
            }
        );
        
        let pipeline = renderer.device().create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("NDC Quad Pipeline"),
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
                        format: renderer.surface_format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                cache: None,
                multiview_mask: None,
            }
        );
        
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
            if FRAME_COUNT < 180 {
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

impl NdcQuadTest {
    fn render_frame(&mut self) {
        if let (Some(renderer), Some(pipeline)) = (&self.renderer, &self.pipeline) {
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
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
                render_pass.draw(0..6, 0..1);
            }
            
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NDC QUAD TEST - Should render colored quad\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = NdcQuadTest { window: None, renderer: None, pipeline: None };
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
