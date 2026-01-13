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
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,
    clear_color: wgpu::Color,
    /// White texture resource for colored sprites (multiply by white instead of transparent black)
    white_texture: Option<crate::sprite_data::TextureResource>,
}

impl Renderer {
    /// Create a new renderer with an existing window
    ///
    /// This method properly manages the surface lifetime by:
    /// 1. Creating the instance and surface first
    /// 2. The surface gets `'static` lifetime because `Arc<Window>` is `'static`
    /// 3. WGPU 28.0.0 supports `Arc<Window>` -> `Surface<'static>` conversion
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        // Create a WGPU instance
        let instance = wgpu::Instance::default();

        // Create a surface with 'static lifetime
        // Arc<Window> implements Into<SurfaceTarget<'static>> because Arc<T> is 'static when T: 'static
        // This is safe and doesn't require unsafe code - WGPU 28.0.0 handles this correctly
        let surface: Surface<'static> = instance
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
                    experimental_features: Default::default(),
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
        
        // Wrap device and queue in Arc for sharing
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Create white texture for colored sprites
        let white_texture = Self::create_white_texture_resource(&device, &queue);

        Ok(Self {
            window,
            surface,
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
            white_texture: Some(white_texture),
        })
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }

    /// Acquire the current surface texture for rendering.
    ///
    /// Returns:
    /// - `Ok(Some(frame))` - Successfully acquired frame, proceed with rendering
    /// - `Ok(None)` - Transient error, skip this frame
    /// - `Err(_)` - Fatal or recoverable error that needs handling
    fn acquire_frame(&self) -> Result<Option<wgpu::SurfaceTexture>, RendererError> {
        match self.surface.get_current_texture() {
            Ok(frame) => Ok(Some(frame)),
            Err(wgpu::SurfaceError::Lost) => {
                // Surface was lost, return error so caller can recreate it
                Err(RendererError::SurfaceError("Surface lost".to_string()))
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // Fatal error, we can't recover
                Err(RendererError::RenderingError("Out of memory".to_string()))
            }
            Err(e) => {
                // Other errors (Timeout, Outdated) can be logged and skipped
                log::warn!("Surface error: {:?}, skipping frame", e);
                Ok(None)
            }
        }
    }

    /// Render a frame
    pub fn render(&self) -> Result<(), RendererError> {
        // Get a frame (returns None if we should skip this frame)
        let frame = match self.acquire_frame()? {
            Some(frame) => frame,
            None => return Ok(()),
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
                multiview_mask: None,
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
        sprite_pipeline: &mut crate::sprite::SpritePipeline,
        camera: &crate::sprite_data::Camera2D,
        texture_resources: &std::collections::HashMap<crate::texture::TextureHandle, crate::sprite_data::TextureResource>,
        sprite_batches: &[&crate::sprite::SpriteBatch]
    ) -> Result<(), RendererError> {
        // Create a combined texture resources map that includes the white texture for colored sprites
        let mut combined_texture_resources = texture_resources.clone();

        // Add the white texture if it exists, using a special handle for colored sprites
        if let Some(white_texture) = &self.white_texture {
            let white_texture_handle = crate::texture::TextureHandle { id: 0 }; // Use handle 0 for white texture
            combined_texture_resources.insert(white_texture_handle, white_texture.clone());
        }

        // Prepare sprites - update instance buffer with sprite data
        sprite_pipeline.prepare_sprites(&self.queue, sprite_batches);

        self.render_with_sprites_internal(sprite_pipeline, camera, &combined_texture_resources, sprite_batches)
    }

    /// Internal method to render sprites with the combined texture resources
    fn render_with_sprites_internal(
        &self,
        sprite_pipeline: &mut crate::sprite::SpritePipeline,
        camera: &crate::sprite_data::Camera2D,
        texture_resources: &std::collections::HashMap<crate::texture::TextureHandle, crate::sprite_data::TextureResource>,
        sprite_batches: &[&crate::sprite::SpriteBatch]
    ) -> Result<(), RendererError> {
        // Get a frame (returns None if we should skip this frame)
        let frame = match self.acquire_frame()? {
            Some(frame) => frame,
            None => return Ok(()),
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

        // Update camera - do this before rendering
        sprite_pipeline.update_camera(&self.queue, camera);
        
        // Draw sprites directly - this will handle clearing and drawing in one render pass
        sprite_pipeline.draw(&mut encoder, camera, texture_resources, sprite_batches, &view, self.clear_color);

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

    /// Get a shared reference to the device (clones the Arc).
    ///
    /// Use this when you need to store the device or pass ownership to another struct.
    /// For temporary usage within a function, prefer `device_ref()` instead.
    pub fn device(&self) -> Arc<Device> {
        Arc::clone(&self.device)
    }

    /// Get a shared reference to the queue (clones the Arc).
    ///
    /// Use this when you need to store the queue or pass ownership to another struct.
    /// For temporary usage within a function, prefer `queue_ref()` instead.
    pub fn queue(&self) -> Arc<Queue> {
        Arc::clone(&self.queue)
    }

    /// Get a borrowed reference to the device.
    ///
    /// Use this for temporary access within a function without cloning the Arc.
    /// For storing or sharing ownership, use `device()` instead.
    pub fn device_ref(&self) -> &Device {
        &self.device
    }

    /// Get a borrowed reference to the queue.
    ///
    /// Use this for temporary access within a function without cloning the Arc.
    /// For storing or sharing ownership, use `queue()` instead.
    pub fn queue_ref(&self) -> &Queue {
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

    /// Get a reference to the surface (for diagnostic purposes)
    pub fn surface(&self) -> &Surface<'_> {
        &self.surface
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

    /// Create a white texture resource for colored sprites
    fn create_white_texture_resource(device: &Device, queue: &Queue) -> crate::sprite_data::TextureResource {
        use crate::sprite_data::TextureResource;
        use std::sync::Arc;
        
        log::info!("Creating white texture resource for colored sprites");
        
        // Create a 1x1 white texture
        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        // Create white pixel data (1, 1, 1, 1) - RGBA all 255 for white
        let white_pixel: [u8; 4] = [255, 255, 255, 255];
        
        // Write the white pixel data to the texture using the queue
        queue.write_texture(
            texture.as_image_copy(),
            &white_pixel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        log::info!("White texture created successfully with pixel data (255,255,255,255)");

        TextureResource::new(device, texture)
    }

    /// Get the white texture resource for colored sprites
    pub fn white_texture(&self) -> Option<&crate::sprite_data::TextureResource> {
        self.white_texture.as_ref()
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