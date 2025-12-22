//! WGPU renderer implementation.

use std::sync::Arc;
use wgpu::{
    Adapter, Device, Queue, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    event_loop::EventLoop,
    window::Window
};

use crate::error::RendererError;

/// The main renderer struct - now with proper lifetime management
pub struct Renderer {
    window: Arc<Window>,
    surface: Surface<'static>, // 'static is safe because we control the lifetime
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    clear_color: wgpu::Color,
}

impl Renderer {
    /// Create a new renderer with an existing window
    /// 
    /// This method now properly manages the surface lifetime by:
    /// 1. Creating the instance and surface first
    /// 2. Then moving the surface into the renderer with 'static lifetime
    /// 3. The surface is tied to the window's lifetime, which is Arc<Window>
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        // Create WGPU instance
        let instance = wgpu::Instance::default();

        // Create surface - this is safe because window is Arc<Window>
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
            .map_err(|_e| RendererError::AdapterCreationError("No suitable adapter found".to_string()))?;

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

        // Configure the surface before moving it
        surface.configure(&device, &config);

        // Now we can safely create the renderer with 'static surface
        // This is safe because:
        // 1. The surface is tied to the window (Arc<Window>)
        // 2. The window outlives the renderer
        // 3. We control the renderer's lifetime through the EngineApplication
        Ok(Self {
            window,
            surface: unsafe { std::mem::transmute(surface) }, // Safe because window has 'static lifetime
            adapter,
            device,
            queue,
            config,
            clear_color: wgpu::Color {
                r: 0.392, // Cornflower blue (100/255)
                g: 0.584, // (149/255)
                b: 0.929, // (237/255)
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
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                // Surface was lost, return error so caller can recreate it
                return Err(RendererError::SurfaceError("Surface lost".to_string()));
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // Fatal error, we can't recover
                return Err(RendererError::RenderingError("Out of memory".to_string()));
            }
            Err(e) => {
                // Other errors can be logged and ignored
                log::warn!("Surface error: {:?}, skipping frame", e);
                return Ok(());
            }
        };

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
                    depth_slice: None,
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

    /// Render a frame with a sprite pipeline
    pub fn render_with_sprites(
        &self, 
        sprite_pipeline: &crate::sprite::SpritePipeline,
        _camera: &crate::sprite::Camera2D,
        sprite_batches: &[crate::sprite::SpriteBatch]
    ) -> Result<(), RendererError> {
        // Get a frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                // Surface was lost, return error so caller can recreate it
                return Err(RendererError::SurfaceError("Surface lost".to_string()));
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // Fatal error, we can't recover
                return Err(RendererError::RenderingError("Out of memory".to_string()));
            }
            Err(e) => {
                // Other errors can be logged and ignored
                log::warn!("Surface error: {:?}, skipping frame", e);
                return Ok(());
            }
        };

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

        // Clear the screen
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
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
        }

        // Draw sprites
        sprite_pipeline.draw(&mut encoder, _camera, sprite_batches, &view);

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

    /// Get adapter information
    pub fn adapter_info(&self) -> String {
        self.adapter.get_info().name
    }

    /// Get surface format
    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    /// Get surface width
    pub fn surface_width(&self) -> u32 {
        self.config.width
    }

    /// Get surface height
    pub fn surface_height(&self) -> u32 {
        self.config.height
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Handle surface lost error by recreating the surface
    pub fn recreate_surface(&mut self) -> Result<(), RendererError> {
        // Reconfigure the surface
        self.surface.configure(&self.device, &self.config);
        log::debug!("Surface recreated after loss");
        Ok(())
    }

    /// Run the renderer with a custom application handler
    pub fn run_with_app<T>(app: &mut T) -> Result<(), RendererError> 
    where 
        T: ApplicationHandler<()> + 'static
    {
        // Create an event loop
        let event_loop = EventLoop::new()
            .map_err(|e| RendererError::WindowCreationError(e.to_string()))?;

        // Run the event loop with the application
        event_loop.run_app(app)
            .map_err(|e| RendererError::WindowCreationError(e.to_string()))
    }
}