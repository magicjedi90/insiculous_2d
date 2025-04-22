// crates/renderer/src/egui.rs

use std::mem::transmute;
use egui::{ClippedPrimitive, TextureId};
use egui::epaint::ImageDelta;
use egui_wgpu::{Renderer as EguiRenderer, ScreenDescriptor};
use wgpu::{CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};
use crate::core::Renderer;

impl Renderer {
    /// Render egui paint jobs to the screen
    pub fn render_egui(
        &self,
        egui_renderer: &mut EguiRenderer,
        paint_jobs: &[ClippedPrimitive],
        screen_desc: &ScreenDescriptor,
    ) {
        // Acquire frame & view
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // Create command encoder
        let mut encoder =
            self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("egui_encoder") });

        // Update the egui buffers
        egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, paint_jobs, screen_desc);

        // Begin render pass
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Transmute to 'static and render
        let mut rpass = unsafe { transmute::<_, wgpu::RenderPass<'static>>(render_pass) };
        egui_renderer.render(&mut rpass, paint_jobs, screen_desc);
        drop(rpass);

        // Submit & present
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    /// Helper method to update egui textures
    pub fn update_egui_texture(
        &self,
        egui_renderer: &mut EguiRenderer,
        id: TextureId,
        delta: &ImageDelta,
    ) {
        egui_renderer.update_texture(&self.device, &self.queue, id, delta);
    }

    /// Helper to free egui textures
    pub fn free_egui_texture(
        &self,
        egui_renderer: &mut EguiRenderer,
        id: &TextureId,
    ) {
        egui_renderer.free_texture(id);
    }
}
