//! 🔺 **MINIMAL TRIANGLE TEST** - Render a single triangle directly
//! 
//! This bypasses all sprite systems and renders a triangle directly to screen
//! to verify basic rendering pipeline works.

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
use glam::Vec2;

/// 🔺 Minimal triangle test - bypass all sprite systems
struct MinimalTriangleTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_count: u32,
}

impl MinimalTriangleTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            frame_count: 0,
        }
    }
}

impl ApplicationHandler<()> for MinimalTriangleTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🔺 MINIMAL TRIANGLE TEST");
        println!("=========================");
        println!("Bypassing all sprite systems");
        println!("Rendering single red triangle directly");
        println!("=========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔺 MINIMAL TRIANGLE TEST")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        println!("✅ Surface format: {:?}", renderer.surface_format());
        
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

impl MinimalTriangleTest {
    fn render_frame(&mut self) {
        if let Some(renderer) = &self.renderer {
            if self.frame_count == 0 {
                println!("🔺 Drawing triangle...");
                println!("🎯 EXPECTED: Red triangle on dark blue background");
            }
            
            // Get frame
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Create command encoder
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Triangle Test Encoder"),
                }
            );
            
            // Clear (dark blue)
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
                                b: 0.3,
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
            
            // Create simple shader
            let shader = renderer.device().create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Triangle Shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("minimal_triangle.wgsl"))),
            });
            
            // Create render pipeline
            let pipeline_layout = renderer.device().create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Triangle Pipeline Layout"),
                    bind_group_layouts: &[],
                    ..Default::default()
                }
            );
            
            let pipeline = renderer.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Triangle Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: (std::mem::size_of::<f32>() * 5) as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2, // position
                            1 => Float32x3, // color
                        ],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: renderer.surface_format(),
                        blend: Some(wgpu::BlendState::REPLACE),
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
            
            // Triangle vertices: position (x,y) + color (r,g,b)
            let vertices: &[f32] = &[
                // Top vertex - Red
                0.0, 0.5,    1.0, 0.0, 0.0,
                // Bottom-left - Green  
                -0.5, -0.5,  0.0, 1.0, 0.0,
                // Bottom-right - Blue
                0.5, -0.5,   0.0, 0.0, 1.0,
            ];
            
            let vertex_buffer = renderer.device().create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Triangle Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            
            // Render
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Triangle Render Pass"),
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
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..3, 0..1);
            }
            
            // Submit
            renderer.queue().submit(std::iter::once(encoder.finish()));
            frame.present();
            
            if self.frame_count == 0 {
                println!("✅ Draw call completed");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("🔺 MINIMAL TRIANGLE TEST");
    println!("=========================");
    println!("If this works, you should see:");
    println!("🟥 Red triangle top");
    println!("🟩 Green triangle bottom-left");
    println!("🟦 Blue triangle bottom-right");
    println!("=========================\n");

    let event_loop = EventLoop::new()?;
    let mut app = MinimalTriangleTest::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
