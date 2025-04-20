// crates/renderer/src/lib.rs
use wgpu::{
    Color, CompositeAlphaMode, Device, PresentMode, Queue, Surface, SurfaceConfiguration,
    TextureUsages,
};
use winit::window::Window;

/// GPU renderer: holds surface, device, queue, and swapchain configuration
pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl Renderer {
    /// Asynchronously create a new Renderer for the given window
    pub async fn new(window: &Window) -> Self {
        // 1. Create WGPU instance and surface
        let instance = wgpu::Instance::default();
        let raw_surface = instance.create_surface(window).unwrap();
        // It's safe to transmute to 'static
        let surface: Surface = unsafe { std::mem::transmute(raw_surface) };

        // 2. Request an adapter compatible with the surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        // 3. Request device and queue
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        // 4. Configure the surface
        let size = window.inner_size();
        let format = surface.get_capabilities(&adapter).formats[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![format],
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
        }
    }

    // Change clear_screen to take &self
    /// Clear the screen with a given color [r,g,b,a]
    pub fn clear_screen(&self, rgba: [f64; 4]) {
        // Acquire next frame
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // Encode commands
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: rgba[0],
                            g: rgba[1],
                            b: rgba[2],
                            a: rgba[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        // Submit and present
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    /// Get a reference to the WGPU device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a reference to the command queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get the swapchain texture format
    pub fn target_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get the current target size (width, height)
    pub fn target_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Helper to run custom render passes (e.g., egui)
    pub fn with_encoder<F: FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView)>(&self, f: F) {
        // Acquire next frame
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // Encode commands
        let mut encoder = self.device.create_command_encoder(&Default::default());
        f(&mut encoder, &view);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn render_egui(
        &self,
        egui_renderer: &mut egui_wgpu::Renderer,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_desc: &egui_wgpu::ScreenDescriptor
    ) {
        // Get the current frame
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui_encoder"),
        });
        
        // Update the egui buffers
        egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            paint_jobs,
            screen_desc
        );
        
        // Create render pass and forget its lifetime
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // Forget the lifetime of the render pass
        let mut rpass = unsafe { std::mem::transmute::<_, wgpu::RenderPass<'static>>(rpass) };
        
        // Render egui
        egui_renderer.render(&mut rpass, paint_jobs, screen_desc);
        
        // Drop the render pass
        drop(rpass);
        
        // Submit work
        self.queue.submit(std::iter::once(encoder.finish()));
        
        // Present
        frame.present();
    }
    
    // Helper method to update textures
    pub fn update_egui_texture(
        &self,
        egui_renderer: &mut egui_wgpu::Renderer,
        id: egui::TextureId,
        delta: &egui::epaint::ImageDelta
    ) {
        egui_renderer.update_texture(
            &self.device,
            &self.queue,
            id,
            delta
        );
    }
    
    // Helper to free textures
    pub fn free_egui_texture(
        &self,
        egui_renderer: &mut egui_wgpu::Renderer,
        id: &egui::TextureId
    ) {
        egui_renderer.free_texture(id);
    }
}