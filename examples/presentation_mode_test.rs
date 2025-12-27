use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use std::time::{Duration, Instant};

/// Presentation Mode Test - Tests different WGPU present modes to isolate compositor issues
/// This creates a simple colored quad and tests each present mode
fn main() {
    env_logger::init();
    println!("🧪 PRESENTATION MODE TEST - Testing WGPU Present Modes");
    println!("=" .repeat(55));
    
    let present_modes_to_test = [
        ("IMMEDIATE", wgpu::PresentMode::Immediate),
        ("FIFO_RELAXED", wgpu::PresentMode::FifoRelaxed),
        ("FIFO", wgpu::PresentMode::Fifo),
        ("MAILBOX", wgpu::PresentMode::Mailbox),
    ];
    
    for (mode_name, present_mode) in present_modes_to_test.iter() {
        println!("\n🎯 Testing Present Mode: {} ({:?})", mode_name, present_mode);
        println!("-" .repeat(45));
        
        match test_present_mode(*present_mode) {
            Ok(result) => {
                println!("✅ {}: {}", mode_name, result);
            }
            Err(e) => {
                println!("❌ {}: FAILED - {}", mode_name, e);
            }
        }
        
        // Small delay between tests
        std::thread::sleep(Duration::from_millis(500));
    }
    
    println!("\n🎯 Presentation Mode Test Complete");
    println!("💡 Summary:");
    println!("   - If all modes show colored quads: rendering pipeline is working");
    println!("   - If some modes fail: presentation/compositor issue");
    println!("   - If all modes fail: core rendering issue");
}

fn test_present_mode(present_mode: wgpu::PresentMode) -> Result<String, String> {
    let event_loop = EventLoop::new().map_err(|e| format!("Event loop failed: {}", e))?;
    let window = WindowBuilder::new()
        .with_title(format!("Present Mode: {:?}", present_mode))
        .with_inner_size(winit::dpi::LogicalSize::new(400, 300))
        .with_resizable(false)
        .with_visible(true)
        .build(&event_loop)
        .map_err(|e| format!("Window creation failed: {}", e))?;
    
    // Center window
    window.set_outer_position(winit::dpi::LogicalPosition::new(200, 200));
    
    println!("🪟 Window created and positioned");
    
    // Initialize WGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    
    let surface = unsafe { instance.create_surface(&window) }
        .map_err(|e| format!("Surface creation failed: {}", e))?;
    
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })).ok_or_else(|| "No suitable adapter found".to_string())?;
    
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("present_mode_test_device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    )).map_err(|e| format!("Device request failed: {}", e))?;
    
    // Get surface capabilities
    let surface_caps = surface.get_capabilities(&adapter);
    println!("📊 Available present modes: {:?}", surface_caps.present_modes);
    println!("📊 Testing with: {:?}", present_mode);
    
    let surface_format = surface_caps.formats.iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: 400,
        height: 300,
        present_mode,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    
    surface.configure(&device, &config);
    println!("🎨 Surface configured: {}x{}, format: {:?}", config.width, config.height, surface_format);
    
    // Create colored quad shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("present_mode_test_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("colored_quad.wgsl")),
    });
    
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("present_mode_pipeline_layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("present_mode_render_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    
    println!("🔧 Render pipeline created");
    
    // Track rendering results
    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut render_errors = Vec::new();
    
    let result = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("🚪 Window close requested");
                    elwt.exit();
                }
                WindowEvent::RedrawRequested => {
                    if frame_count < 5 {
                        match surface.get_current_texture() {
                            Ok(surface_texture) => {
                                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                                
                                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("present_mode_encoder"),
                                });
                                
                                {
                                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("present_mode_render_pass"),
                                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                            view: &view,
                                            resolve_target: None,
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
                                    });
                                    
                                    render_pass.set_pipeline(&render_pipeline);
                                    render_pass.draw(0..6, 0..1); // Two triangles = quad
                                }
                                
                                queue.submit(std::iter::once(encoder.finish()));
                                surface_texture.present();
                                
                                frame_count += 1;
                                println!("  ✅ Frame {} rendered and presented", frame_count);
                                
                                if frame_count >= 5 {
                                    println!("🎉 Present mode test successful!");
                                    println!("💡 Look for a colored quad on dark gray background");
                                    println!("⏱️  Test completed in {:?}", start_time.elapsed());
                                    
                                    // Wait a bit to let user see the result
                                    std::thread::sleep(Duration::from_millis(1000));
                                    elwt.exit();
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Surface error: {:?}", e);
                                println!("❌ {}", error_msg);
                                render_errors.push(error_msg);
                                elwt.exit();
                            }
                        }
                    }
                }
                _ => {}
            }
            Event::AboutToWait => {
                if frame_count < 5 {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    });
    
    match result {
        Ok(()) => {
            if render_errors.is_empty() {
                Ok(format!("Rendered {} frames successfully", frame_count))
            } else {
                Err(format!("Render errors: {:?}", render_errors))
            }
        }
        Err(e) => Err(format!("Event loop error: {}", e)),
    }
}