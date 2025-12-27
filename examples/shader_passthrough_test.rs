// Direct shader test - passthrough shader with hardcoded positions

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

const PASSTHROUGH_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec4<f32>,
}

@vertex
fn vs_main(v: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip = vec4<f32>(v.pos, 1.0);
    out.color = v.col;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

struct ShaderPassthroughTest {
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl ApplicationHandler<()> for ShaderPassthroughTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default()).unwrap());
        self.window = Some(window.clone());
        
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })).unwrap();
        
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();
        let format = surface.get_capabilities(&adapter).formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Passthrough"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PASSTHROUGH_SHADER)),
        });
        
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout"),
            bind_group_layouts: &[],
            ..Default::default()
        });
        
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 9) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x2,
                2 => Float32x4,
            ],
        };
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
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
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        static mut FRAME: u32 = 0;
        unsafe {
            if FRAME < 60 {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME += 1;
            } else {
                std::process::exit(0);
            }
        }
    }
}

impl ShaderPassthroughTest {
    fn render(&mut self) {
        if let (Some(device), Some(queue), Some(surface), Some(pipeline)) = 
            (&self.device, &self.queue, &self.surface, &self.pipeline) {
            
            let frame = surface.get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Sprite-style vertices (pos.xyz + uv.xy + color.rgba)
            let vertices: &[f32] = &[
                -0.5, 0.5, 0.0,   0.0, 0.0,   1.0, 0.0, 0.0, 1.0,  // TL, red
                0.5, 0.5, 0.0,    1.0, 0.0,   0.0, 1.0, 0.0, 1.0,  // TR, green
                0.5, -0.5, 0.0,   1.0, 1.0,   0.0, 0.0, 1.0, 1.0,  // BR, blue
                -0.5, 0.5, 0.0,   0.0, 0.0,   1.0, 0.0, 0.0, 1.0,  // TL, red
                0.5, -0.5, 0.0,   1.0, 1.0,   0.0, 0.0, 1.0, 1.0,  // BR, blue
                -0.5, -0.5, 0.0,  0.0, 1.0,   1.0, 1.0, 0.0, 1.0,  // BL, yellow
            ];
            
            let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (vertices.len() * std::mem::size_of::<f32>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            
            vertex_buffer.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(vertices));
            vertex_buffer.unmap();
            
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });
            
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
            
            queue.submit(std::iter::once(encoder.finish()));
            frame.present();
            
            static mut FIRST: bool = true;
            unsafe {
                if FIRST {
                    println!("✅ PASSTHROUGH TEST RENDERED!");
                    FIRST = false;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = ShaderPassthroughTest {
        window: None,
        device: None,
        queue: None,
        surface: None,
        config: None,
        pipeline: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
