use wgpu::{Color, Device, PresentMode, Queue, Surface, SurfaceConfiguration, TextureUsages};
use winit::window::Window;

pub struct Renderer {
    surface: Surface<'static>, // no lifetime on the struct now
    device: Device,
    queue: Queue,
}
impl<'window> Renderer {
    pub async fn new(window: &'window Window) -> Self {
        let instance = wgpu::Instance::default();
        let raw_surface = instance.create_surface(window).unwrap();
        let surface: Surface<'static> = unsafe { std::mem::transmute(raw_surface) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();

        let size = window.inner_size();
        let format = surface.get_capabilities(&adapter).formats[0];
        let cfg = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![format],
        };
        surface.configure(&device, &cfg);

        Self {
            surface,
            device,
            queue,
        }
    }

    pub fn clear_screen(&mut self, rgba: [f64; 4]) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
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
        self.queue.submit([encoder.finish()]);
        frame.present();
    }
}
