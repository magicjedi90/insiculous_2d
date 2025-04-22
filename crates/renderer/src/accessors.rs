// crates/renderer/src/accessors.rs

use wgpu::{CommandEncoderDescriptor, TextureView};
use crate::core::Renderer;

impl Renderer {
    /// Get a reference to the WGPU device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get a reference to the command queue
    pub fn queue(&self) -> &wgpu::Queue {
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

    /// Helper to run custom render passes (e.g., with `egui`)
    pub fn with_encoder<F: FnOnce(&mut wgpu::CommandEncoder, &TextureView)>(&self, f: F) {
        // Acquire next frame
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // Encode commands
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());
        f(&mut encoder, &view);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
