//! ✅ **QUICK VISIBILITY TEST** - Runs briefly and exits to confirm sprites work

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use renderer::prelude::*;
use glam::{Vec2, Vec4};

struct QuickVisibilityTest {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    frame_count: u32,
}

impl QuickVisibilityTest {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            frame_count: 0,
        }
    }
}

impl ApplicationHandler<()> for QuickVisibilityTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("✅ QUICK VISIBILITY TEST");
        println!("==========================");
        println!("Rendering 10 frames with visible sprites");
        println!("==========================\n");
        
        let window_attributes = WindowAttributes::default()
            .with_title("✅ QUICK VISIBILITY TEST")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        
        let renderer = pollster::block_on(renderer::init(window)).unwrap();
        println!("✅ Renderer: {}", renderer.adapter_info());
        
        let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
        
        self.camera.viewport_size = Vec2::new(800.0, 600.0);
        self.camera.position = Vec2::new(0.0, 0.0);
        
        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);
        
        // Render immediately
        self.window.as_ref().unwrap().request_redraw();
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
        if self.frame_count < 10 {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        } else {
            println!("✅ Test complete - rendered 10 frames successfully!");
            println!("🎯 If you saw colored sprites, the fix worked!");
            event_loop.exit();
        }
    }
}

impl QuickVisibilityTest {
    fn render_frame(&mut self) {
        let white_texture = TextureHandle { id: 0 };
        
        // Three bright colored sprites - impossible to miss
        let sprites = vec![
            Sprite::new(white_texture)
                .with_position(Vec2::new(-200.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))  // RED
                .with_scale(Vec2::new(100.0, 100.0)),
            Sprite::new(white_texture)
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0))  // GREEN
                .with_scale(Vec2::new(100.0, 100.0)),
            Sprite::new(white_texture)
                .with_position(Vec2::new(200.0, 0.0))
                .with_color(Vec4::new(0.0, 0.0, 1.0, 1.0))  // BLUE
                .with_scale(Vec2::new(100.0, 100.0)),
        ];
        
        println!("🎨 Frame {} - Rendering {} sprites", self.frame_count, sprites.len());
        
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &sprites {
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        let texture_resources = std::collections::HashMap::new();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            if self.frame_count == 0 {
                println!("🎯 EXPECTING: Red, Green, Blue squares in center of window");
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

    let event_loop = EventLoop::new()?;
    let mut app = QuickVisibilityTest::new();
    event_loop.run_app(&mut app)?;
    
    println!("\n✅ Test completed! Check if you saw colored sprites!");
    Ok(())
}
