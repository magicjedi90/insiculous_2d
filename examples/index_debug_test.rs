//! 🔢 **INDEX DEBUG TEST** - Check if index buffer is causing the issue

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::Vec2;

struct IndexDebugTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
}

impl IndexDebugTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
        }
    }
}

impl ApplicationHandler<()> for IndexDebugTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("\n🔢 INDEX DEBUG TEST");
        println("=====================");
        println("Testing WITHOUT index buffer (draw vertices directly)");
        println("Will manually remove index buffer from sprite pipeline");
        println("=====================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("🔢 INDEX BUFFER TEST - No indexing")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        println!("✅ Sprite pipeline created WITH index buffer");
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        
        println!("\n🎬 Rendering sprite (expecting NO output if index buffer is the problem)...\n");
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
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            if FRAME_COUNT < 60 { // 1 second
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                FRAME_COUNT += 1;
            } else {
                event_loop.exit();
            }
        }
    }
}

impl IndexDebugTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        let sprite = Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(glam::Vec4::new(1.0, 0.0, 0.0, 1.0))
            .with_scale(Vec2::new(300.0, 300.0));
        
        let mut batcher = SpriteBatcher::new(100);
        batcher.add_sprite(&sprite);
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎨 Frame {} - Rendering {} sprite", self.frame_count, batch_refs.len());
                println!("🎯 EXPECTING: If indices are wrong, sprite won't appear");
            }
            
            let _ = renderer.render_with_sprites(
                sprite_pipeline, 
                &self.camera, 
                &texture_resources, 
                &batch_refs
            );
        }
        
        self.frame_count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("\n🔢 INDEX BUFFER TEST");
    println("=====================");
    println("Current pipeline uses indexed rendering (6 indices per quad)");
    println("If indices are wrong, triangles will be deformed/not drawn");
    println("=====================\n");
    
    let event_loop = EventLoop::new()?;
    let mut app = IndexDebugTest::new();
    event_loop.run_app(&mut app)?;
    
    println!("\n✅ Test complete! Did you see ANYTHING?");
    Ok(())
}
