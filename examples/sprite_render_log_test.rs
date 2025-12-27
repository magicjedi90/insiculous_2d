//! SPRITE RENDER LOG TEST - Log every render pass setup detail

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::Vec2;

struct SpriteRenderLogTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
}

impl SpriteRenderLogTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
        }
    }
}

impl ApplicationHandler<()> for SpriteRenderLogTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("Sprite Render Log");
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        
        println!("=== SPRITE RENDER LOG TEST ===");
        println!("Will log detailed render pass setup");
        println!("================================\n");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        static mut FRAME: u32 = 0;
        unsafe {
            if FRAME < 60 {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME += 1;
            } else {
                event_loop.exit();
            }
        }
    }
}

impl SpriteRenderLogTest {
    fn render(&mut self) {
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            // Create simple sprite
            let white_texture = TextureHandle { id: 0 };
            let sprite = Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0))
                .with_scale(Vec2::new(100.0, 100.0));
            
            let mut batcher = SpriteBatcher::new(100);
            batcher.add_sprite(&sprite);
            
            let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
            let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
            let texture_resources = std::collections::HashMap::new();
            
            println!("🎨 Rendering 1 sprite");
            println!("   Sprite pos: {:?}, scale: {:?}", sprite.position, sprite.scale);
            println!("   Batch: texture={:?}, instances={}", batch_refs[0].texture_handle, batch_refs[0].len());
            
            // Manually trace through render_with_sprites
            let frame = renderer.surface().get_current_texture().unwrap();
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            let mut encoder = renderer.device().create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                }
            );
            
            // Update camera
            sprite_pipeline.update_camera(&renderer.queue, &self.camera);
            println!("   ✅ Camera updated");
            
            // Create combined texture resources
            let mut combined_texture_resources = texture_resources.clone();
            if let Some(white_texture) = &renderer.white_texture {
                let white_texture_handle = TextureHandle { id: 0 };
                combined_texture_resources.insert(white_texture_handle, white_texture.clone());
                println!("   ✅ White texture added to resources");
            }
            
            // Draw sprites
            sprite_pipeline.draw(
                &mut encoder, 
                &self.camera, 
                &combined_texture_resources, 
                &batch_refs,
                &view,
                wgpu::Color { r: 0.05, g: 0.08, b: 0.15, a: 1.0 }
            );
            
            println!("   ✅ draw() called");
            
            renderer.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
            
            println!("   ✅ Frame presented\n");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = SpriteRenderLogTest {
        window: None,
        renderer: None,
        sprite_pipeline: None,
        camera: Camera2D::default(),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
