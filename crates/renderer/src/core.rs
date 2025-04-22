// crates/renderer/src/core.rs

use std::mem::transmute;
use wgpu::{
    CompositeAlphaMode, Device, Instance, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::window::Window;

/// GPU renderer: holds surface, device, queue, and swapchain configuration
pub struct Renderer {
    pub(crate) surface: Surface<'static>,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) config: SurfaceConfiguration,
}

impl Renderer {
    /// Asynchronously create a new Renderer for the given window
    pub async fn new(window: &Window) -> Self {
        // 1. Create WGPU instance and surface
        let instance = Instance::default();
        let raw_surface = instance.create_surface(window).unwrap();
        // It's safe to transmute to 'static
        let surface: Surface = unsafe { transmute(raw_surface) };

        // 2. Request an adapter compatible with the surface
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
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
}
