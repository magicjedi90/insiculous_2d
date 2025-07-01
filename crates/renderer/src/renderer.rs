//! WGPU renderer implementation.

use std::sync::Arc;
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::{event_loop::EventLoop, window::Window};

use crate::error::RendererError;
use crate::window::{create_window, WindowConfig};

/// The main renderer struct
pub struct Renderer<'a> {
    window: Arc<Window>,
    instance: Instance,
    surface: Surface<'a>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    clear_color: wgpu::Color,
}

impl<'a> Renderer<'a> {
    /// Create a new renderer
    pub async fn new() -> Result<Self, RendererError> {
        // Create an event loop and window
        let event_loop = EventLoop::new().map_err(|e| RendererError::WindowCreationError(e.to_string()))?;
        let window_config = WindowConfig::default();
        let window = Arc::new(create_window(&window_config, &event_loop)?);

        // Create WGPU instance
        let instance = wgpu::Instance::default();

        // Create surface
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| RendererError::SurfaceCreationError(e.to_string()))?;

        // Get adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .or_else(|_| Err(RendererError::AdapterCreationError("No suitable adapter found".to_string())))?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Primary device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    trace: Default::default(),
                },
            )
            .await
            .map_err(|e| RendererError::DeviceCreationError(e.to_string()))?;

        // Configure surface
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        })
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }

    /// Render a frame
    pub fn render(&self) -> Result<(), RendererError> {
        // Get a frame
        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| RendererError::RenderingError(e.to_string()))?;

        // Create a view
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create a command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Create a render pass
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // In a real implementation, we would draw things here
        }

        // Submit the command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        // Present the frame
        frame.present();

        Ok(())
    }

    /// Get a reference to the window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get a reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
